use super::read_bits::DemoParserError;
use crate::collect_data::TYPEHM;
use crate::entities_utils::*;
use crate::parser_thread_settings::ParserThread;
use crate::read_bits::Bitreader;
use crate::variants::Variant;
use ahash::HashMap;
use bitter::BitReader;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use itertools::Itertools;
use protobuf::Message;

const NSERIALBITS: u32 = 17;
const STOP_READING_SYMBOL: u32 = 39;
const HUFFMAN_CODE_MAXLEN: u32 = 17;

#[derive(Debug, Clone)]
pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: HashMap<u32, Variant>,
    pub entity_type: EntityType,
    pub history: HashMap<[i32; 7], Vec<Variant>>,
}

#[derive(Debug, Clone)]
pub struct PlayerMetaData {
    pub player_entity_id: Option<i32>,
    pub steamid: Option<u64>,
    pub controller_entid: Option<i32>,
    pub name: Option<String>,
    pub team_num: Option<u32>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    PlayerController,
    Rules,
    Projectile,
    Team,
    Normal,
}
enum EntityCmd {
    Delete,
    CreateAndUpdate,
    Update,
}

impl ParserThread {
    pub fn parse_packet_ents(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
        let packet_ents: CSVCMsg_PacketEntities = Message::parse_from_bytes(&bytes).unwrap();
        let n_updates = packet_ents.updated_entries();

        let data = match packet_ents.entity_data {
            Some(data) => data,
            None => return Err(DemoParserError::MalformedMessage),
        };
        let mut bitreader = Bitreader::new(&data);
        let mut entity_id: i32 = -1;
        for _ in 0..n_updates {
            entity_id += 1 + (bitreader.read_u_bit_var()? as i32);
            // Read 2 bits to know which operation should be done to the entity.
            let cmd = match bitreader.read_nbits(2)? {
                0b01 => EntityCmd::Delete,
                0b11 => EntityCmd::Delete,
                0b10 => EntityCmd::CreateAndUpdate,
                0b00 => EntityCmd::Update,
                _ => panic!("impossible cmd"),
            };
            match cmd {
                EntityCmd::Delete => {
                    self.projectiles.remove(&entity_id);
                    self.entities.remove(&entity_id);
                }
                EntityCmd::CreateAndUpdate => {
                    self.create_new_entity(&mut bitreader, &entity_id)?;
                    self.update_entity(&mut bitreader, entity_id)?;
                }
                EntityCmd::Update => {
                    self.update_entity(&mut bitreader, entity_id)?;
                }
            }
        }
        Ok(())
    }
    pub fn update_entity(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
    ) -> Result<(), DemoParserError> {
        let n_updated_values = self.decode_entity_update(bitreader, entity_id)?;
        if n_updated_values > 0 {
            self.gather_extra_info(&entity_id)?;
        }
        Ok(())
    }
    pub fn decode_entity_update(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
    ) -> Result<usize, DemoParserError> {
        let n_paths = self.parse_paths(bitreader, &entity_id)?;

        for field_info in &self.paths[..n_paths] {
            let result = bitreader.decode(&field_info.decoder, &self.qf_mapper)?;
            if field_info.should_parse {
                let entity = match self.entities.get_mut(&(entity_id)) {
                    Some(ent) => ent,
                    None => return Err(DemoParserError::EntityNotFound),
                };
                entity.props.insert(field_info.df_pos as u32, result);
            }
        }
        Ok(n_paths)
    }

    pub fn parse_paths(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: &i32,
    ) -> Result<usize, DemoParserError> {
        /*
        Create a field path by decoding using a Huffman tree.
        The huffman tree can be found at the bottom of entities_utils.rs

        A field path is a "path trough a struct" where
        the struct can have normal fields but also pointers
        to other (nested) structs.

        Example:

        The array will be filled with these:

        Struct Field{
            wanted_information: Option<T>,
            Pointer: bool,
            fields: Option<Vec<Field>>
        },

        (struct is simplified for this example. In reality it also includes field name etc.)


        Path to each of the fields in the below fields list: [
            [0], [1, 0], [1, 1], [2]
        ]
        and they would map to:
        [0] => FloatDecoder,
        [1, 0] => IntegerDecoder,
        [1, 1] => StringDecoder,
        [2] => VectorDecoder,

        fields = [
            Field{
                wanted_information: FloatDecoder,
                pointer: false,
                fields: None,
            },
            Field{
                wanted_information: None,
                pointer: true,
                fields: Some(
                    [
                        Field{
                            wanted_information: IntegerDecoder,
                            pointer: false,
                            fields: Some(
                        },
                        Field{
                            wanted_information: StringDecoder,
                            pointer: flase,
                            fields: Some(
                        }
                    ]
                ),
            },
            Field{
                wanted_information: VectorDecoder,
                pointer: false,
                fields: None,
            },
        ]
        Not sure what the maximum depth of these structs are, but others seem to use
        7 as the max length of field path so maybe that?

        Personally I find this path idea horribly complicated. Why is this chosen over
        the way it was done in source 1 demos?
        */
        let entity = match self.entities.get(&(entity_id)) {
            Some(ent) => ent,
            None => return Err(DemoParserError::EntityNotFound),
        };

        let class = match self.cls_by_id.get(&entity.cls_id) {
            Some(cls) => cls,
            None => return Err(DemoParserError::ClassNotFound),
        };
        // Create an "empty" path ([-1, 0, 0, 0, 0, 0, 0])
        // For perfomance reasons have them always the same len
        let mut fp = generate_fp();
        let mut idx = 0;
        // Do huffman decoding with a lookup table instead of reading one bit at a time
        // and traversing a tree.
        // Here we peek ("HUFFMAN_CODE_MAXLEN" == 17) amount of bits and see from a table which
        // symbol it maps to and how many bits should be consumed from the stream.
        // The symbol is then mapped into an op for filling the field path.
        loop {
            bitreader.reader.refill_lookahead();
            let peeked_bits = bitreader.reader.peek(HUFFMAN_CODE_MAXLEN);
            let (symbol, code_len) = self.huffman_lookup_table[peeked_bits as usize];
            bitreader.reader.consume(code_len as u32);

            if symbol == STOP_READING_SYMBOL {
                break;
            }

            do_op(symbol, bitreader, &mut fp)?;
            self.paths[idx] = class.serializer.find_decoder(&fp, 0);
            /*
            let tmp = class
                .serializer
                .debug_find_decoder(&fp, 0, class.name.clone());
            if !tmp.full_name.contains("Player") {
                println!("{:?}", tmp.full_name);
            }
            */
            // We reuse one big vector for holding paths. Purely for performance.
            // Alternatively we could create a new vector in this function and return it.
            idx += 1;
        }
        Ok(idx)
    }
    pub fn gather_extra_info(&mut self, entity_id: &i32) -> Result<(), DemoParserError> {
        // Boring stuff.. function does some bookkeeping

        let entity = match self.entities.get(&(entity_id)) {
            Some(ent) => ent,
            None => return Err(DemoParserError::EntityNotFound),
        };

        if !(entity.entity_type == EntityType::PlayerController
            || entity.entity_type == EntityType::Team)
        {
            return Ok(());
        }

        if entity.entity_type == EntityType::Team {
            if let Some(Variant::U32(t)) = self.get_prop_for_ent(
                self.prop_controller
                    .special_ids
                    .team_team_num
                    .as_ref()
                    .unwrap(),
                entity_id,
            ) {
                match t {
                    1 => self.teams.team1_entid = Some(*entity_id),
                    2 => self.teams.team2_entid = Some(*entity_id),
                    3 => self.teams.team3_entid = Some(*entity_id),
                    _ => {}
                }
            }
            return Ok(());
        }

        let team_num = match self.get_prop_for_ent(
            self.prop_controller.special_ids.teamnum.as_ref().unwrap(),
            entity_id,
        ) {
            Some(team_num) => match team_num {
                Variant::U32(team_num) => Some(team_num),
                // Signals that something went very wrong
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        let name = match self.get_prop_for_ent(
            self.prop_controller
                .special_ids
                .player_name
                .as_ref()
                .unwrap(),
            entity_id,
        ) {
            Some(name) => match name {
                Variant::String(name) => Some(name),
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        let steamid = match self.get_prop_for_ent(
            self.prop_controller.special_ids.steamid.as_ref().unwrap(),
            entity_id,
        ) {
            Some(steamid) => match steamid {
                Variant::U64(steamid) => Some(steamid),
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        let player_entid = match self.get_prop_for_ent(
            self.prop_controller
                .special_ids
                .player_pawn
                .as_ref()
                .unwrap(),
            entity_id,
        ) {
            Some(player_entid) => match player_entid {
                Variant::U32(handle) => Some((handle & 0x7FF) as i32),
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        match player_entid {
            Some(e) => {
                if e != 2047 && steamid != Some(0) {
                    self.players.insert(
                        e,
                        PlayerMetaData {
                            name: name,
                            team_num: team_num,
                            player_entity_id: player_entid,
                            steamid: steamid,
                            controller_entid: Some(*entity_id),
                        },
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn create_new_entity(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: &i32,
    ) -> Result<(), DemoParserError> {
        let cls_id: u32 = bitreader.read_nbits(8)?;
        // Both of these are not used. Don't think they are interesting for the parser
        let _serial = bitreader.read_nbits(NSERIALBITS)?;
        let _unknown = bitreader.read_varint();

        let entity_type = self.check_entity_type(&cls_id)?;
        match entity_type {
            EntityType::Projectile => {
                self.projectiles.insert(*entity_id);
            }
            EntityType::Rules => self.rules_entity_id = Some(*entity_id),
            _ => {}
        };

        let entity = Entity {
            entity_id: *entity_id,
            cls_id: cls_id,
            props: HashMap::default(),
            entity_type: entity_type,
            history: HashMap::default(),
        };
        self.entities.insert(*entity_id, entity);
        // Insert baselines
        if let Some(baseline_bytes) = self.baselines.get(&cls_id) {
            let b = &baseline_bytes.clone();
            let mut br = Bitreader::new(&b);
            self.update_entity(&mut br, *entity_id)?;
        }
        Ok(())
    }
    pub fn check_entity_type(&self, cls_id: &u32) -> Result<EntityType, DemoParserError> {
        let class = match self.cls_by_id.get(&cls_id) {
            Some(cls) => cls,
            None => {
                return Err(DemoParserError::ClassNotFound);
            }
        };
        match class.name.as_str() {
            "CCSPlayerController" => return Ok(EntityType::PlayerController),
            "CCSGameRulesProxy" => return Ok(EntityType::Rules),
            "CCSTeam" => return Ok(EntityType::Team),
            _ => {}
        }
        if class.name.contains("Projectile") {
            return Ok(EntityType::Projectile);
        }
        return Ok(EntityType::Normal);
    }
}

fn generate_fp() -> FieldPath {
    FieldPath {
        path: [-1, 0, 0, 0, 0, 0, 0],
        last: 0,
    }
}
