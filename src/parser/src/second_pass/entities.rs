use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::find_field;
use crate::first_pass::sendtables::get_decoder_from_field;
use crate::first_pass::sendtables::get_propinfo;
use crate::first_pass::sendtables::Field;
use crate::first_pass::sendtables::FieldInfo;
use crate::second_pass::game_events::GameEventInfo;
use crate::second_pass::other_netmessages::Class;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::path_ops::*;
use crate::second_pass::variants::Variant;
use ahash::AHashMap;
use csgoproto::CsvcMsgPacketEntities;
use prost::Message;

const NSERIALBITS: u32 = 17;
const STOP_READING_SYMBOL: u8 = 39;
const HUFFMAN_CODE_MAXLEN: u32 = 17;

#[derive(Debug, Clone)]
pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: AHashMap<u32, Variant>,
    pub entity_type: EntityType,
}

#[derive(Debug, Clone, PartialEq)]
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
    C4,
}
enum EntityCmd {
    Delete,
    CreateAndUpdate,
    Update,
}

impl<'a> SecondPassParser<'a> {
    pub fn parse_packet_ents(&mut self, bytes: &[u8], is_fullpacket: bool) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
        let msg = match CsvcMsgPacketEntities::decode(bytes) {
            Err(_) => return Err(DemoParserError::MalformedMessage),
            Ok(msg) => msg,
        };

        let mut bitreader = Bitreader::new(msg.entity_data());
        let mut entity_id: i32 = -1;
        let mut events_to_emit = vec![];
        for _ in 0..msg.updated_entries() {
            entity_id += 1 + (bitreader.read_u_bit_var()? as i32);
            // Read 2 bits to know which operation should be done to the entity.
            let cmd = match bitreader.read_nbits(2)? {
                0b01 => EntityCmd::Delete,
                0b11 => EntityCmd::Delete,
                0b10 => EntityCmd::CreateAndUpdate,
                0b00 => EntityCmd::Update,
                _ => return Err(DemoParserError::ImpossibleCmd),
            };

            match cmd {
                EntityCmd::Delete => {
                    self.projectiles.remove(&entity_id);
                    if let Some(entry) = self.entities.get_mut(entity_id as usize) {
                        *entry = None;
                    }
                }
                EntityCmd::CreateAndUpdate => {
                    self.create_new_entity(&mut bitreader, &entity_id, &mut events_to_emit)?;
                    self.update_entity(&mut bitreader, entity_id, false, &mut events_to_emit, is_fullpacket)?;
                }
                EntityCmd::Update => {
                    if msg.has_pvs_vis_bits() > 0 {
                        // Most entities pass trough here. Seems like entities that are not updated.
                        if bitreader.read_nbits(2)? & 0x01 == 1 {
                            continue;
                        }
                    }
                    self.update_entity(&mut bitreader, entity_id, false, &mut events_to_emit, is_fullpacket)?;
                }
            }
        }
        if !events_to_emit.is_empty() {
            self.emit_events(events_to_emit)?;
        }
        Ok(())
    }

    pub fn update_entity(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        is_baseline: bool,
        events_to_emit: &mut Vec<GameEventInfo>,
        is_fullpacket: bool,
    ) -> Result<(), DemoParserError> {
        let n_updates = self.parse_paths(bitreader)?;
        let n_updated_values = self.decode_entity_update(bitreader, entity_id, n_updates, is_fullpacket, is_baseline, events_to_emit)?;
        if n_updated_values > 0 {
            self.gather_extra_info(&entity_id, is_baseline)?;
        }
        Ok(())
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
            if bitreader.bits_left < HUFFMAN_CODE_MAXLEN {
                bitreader.refill();
            }

            let peeked_bits = bitreader.peek(HUFFMAN_CODE_MAXLEN);
            let (symbol, code_len) = self.huffman_lookup_table[peeked_bits as usize];
            bitreader.consume(code_len as u32);
            if symbol == STOP_READING_SYMBOL {
                break;
            }
            do_op(symbol, bitreader, &mut fp)?;
            self.write_fp(&mut fp, idx)?;
            idx += 1;
        }
        Ok(idx)
    }

    pub fn decode_entity_update(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        n_updates: usize,
        is_fullpacket: bool,
        is_baseline: bool,
        events_to_emit: &mut Vec<GameEventInfo>,
    ) -> Result<usize, DemoParserError> {
        let entity = match self.entities.get_mut(entity_id as usize) {
            Some(Some(entity)) => entity,
            _ => return Err(DemoParserError::EntityNotFound),
        };
        let class = match self.cls_by_id.get(entity.cls_id as usize) {
            Some(cls) => cls,
            None => return Err(DemoParserError::ClassNotFound),
        };

        for path in self.paths.iter().take(n_updates) {
            let field = find_field(&path, &class.serializer)?;
            let field_info = get_propinfo(&field, path);
            let decoder = get_decoder_from_field(field)?;
            let result = bitreader.decode(&decoder, self.qf_mapper)?;

            if !is_fullpacket && !is_baseline {
                events_to_emit.extend(SecondPassParser::listen_for_events(entity, &result, field, field_info, &self.prop_controller));
            }
            if self.is_debug_mode {
                SecondPassParser::debug_inspect(
                    &result,
                    field,
                    self.tick,
                    field_info,
                    path,
                    is_fullpacket,
                    is_baseline,
                    class,
                    &entity.cls_id,
                    &entity_id,
                );
            }

            SecondPassParser::insert_field(entity, result, field_info);
        }
        Ok(n_updates)
    }
    pub fn debug_inspect(
        _result: &Variant,
        field: &Field,
        _tick: i32,
        field_info: Option<FieldInfo>,
        _path: &FieldPath,
        _is_fullpacket: bool,
        _is_baseline: bool,
        _cls: &Class,
        _cls_id: &u32,
        _entity_id: &i32,
    ) {
        if let Field::Value(_v) = field {
            if _v.full_name.contains("Services") {
                println!("{:?} {:?} {:?} {:?}", _path, field_info, _v.full_name, _result);
            }
        }
    }

    pub fn insert_field(entity: &mut Entity, result: Variant, field_info: Option<FieldInfo>) {
        if let Some(fi) = field_info {
            if fi.should_parse {
                entity.props.insert(fi.prop_id, result);
            }
        }
    }

    #[inline]
    fn write_fp(&mut self, fp_src: &mut FieldPath, idx: usize) -> Result<(), DemoParserError> {
        match self.paths.get_mut(idx) {
            Some(entry) => *entry = *fp_src,
            // need to extend vec (rare)
            None => {
                self.paths.resize(idx + 1, generate_fp());
                match self.paths.get_mut(idx) {
                    Some(entry) => *entry = *fp_src,
                    None => return Err(DemoParserError::VectorResizeFailure),
                }
            }
        }
        Ok(())
    }
    fn create_new_entity(&mut self, bitreader: &mut Bitreader, entity_id: &i32, _events_to_emit: &mut Vec<GameEventInfo>) -> Result<(), DemoParserError> {
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
            EntityType::C4 => self.c4_entity_id = Some(*entity_id),
            _ => {}
        };
        let entity = Entity {
            entity_id: *entity_id,
            cls_id,
            props: AHashMap::with_capacity(0),
            entity_type,
        };
        if self.entities.len() as i32 <= *entity_id {
            // if corrupt, this can cause oom allocations
            if *entity_id > 100000 {
                return Err(DemoParserError::EntityNotFound);
            }
            self.entities.resize(*entity_id as usize + 1, None);
        }
        match self.entities.get_mut(*entity_id as usize) {
            Some(entry) => *entry = Some(entity),
            None => return Err(DemoParserError::VectorResizeFailure),
        };
        // Insert baselines
        if let Some(baseline_bytes) = self.baselines.get(&cls_id) {
            let b = &baseline_bytes.clone();
            let mut br = Bitreader::new(&b);
            self.update_entity(&mut br, *entity_id, true, &mut vec![], false)?;
        }
        Ok(())
    }

    pub fn check_entity_type(&self, cls_id: &u32) -> Result<EntityType, DemoParserError> {
        let class = match self.cls_by_id.get(*cls_id as usize) {
            Some(cls) => cls,
            None => {
                return Err(DemoParserError::ClassNotFound);
            }
        };
        match class.name.as_str() {
            "CCSPlayerController" => return Ok(EntityType::PlayerController),
            "CCSGameRulesProxy" => return Ok(EntityType::Rules),
            "CCSTeam" => return Ok(EntityType::Team),
            "CC4" => return Ok(EntityType::C4),
            _ => {}
        }
        if class.name.contains("Projectile") || class.name == "CIncendiaryGrenade" {
            return Ok(EntityType::Projectile);
        }
        return Ok(EntityType::Normal);
    }
}
