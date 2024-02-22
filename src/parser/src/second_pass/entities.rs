use crate::first_pass::prop_controller::PropController;
use crate::first_pass::prop_controller::MY_WEAPONS_OFFSET;
use crate::first_pass::prop_controller::PLAYER_ENTITY_HANDLE_MISSING;
use crate::first_pass::prop_controller::SPECTATOR_TEAM_NUM;
use crate::first_pass::prop_controller::WEAPON_SKIN_ID;
use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Field;
use crate::first_pass::sendtables::Serializer;
use crate::second_pass::decoder::Decoder;
use crate::second_pass::decoder::Decoder::UnsignedDecoder;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::path_ops::*;
use crate::second_pass::variants::Variant;
use ahash::AHashMap;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;

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
#[derive(Debug, Clone, Copy)]
pub struct FieldInfo {
    pub decoder: Decoder,
    pub should_parse: bool,
    pub prop_id: u32,
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
#[derive(Debug, Clone)]
pub struct RoundEnd {
    pub old_value: Option<Variant>,
    pub new_value: Option<Variant>,
}
#[derive(Debug, Clone)]
pub struct RoundWinReason {
    pub reason: i32,
}
#[derive(Debug, Clone)]
pub enum GameEventInfo {
    RoundEnd(RoundEnd),
    RoundWinReason(RoundWinReason),
    FreezePeriodStart(bool),
}

impl<'a> SecondPassParser<'a> {
    pub fn parse_packet_ents(&mut self, bytes: &[u8], is_fullpacket: bool) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
        let msg: CSVCMsg_PacketEntities = match Message::parse_from_bytes(bytes) {
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
                    self.create_new_entity(&mut bitreader, &entity_id)?;
                    self.update_entity(&mut bitreader, entity_id, false, &mut events_to_emit, is_fullpacket)?;
                }
                EntityCmd::Update => {
                    if msg.has_has_pvs_vis_bits() {
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
        let n_updated_values =
            self.decode_entity_update(bitreader, entity_id, n_updates, is_fullpacket, is_baseline, events_to_emit)?;
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
            let field = SecondPassParser::find_field(&path, &class.serializer)?;
            let field_info = SecondPassParser::get_propinfo(&field, path);
            let decoder = SecondPassParser::get_decoder_from_field(field)?;
            let result = bitreader.decode(&decoder, self.qf_mapper)?;

            if !is_fullpacket && !is_baseline {
                events_to_emit.extend(SecondPassParser::listen_for_events(
                    entity,
                    &result,
                    field,
                    field_info,
                    &self.prop_controller,
                ));
            }
            if self.is_debug_mode {
                SecondPassParser::debug_inspect(&result, field, self.tick);
            }
            SecondPassParser::insert_field(entity, result, field_info);
        }
        Ok(n_updates)
    }
    pub fn debug_inspect(_result: &Variant, field: &Field, tick: i32) {
        if let Field::Value(_v) = field {
            if _v.full_name.contains("Started") || _v.full_name.contains("Reason") {
                println!("{:?} {:?} {:?}", _v.full_name, _result, tick);
            }
        }
    }
    fn get_decoder_from_field(field: &Field) -> Result<Decoder, DemoParserError> {
        let decoder = match field {
            Field::Value(inner) => inner.decoder,
            Field::Vector(_) => UnsignedDecoder,
            Field::Pointer(inner) => inner.decoder,
            _ => return Err(DemoParserError::FieldNoDecoder),
        };
        Ok(decoder)
    }
    pub fn insert_field(entity: &mut Entity, result: Variant, field_info: Option<FieldInfo>) {
        if let Some(fi) = field_info {
            if fi.should_parse {
                entity.props.insert(fi.prop_id, result);
            }
        }
    }
    pub fn get_propinfo(field: &Field, path: &FieldPath) -> Option<FieldInfo> {
        let info = match field {
            Field::Value(v) => Some(FieldInfo {
                decoder: v.decoder,
                should_parse: v.should_parse,
                prop_id: v.prop_id,
            }),
            Field::Vector(v) => match field.get_inner(0) {
                Ok(Field::Value(inner)) => Some(FieldInfo {
                    decoder: v.decoder,
                    should_parse: inner.should_parse,
                    prop_id: inner.prop_id,
                }),
                _ => None,
            },
            _ => None,
        };
        if let Some(mut fi) = info {
            if fi.prop_id == MY_WEAPONS_OFFSET {
                if path.last == 1 {
                } else {
                    fi.prop_id = MY_WEAPONS_OFFSET + path.path[2] as u32 + 1;
                }
            }
            if fi.prop_id == WEAPON_SKIN_ID {
                if let Some(entry) = path.path.get(path.last - 1) {
                    if *entry != 0 {
                        // Fill with impossible id
                        fi.prop_id = u32::MAX;
                    }
                }
            }
            return Some(fi);
        }
        return None;
    }
    fn listen_for_events(
        entity: &mut Entity,
        result: &Variant,
        _field: &Field,
        field_info: Option<FieldInfo>,
        prop_controller: &PropController,
    ) -> Vec<GameEventInfo> {
        let mut events = vec![];

        if let Some(fi) = field_info {
            // Total rounds played
            if let Some(id) = prop_controller.special_ids.total_rounds_played {
                if fi.prop_id == id {
                    events.push(GameEventInfo::RoundEnd(RoundEnd {
                        old_value: entity.props.get(&id).cloned(),
                        new_value: Some(result.clone()),
                    }));
                }
            }
            // Round win reason
            if let Some(id) = prop_controller.special_ids.round_win_reason {
                if fi.prop_id == id {
                    if let Variant::I32(reason) = result {
                        events.push(GameEventInfo::RoundWinReason(RoundWinReason { reason: *reason }));
                    }
                }
            }
            // freeze period end
            if let Some(id) = prop_controller.special_ids.round_start_time {
                if fi.prop_id == id {
                    events.push(GameEventInfo::FreezePeriodStart(true));
                }
            }
        }
        events
    }

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
    #[inline(always)]
    fn find_field<'b>(fp: &FieldPath, ser: &'b Serializer) -> Result<&'b Field, DemoParserError> {
        let f = match ser.fields.get(fp.path[0] as usize) {
            Some(entry) => entry,
            None => return Err(DemoParserError::IllegalPathOp),
        };
        match fp.last {
            0 => Ok(f),
            1 => Ok(f.get_inner(fp.path[1] as usize)?),
            2 => Ok(f.get_inner(fp.path[1] as usize)?.get_inner(fp.path[2] as usize)?),
            3 => Ok(f
                .get_inner(fp.path[1] as usize)?
                .get_inner(fp.path[2] as usize)?
                .get_inner(fp.path[3] as usize)?),
            4 => Ok(f
                .get_inner(fp.path[1] as usize)?
                .get_inner(fp.path[2] as usize)?
                .get_inner(fp.path[3] as usize)?
                .get_inner(fp.path[4] as usize)?),
            5 => Ok(f
                .get_inner(fp.path[1] as usize)?
                .get_inner(fp.path[2] as usize)?
                .get_inner(fp.path[3] as usize)?
                .get_inner(fp.path[4] as usize)?
                .get_inner(fp.path[5] as usize)?),
            _ => return Err(DemoParserError::IllegalPathOp),
        }
    }

    pub fn gather_extra_info(&mut self, entity_id: &i32, is_baseline: bool) -> Result<(), DemoParserError> {
        // Boring stuff.. function does some bookkeeping
        let entity = match self.entities.get(*entity_id as usize) {
            Some(Some(entity)) => entity,
            _ => return Err(DemoParserError::EntityNotFound),
        };
        if !(entity.entity_type == EntityType::PlayerController || entity.entity_type == EntityType::Team) {
            return Ok(());
        }
        if entity.entity_type == EntityType::Team && !is_baseline {
            if let Some(team_num_id) = self.prop_controller.special_ids.team_team_num {
                if let Ok(Variant::U32(t)) = self.get_prop_from_ent(&team_num_id, entity_id) {
                    match t {
                        1 => self.teams.team1_entid = Some(*entity_id),
                        2 => self.teams.team2_entid = Some(*entity_id),
                        3 => self.teams.team3_entid = Some(*entity_id),
                        _ => {}
                    }
                }
            }
        }

        let team_num = match self.prop_controller.special_ids.teamnum {
            Some(team_num_id) => match self.get_prop_from_ent(&team_num_id, entity_id) {
                Ok(team_num) => match team_num {
                    Variant::U32(team_num) => Some(team_num),
                    _ => return Err(DemoParserError::IncorrectMetaDataProp),
                },
                Err(_) => None,
            },
            _ => None,
        };

        let name = match self.prop_controller.special_ids.player_name {
            Some(id) => match self.get_prop_from_ent(&id, entity_id) {
                Ok(team_num) => match team_num {
                    Variant::String(team_num) => Some(team_num),
                    _ => return Err(DemoParserError::IncorrectMetaDataProp),
                },
                Err(_) => None,
            },
            _ => None,
        };
        let steamid = match self.prop_controller.special_ids.steamid {
            Some(id) => match self.get_prop_from_ent(&id, entity_id) {
                Ok(team_num) => match team_num {
                    Variant::U64(team_num) => Some(team_num),
                    _ => return Err(DemoParserError::IncorrectMetaDataProp),
                },
                Err(_) => None,
            },
            _ => None,
        };
        let player_entid = match self.prop_controller.special_ids.player_pawn {
            Some(id) => match self.get_prop_from_ent(&id, entity_id) {
                Ok(player_entid) => match player_entid {
                    Variant::U32(handle) => Some((handle & 0x7FF) as i32),
                    _ => return Err(DemoParserError::IncorrectMetaDataProp),
                },
                Err(_) => None,
            },
            _ => None,
        };
        if let Some(e) = player_entid {
            if e != PLAYER_ENTITY_HANDLE_MISSING && steamid != Some(0) && team_num != Some(SPECTATOR_TEAM_NUM) {
                match self.should_remove(steamid) {
                    Some(eid) => {
                        self.players.remove(&eid);
                    }
                    None => {}
                }
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
        Ok(())
    }
    fn should_remove(&self, steamid: Option<u64>) -> Option<i32> {
        for (entid, player) in &self.players {
            if player.steamid == steamid {
                return Some(*entid);
            }
        }
        None
    }

    fn create_new_entity(&mut self, bitreader: &mut Bitreader, entity_id: &i32) -> Result<(), DemoParserError> {
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
        let entity = SecondPassParser::make_ent(entity_id, cls_id, entity_type);
        if self.entities.len() as i32 <= *entity_id {
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
    fn make_ent(entity_id: &i32, cls_id: u32, entity_type: EntityType) -> Entity {
        Entity {
            entity_id: *entity_id,
            cls_id: cls_id,
            props: AHashMap::with_capacity(0),
            entity_type: entity_type,
        }
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
#[inline(always)]
fn generate_fp() -> FieldPath {
    FieldPath {
        path: [-1, 0, 0, 0, 0, 0, 0],
        last: 0,
    }
}
