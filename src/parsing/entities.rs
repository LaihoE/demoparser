use crate::parsing::class::Class;
use crate::parsing::entities_utils::*;
use crate::parsing::parser_settings::Parser;
use crate::parsing::read_bits::Bitreader;
use crate::parsing::variants::PropData;
use ahash::HashMap;
use bit_reverse::LookupReverse;
use bitter::BitReader;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;
use smallvec::smallvec;

const NSERIALBITS: u32 = 17;
const STOP_READING_SYMBOL: u32 = 39;
const HUFFMAN_CODE_MAXLEN: u32 = 17;
// (64-MAX_LEN+1)
const RIGHTSHIFT_BITORDER: u32 = 46;

pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: HashMap<[i32; 7], PropData>,
}

#[derive(Debug, Clone)]
pub struct PlayerMetaData {
    pub player_entity_id: i32,
    pub steamid: u64,
    pub controller_entid: i32,
    pub name: String,
    pub team_num: i32,
}

impl Parser {
    pub fn get_prop_for_ent(&self, prop_name: &str, entity_id: &i32) -> Option<PropData> {
        let path = self.prop_name_to_path[prop_name][0];
        if let Some(ent) = self.entities.get(&entity_id) {
            if let Some(prop) = ent.props.get(&path) {
                return Some(prop.clone());
            }
        }
        None
    }
    #[inline(always)]
    pub fn parse_packet_ents(&mut self, bytes: &[u8]) -> Result<(), BitReaderError> {
        if !self.parse_entities {
            return Ok(());
        }
        let packet_ents: CSVCMsg_PacketEntities = Message::parse_from_bytes(&bytes).unwrap();
        Ok(self._parse_packet_ents(packet_ents)?)
    }
    #[inline(always)]
    fn _parse_packet_ents(
        &mut self,
        packet_ents: CSVCMsg_PacketEntities,
    ) -> Result<(), BitReaderError> {
        let n_updates = packet_ents.updated_entries();
        let data = match packet_ents.entity_data {
            Some(data) => data,
            None => return Err(BitReaderError::MalformedMessage),
        };
        let mut bitreader = Bitreader::new(&data);
        let mut entity_id: i32 = -1;

        for _ in 0..n_updates {
            entity_id += 1 + (bitreader.read_u_bit_var()? as i32);
            // ents.push(entity_id);
            // If the enitity should be deleted
            if bitreader.read_boolie()? {
                self.projectiles.remove(&entity_id);
                self.entities.remove(&entity_id);
                bitreader.read_boolie()?;
                continue;
            }
            let is_new_entity = bitreader.read_boolie()?;
            // Should we create the entity, or refer to an old one
            if is_new_entity {
                let cls_id = bitreader.read_nbits(self.cls_bits.unwrap())?;
                // Both of these are not used. Don't think they are interesting for the parser
                let _serial = bitreader.read_nbits(NSERIALBITS)?;
                let _unknown = bitreader.read_varint();

                let entity = Entity {
                    entity_id: entity_id,
                    cls_id: cls_id,
                    props: HashMap::default(),
                };

                self.entities.insert(entity_id, entity);

                if let Some(baseline_bytes) = self.baselines.get(&cls_id) {
                    let b = &baseline_bytes.clone();
                    let mut br = Bitreader::new(&b);
                    self.decode_entity_update(&mut br, entity_id, true)?;
                };

                if self.cls_by_id[cls_id as usize].as_ref().unwrap().name == "CCSGameRulesProxy" {
                    self.rules_entity_id = Some(entity_id);
                }
                if self.cls_by_id[cls_id as usize]
                    .as_ref()
                    .unwrap()
                    .name
                    .contains("Projectile")
                {
                    self.projectiles.insert(entity_id);
                }
                self.decode_entity_update(&mut bitreader, entity_id, false)?;
            } else {
                // Entity already exists, don't create it
                self.decode_entity_update(&mut bitreader, entity_id, false)?;
            }
        }
        Ok(())
    }
    #[inline(always)]
    pub fn parse_paths(&mut self, bitreader: &mut Bitreader) -> Result<usize, BitReaderError> {
        /*
        Create a field path by decoding using a Huffman tree.
        A field path is like a "path trough a struct" where
        the struct can have normal fields but also pointers
        to another structs.

        Example:

        The list will be filled with these:

        Struct Field{
            wanted_information: Option<T>,
            Pointer: bool,
            fields: Option<Vec<Field>>
        },

        (struct is simplified for this example. In reality it also includes field name etc.)


        Path to each of the fields in the below fields list: [
            [0], [1, 0], [1, 1], [2]    <-- This function generates these
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
        Not sure what is the maximum depth of these structs are, but others seem to use
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
        // Here we peek ("HUFFMAN_CODE_MAXLEN" == 17) amount of bits and see from a table what
        // symbol it maps to and how many bits should be read.
        // The symbol is then mapped into an op for filling the field path.
        loop {
            bitreader.reader.refill_lookahead();
            let peek_wrong_order = bitreader.reader.peek(HUFFMAN_CODE_MAXLEN);
            let peekbits = peek_wrong_order.swap_bits() >> RIGHTSHIFT_BITORDER;
            // Check if first bit is zero then symbol should be zero.
            // Don't know how a lookup table could handle this.
            let symbol = match peek_wrong_order & 1 {
                0 => 0,
                _ => self.huffman_lookup_table[peekbits as usize],
            };
            let n_skip_bits = self.symbol_bits[symbol as usize];
            bitreader.reader.consume((n_skip_bits) as u32);
            if symbol == STOP_READING_SYMBOL {
                break;
            }
            do_op(symbol, bitreader, &mut fp)?;
            self.paths[idx] = fp;
            idx += 1;
        }
        Ok(idx)
    }
    #[inline(always)]
    pub fn decode_entity_update(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        is_baseline: bool,
    ) -> Result<(), BitReaderError> {
        let n_paths = self.parse_paths(bitreader)?;
        let entity = match self.entities.get_mut(&(entity_id)) {
            Some(ent) => ent,
            None => return Err(BitReaderError::EntityNotFound),
        };
        let class = match self.cls_by_id[entity.cls_id as usize].as_ref() {
            Some(cls) => cls,
            None => return Err(BitReaderError::ClassNotFound),
        };
        if class.name == "CCSPlayerController" {
            // hacky solution for now

            let player_md = Parser::fill_player_data(
                &self.paths[..n_paths],
                bitreader,
                class,
                entity,
                is_baseline,
            )?;

            if player_md.player_entity_id != -1 {
                self.players.insert(player_md.player_entity_id, player_md);
            }
        } else {
            for path in &self.paths[..n_paths] {
                // probably problem with baseline, this seems to fix
                if is_baseline && bitreader.reader.bits_remaining().unwrap() < 32 {
                    break;
                }

                let decoder = class.serializer.find_decoder(&path, 0, is_baseline);
                let result = bitreader.decode(&decoder);

                // println!("{:?}", result);
                /*
                let key = path_to_key(&path, cls.class_id);
                match self.pattern_cache.get(&key) {
                    Some(e) => {
                        let result = bitreader.decode(e);
                        continue;
                    }
                    None => {
                        let (name, f, decoder) = cls.serializer.find_decoder(&path, 0, is_baseline);
                        let result = bitreader.decode(&decoder);
                        self.pattern_cache.insert(key, decoder);
                        continue;
                    }
                }

                let (name, f, decoder) = cls.serializer.find_decoder(&path, 0, is_baseline);
                let result = bitreader.decode(&decoder);

                // println!("{} {} {:?} {:?}", name, cls.name, decoder, path);
                if cls.name == "CCSTeam" && name == "m_iTeamNum" {
                    if let PropData::U32(t) = result {
                        match t {
                            1 => self.teams.team1_entid = Some(entity_id),
                            2 => self.teams.team2_entid = Some(entity_id),
                            3 => self.teams.team3_entid = Some(entity_id),
                            _ => {}
                        }
                    }
                }
                if self.count_props {
                    self.props_counter
                        .entry(name.clone())
                        .and_modify(|counter| *counter += 1)
                        .or_insert(1);
                }

                if (name == "m_vecX" && f.var_name != "CBodyComponent")
                    || (name == "m_vecY" && f.var_name != "CBodyComponent")
                {
                } else {
                    // entity.props.insert(name, result);
                }
                */
            }
        }
        Ok(())
    }

    pub fn fill_player_data(
        paths: &[FieldPath],
        bitreader: &mut Bitreader,
        cls: &Class,
        entity: &mut Entity,
        is_baseline: bool,
    ) -> Result<PlayerMetaData, BitReaderError> {
        let mut player = PlayerMetaData {
            player_entity_id: -1,
            controller_entid: entity.entity_id,
            team_num: -1,
            name: "".to_string(),
            steamid: 0,
        };

        // m_iTeamNum 5
        // m_iszPlayerName 9
        // m_steamID 10
        // m_hPlayerPawn 38
        for path in paths {
            if is_baseline && bitreader.reader.bits_remaining().unwrap() < 32 {
                break;
            }
            let decoder = cls.serializer.find_decoder(&path, 0, is_baseline);
            let result = bitreader.decode(&decoder)?;
            entity.props.insert(path.path, result.clone());

            match path.path[0] {
                5 => {}
                9 => {
                    player.name = match result {
                        PropData::String(n) => n,
                        _ => "Broken name!".to_owned(),
                    };
                }
                10 => {
                    player.steamid = match result {
                        PropData::U64(xuid) => xuid,
                        _ => 99999999,
                    };
                }
                38 => {
                    player.player_entity_id = match result {
                        PropData::U32(handle) => {
                            // create helper value
                            entity.props.insert(
                                [69, 69, 69, 69, 69, 69, 69],
                                PropData::I32((handle & 0x7FF) as i32),
                            );
                            (handle & 0x7FF) as i32
                        }
                        _ => -1,
                    };
                }
                _ => {}
            }
        }

        Ok(player)
    }
}
use super::read_bits::BitReaderError;

fn generate_fp() -> FieldPath {
    FieldPath {
        path: [-1, 0, 0, 0, 0, 0, 0],
        last: 0,
    }
}
