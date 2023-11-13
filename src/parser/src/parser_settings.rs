use super::sendtables::Serializer;
use super::stringtables::StringTable;
use crate::decoder::QfMapper;
use crate::maps::FRIENDLY_NAMES_MAPPING;
use crate::maps::NON_MULTITHREADABLE_PROPS;
use crate::other_netmessages::Class;
use crate::parser_thread_settings::PlayerEndMetaData;
use crate::parser_thread_settings::SpecialIDs;
use crate::prop_controller::PropController;
use crate::prop_controller::PropInfo;
use crate::read_bits::DemoParserError;
use crate::stringtables::UserInfo;
use crate::variants::BytesVariant;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use memmap2::Mmap;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ParserInputs {
    pub bytes: Arc<BytesVariant>,
    pub real_name_to_og_name: AHashMap<String, String>,

    pub wanted_player_props: Vec<String>,
    pub wanted_player_props_og_names: Vec<String>,
    pub wanted_other_props: Vec<String>,
    pub wanted_other_props_og_names: Vec<String>,

    pub wanted_ticks: Vec<i32>,
    pub wanted_events: Vec<String>,
    pub parse_ents: bool,
    pub parse_projectiles: bool,
    pub only_header: bool,
    pub count_props: bool,
    pub only_convars: bool,
    pub huffman_lookup_table: Arc<Vec<(u32, u8)>>,
}

pub struct Parser {
    pub added_temp_props: Vec<String>,
    pub real_name_to_og_name: AHashMap<String, String>,
    pub fullpacket_offsets: Vec<usize>,
    pub ptr: usize,
    pub bytes: Arc<BytesVariant>,
    pub tick: i32,
    pub huf: Arc<Vec<(u32, u8)>>,
    pub settings: ParserInputs,
    pub serializers: AHashMap<String, Serializer>,
    pub cls_by_id: Option<Arc<AHashMap<u32, Class>>>,
    pub string_tables: Vec<StringTable>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub convars: AHashMap<String, String>,
    pub player_md: Vec<PlayerEndMetaData>,
    pub maps_ready: bool,
    pub prop_controller: PropController,
    pub prop_controller_is_set: bool,
    pub ge_list: AHashMap<i32, Descriptor_t>,
    pub qf_mapper: QfMapper,
    pub stringtable_players: BTreeMap<u64, UserInfo>,

    pub qf_map_set: bool,
    pub ge_list_set: bool,
    pub cls_by_id_set: bool,

    pub wanted_player_props: Vec<String>,

    pub wanted_ticks: AHashSet<i32, RandomState>,
    pub wanted_player_props_og_names: Vec<String>,
    // Team and rules props
    pub wanted_other_props: Vec<String>,
    pub wanted_other_props_og_names: Vec<String>,
    pub wanted_events: Vec<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,
    pub name_to_id: AHashMap<String, u32>,

    pub id: u32,
    pub wanted_prop_ids: Vec<u32>,
    pub controller_ids: SpecialIDs,
    pub player_output_ids: Vec<u8>,
    pub prop_out_id: u8,
    pub only_header: bool,
    pub prop_infos: Vec<PropInfo>,
    pub largest_wanted_tick: i32,

    pub header: AHashMap<String, String>,
    pub threads_spawned: u32,
    pub is_multithreadable: bool,
}
pub fn needs_velocity(props: &[String]) -> bool {
    for prop in props {
        if prop.contains("velo") {
            return true;
        }
    }
    false
}

impl Parser {
    pub fn new(mut inputs: ParserInputs) -> Self {
        let arc_bytes = inputs.bytes.clone();
        let arc_huf = inputs.huffman_lookup_table.clone();

        let mut added_temp_props = vec![];
        if needs_velocity(&inputs.wanted_player_props) {
            inputs
                .wanted_player_props
                .extend(vec!["X".to_string(), "Y".to_string(), "Z".to_string()]);
            added_temp_props.extend(vec!["X".to_string(), "Y".to_string(), "Z".to_string()]);
        }

        Parser {
            added_temp_props: added_temp_props,
            threads_spawned: 0,
            is_multithreadable: check_multithreadability(&inputs.wanted_player_props),
            largest_wanted_tick: *inputs.wanted_ticks.iter().max().unwrap_or(&999999999),
            stringtable_players: BTreeMap::default(),
            only_header: inputs.only_header,
            ge_list_set: false,
            cls_by_id_set: false,
            qf_map_set: false,
            real_name_to_og_name: inputs.real_name_to_og_name.clone(),
            prop_controller: PropController::new(
                inputs.wanted_player_props.clone(),
                inputs.wanted_other_props.clone(),
                inputs.real_name_to_og_name.clone(),
            ),
            prop_controller_is_set: false,
            cls_by_id: None,
            player_md: vec![],
            maps_ready: false,
            name_to_id: AHashMap::default(),
            convars: AHashMap::default(),
            bytes: arc_bytes.clone(),
            string_tables: vec![],
            fullpacket_offsets: vec![],
            ptr: 0,
            baselines: AHashMap::default(),
            tick: 0,
            huf: arc_huf,
            qf_mapper: QfMapper {
                idx: 0,
                map: AHashMap::default(),
            },
            ge_list: AHashMap::default(),
            parse_entities: true,
            serializers: AHashMap::default(),
            parse_projectiles: false,
            wanted_player_props: inputs.wanted_player_props.clone(),
            wanted_events: inputs.wanted_events.clone(),
            wanted_ticks: AHashSet::from_iter(inputs.wanted_ticks.iter().cloned()),
            wanted_other_props: inputs.wanted_other_props.clone(),
            wanted_other_props_og_names: inputs.wanted_other_props_og_names.clone(),
            settings: inputs,
            wanted_player_props_og_names: vec![],
            controller_ids: SpecialIDs::new(),
            id: 0,
            player_output_ids: vec![],
            wanted_prop_ids: vec![],
            prop_out_id: 0,
            prop_infos: vec![],
            header: AHashMap::default(),
        }
    }
}
fn check_multithreadability(player_props: &[String]) -> bool {
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
        match FRIENDLY_NAMES_MAPPING.get(name) {
            Some(real_name) => real_names.push(real_name.to_string()),
            None => return Err(DemoParserError::UnknownPropName(name.clone())),
        }
    }
    Ok(real_names)
}
use memmap2::MmapOptions;
use std::fs::File;

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
