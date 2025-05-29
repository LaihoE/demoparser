use super::sendtables::Serializer;
use super::stringtables::StringTable;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::prop_controller::PropInfo;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::stringtables::UserInfo;
use crate::maps::FRIENDLY_NAMES_MAPPING;
use crate::maps::NON_MULTITHREADABLE_PROPS;
use crate::second_pass::decoder::QfMapper;
use crate::second_pass::other_netmessages::Class;
use crate::second_pass::parser_settings::PlayerEndMetaData;
use crate::second_pass::parser_settings::SpecialIDs;
use crate::second_pass::variants::Variant;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::RandomState;
use csgoproto::csvc_msg_game_event_list::DescriptorT;
use csgoproto::CDemoSendTables;
use memmap2::Mmap;
use memmap2::MmapOptions;
use std::collections::BTreeMap;
use std::fs::File;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ParserInputs<'a> {
    pub real_name_to_og_name: AHashMap<String, String>,
    pub wanted_players: Vec<u64>,
    pub wanted_player_props: Vec<String>,
    pub wanted_other_props: Vec<String>,
    pub wanted_prop_states: AHashMap<String, Variant>,
    pub wanted_ticks: Vec<i32>,
    pub wanted_events: Vec<String>,
    pub parse_ents: bool,
    pub parse_projectiles: bool,
    pub parse_grenades: bool,
    pub only_header: bool,
    pub only_convars: bool,
    pub huffman_lookup_table: &'a Vec<(u8, u8)>,
    pub order_by_steamid: bool,
    pub list_props: bool,
    pub fallback_bytes: Option<Vec<u8>>,
}

pub struct FirstPassParser<'a> {
    pub added_temp_props: Vec<String>,
    pub real_name_to_og_name: AHashMap<String, String>,
    pub fullpacket_offsets: Vec<usize>,
    pub ptr: usize,
    pub tick: i32,
    pub huf: &'a Vec<(u8, u8)>,
    pub settings: &'a ParserInputs<'a>,
    pub serializers: AHashMap<String, Serializer>,
    pub cls_by_id: Option<Arc<Vec<Class>>>,
    pub string_tables: Vec<StringTable>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub convars: AHashMap<String, String>,
    pub player_md: Vec<PlayerEndMetaData>,
    pub prop_controller: PropController,
    pub ge_list: AHashMap<i32, DescriptorT>,
    pub qf_mapper: QfMapper,
    pub stringtable_players: BTreeMap<i32, UserInfo>,
    pub qf_map_set: bool,
    pub ge_list_set: bool,
    pub cls_by_id_set: bool,
    pub wanted_player_props: Vec<String>,
    pub wanted_players: AHashSet<u64, RandomState>,
    pub wanted_ticks: AHashSet<i32, RandomState>,
    pub wanted_other_props: Vec<String>,
    pub wanted_prop_states: AHashMap<String, Variant>,
    pub wanted_events: Vec<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,
    pub parse_grenades: bool,
    pub name_to_id: AHashMap<String, u32>,
    pub id: u32,
    pub wanted_prop_ids: Vec<u32>,
    pub controller_ids: SpecialIDs,
    pub only_header: bool,
    pub prop_infos: Vec<PropInfo>,
    pub header: AHashMap<String, String>,
    pub is_multithreadable: bool,
    pub needs_velocity: bool,
    pub sendtable_message: Option<CDemoSendTables>,
    pub order_by_steamid: bool,
    pub list_props: bool,
    pub fallback_bytes: Option<&'a [u8]>,
}
pub fn needs_velocity(props: &[String]) -> bool {
    for prop in props {
        if prop.contains("velo") {
            return true;
        }
    }
    false
}

impl<'a> FirstPassParser<'a> {
    pub fn new(inputs: &'a ParserInputs<'a>) -> Self {
        FirstPassParser {
            fallback_bytes: inputs.fallback_bytes.as_deref(),
            order_by_steamid: inputs.order_by_steamid,
            sendtable_message: None,
            needs_velocity: needs_velocity(&inputs.wanted_player_props),
            added_temp_props: vec![],
            is_multithreadable: check_multithreadability(&inputs.wanted_player_props),
            stringtable_players: BTreeMap::default(),
            only_header: inputs.only_header,
            ge_list_set: false,
            cls_by_id_set: false,
            qf_map_set: false,
            real_name_to_og_name: inputs.real_name_to_og_name.clone(),
            prop_controller: PropController::new(
                inputs.wanted_player_props.clone(),
                inputs.wanted_other_props.clone(),
                inputs.wanted_prop_states.clone(),
                inputs.real_name_to_og_name.clone(),
                false,
                &vec!["None".to_string()],
                inputs.parse_projectiles,
            ),
            cls_by_id: None,
            player_md: vec![],
            name_to_id: AHashMap::default(),
            convars: AHashMap::default(),
            string_tables: vec![],
            fullpacket_offsets: vec![],
            ptr: 0,
            baselines: AHashMap::default(),
            tick: 0,
            huf: &inputs.huffman_lookup_table,
            qf_mapper: QfMapper {
                idx: 0,
                map: AHashMap::default(),
            },
            ge_list: AHashMap::default(),
            parse_entities: true,
            serializers: AHashMap::default(),
            parse_projectiles: inputs.parse_projectiles,
            parse_grenades: inputs.parse_grenades,
            wanted_player_props: inputs.wanted_player_props.clone(),
            wanted_events: inputs.wanted_events.clone(),
            wanted_players: AHashSet::from_iter(inputs.wanted_players.iter().cloned()),
            wanted_ticks: AHashSet::from_iter(inputs.wanted_ticks.iter().cloned()),
            wanted_other_props: inputs.wanted_other_props.clone(),
            wanted_prop_states: inputs.wanted_prop_states.clone(),
            settings: &inputs,
            controller_ids: SpecialIDs::new(),
            id: 0,
            wanted_prop_ids: vec![],
            prop_infos: vec![],
            header: AHashMap::default(),
            list_props: inputs.list_props,
        }
    }
}
pub fn check_multithreadability(player_props: &[String]) -> bool {
    for name in player_props {
        if NON_MULTITHREADABLE_PROPS.contains(name) {
            return false;
        }
    }
    true
}

pub fn rm_user_friendly_names(names: &Vec<String>) -> Result<Vec<String>, DemoParserError> {
    let mut real_names = vec![];
    for name in names {
        let mut n = if name.starts_with("Weapon.") { name.split_at(7).1 } else { name };
        n = if name.starts_with("Grenade.") { name.split_at(8).1 } else { n };
        match FRIENDLY_NAMES_MAPPING.get(n) {
            Some(real_name) => real_names.push(real_name.to_string()),
            None => real_names.push(n.to_string()),
        }
    }
    Ok(real_names)
}

pub fn rm_map_user_friendly_names(map: &AHashMap<String, Variant>) -> Result<AHashMap<String, Variant>, DemoParserError> {
    let mut real_names_map: AHashMap<String, Variant> = AHashMap::default();
    for (name, variant) in map {
        let n = if name.starts_with("Weapon.") { name.split_at(7).1 } else { name };
        match FRIENDLY_NAMES_MAPPING.get(&n) {
            Some(real_name) => real_names_map.insert(real_name.to_string(), variant.clone()),
            None => real_names_map.insert(n.to_string(), variant.clone()),
        };
    }
    Ok(real_names_map)
}

pub fn create_mmap(path: String) -> Result<Mmap, DemoParserError> {
    let file = match File::open(path) {
        Err(e) => return Err(DemoParserError::FileNotFound(format!("{}", e))),
        Ok(f) => f,
    };
    let mmap = unsafe {
        match MmapOptions::new().map(&file) {
            Err(e) => return Err(DemoParserError::FileNotFound(format!("{}", e))),
            Ok(f) => f,
        }
    };
    Ok(mmap)
}
