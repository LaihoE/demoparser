use super::read_bits::DemoParserError;
use crate::parsing::entities_utils::*;
use crate::parsing::parser_settings::Parser;
use crate::parsing::read_bits::Bitreader;
use crate::parsing::sendtables::Decoder;
use crate::parsing::variants::PropData;
use ahash::HashMap;
use bit_reverse::LookupReverse;
use bitter::BitReader;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;

const NSERIALBITS: u32 = 17;
const STOP_READING_SYMBOL: u32 = 39;
const HUFFMAN_CODE_MAXLEN: u32 = 17;
// (64-MAX_LEN+1)
const RIGHTSHIFT_BITORDER: u32 = 46;

#[derive(Debug, Clone)]
pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: HashMap<[i32; 7], PropData>,
    pub entity_type: EntityType,
    pub history: HashMap<[i32; 7], Vec<PropData>>,
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

impl Parser {
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
            // self.gather_extra_info(&entity_id)?;
        }
        Ok(())
    }
    pub fn decode_entity_update(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
    ) -> Result<usize, DemoParserError> {
        let n_paths = self.parse_paths(bitreader)?;
        let entity = match self.entities.get_mut(&(entity_id)) {
            Some(ent) => ent,
            None => return Err(DemoParserError::EntityNotFound),
        };
        let class = match self.cls_by_id[entity.cls_id as usize].as_ref() {
            Some(cls) => cls,
            None => return Err(DemoParserError::ClassNotFound),
        };
        /*
        let v: Vec<Decoder> = self.paths[..n_paths]
            .iter()
            .map(|x| class.serializer.find_decoder(&x, 0))
            .collect();
        for d in v {
            let result = bitreader.decode(&d)?;
        }
        return Ok(22);
        */
        // Where the magic happens (all decoded values come from this loop)
        for path in &self.paths[..n_paths] {
            let decoder = class.serializer.find_decoder(&path, 0);
            let result = bitreader.decode(&decoder)?;
            // Can be used for debugging output
            if 1 == 0 {
                let debug_field =
                    class
                        .serializer
                        .debug_find_decoder(&path, 0, class.serializer.name.clone());
                if debug_field.full_name.contains("Round") {
                    println!("{:#?} {:?}", debug_field.full_name, result);
                }
            }

            // entity.props.insert(path.path, result);
        }
        Ok(n_paths)
    }

    fn create_new_entity(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: &i32,
    ) -> Result<(), DemoParserError> {
        let cls_id: u32 = bitreader.read_nbits(self.cls_bits.unwrap())?;
        // Both of these are not used. Don't think they are interesting for the parser
        let _serial = bitreader.read_nbits(NSERIALBITS)?;
        let _unknown = bitreader.read_varint();

        let entity_type = self.check_entity_type(&cls_id);
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
        };
        Ok(())
    }
    pub fn check_entity_type(&self, cls_id: &u32) -> EntityType {
        let class = self.cls_by_id[*cls_id as usize].as_ref().unwrap();
        match class.name.as_str() {
            "CCSPlayerController" => return EntityType::PlayerController,
            "CCSGameRulesProxy" => return EntityType::Rules,
            "CCSTeam" => return EntityType::Team,
            _ => {}
        }
        if class.name.contains("Projectile") {
            return EntityType::Projectile;
        }
        return EntityType::Normal;
    }
    pub fn parse_paths(&mut self, bitreader: &mut Bitreader) -> Result<usize, DemoParserError> {
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
            self.paths[idx] = fp;
            idx += 1;
        }
        Ok(idx)
    }
    pub fn gather_extra_info(&mut self, entity_id: &i32) -> Result<(), DemoParserError> {
        // Boring stuff.. function does some bookkeeping
        let entity = match self.entities.get_mut(entity_id) {
            Some(ent) => ent,
            None => return Err(DemoParserError::EntityNotFound),
        };
        if !(entity.entity_type == EntityType::PlayerController
            || entity.entity_type == EntityType::Team)
        {
            return Ok(());
        }
        let class = match self.cls_by_id[entity.cls_id as usize].as_ref() {
            Some(cls) => cls,
            None => return Err(DemoParserError::ClassNotFound),
        };
        if class.name == "CCSTeam" {
            if let Some(PropData::U32(t)) = self.get_prop_for_ent("CCSTeam.m_iTeamNum", entity_id) {
                match t {
                    1 => self.teams.team1_entid = Some(*entity_id),
                    2 => self.teams.team2_entid = Some(*entity_id),
                    3 => self.teams.team3_entid = Some(*entity_id),
                    _ => {}
                }
            }
            return Ok(());
        }
        let team_num = match self.get_prop_for_ent("CCSPlayerController.m_iTeamNum", entity_id) {
            Some(team_num) => match team_num {
                PropData::U32(team_num) => Some(team_num),
                // Signals that something went very wrong
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        let name = match self.get_prop_for_ent("CCSPlayerController.m_iszPlayerName", entity_id) {
            Some(name) => match name {
                PropData::String(name) => Some(name),
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        let steamid = match self.get_prop_for_ent("CCSPlayerController.m_steamID", entity_id) {
            Some(steamid) => match steamid {
                PropData::U64(steamid) => Some(steamid),
                _ => return Err(DemoParserError::IncorrectMetaDataProp),
            },
            None => None,
        };
        let player_entid =
            match self.get_prop_for_ent("CCSPlayerController.m_hPlayerPawn", entity_id) {
                Some(player_entid) => match player_entid {
                    PropData::U32(handle) => Some((handle & 0x7FF) as i32),
                    _ => return Err(DemoParserError::IncorrectMetaDataProp),
                },
                None => None,
            };
        self.players.insert(
            player_entid.unwrap(),
            PlayerMetaData {
                name: name,
                team_num: team_num,
                player_entity_id: player_entid,
                steamid: steamid,
                controller_entid: Some(*entity_id),
            },
        );
        Ok(())
    }
}

fn generate_fp() -> FieldPath {
    FieldPath {
        path: [-1, 0, 0, 0, 0, 0, 0],
        last: 0,
    }
}
