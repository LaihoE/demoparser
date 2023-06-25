use super::entities_utils::FieldPath;
use super::game_events::GameEvent;
use super::read_bits::DemoParserError;
use super::sendtables::Serializer;
use super::stringtables::StringTable;
use super::variants::PropColumn;
use crate::decoder::QfMapper;
use crate::entities::Entity;
use crate::entities::PlayerMetaData;
use crate::other_netmessages::Class;
use crate::parser_thread_settings::ParserInputs;
use crate::parser_thread_settings::ParserThread;
use crate::parser_thread_settings::PlayerEndMetaData;
use crate::parser_thread_settings::SpecialIDs;
use crate::sendtables::PropController;
use crate::sendtables::PropInfo;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use memmap2::Mmap;
use std::sync::Arc;
use std::time::Instant;

pub struct Parser {
    pub real_name_to_og_name: AHashMap<String, String>,
    pub fullpacket_offsets: Vec<usize>,
    pub ptr: usize,
    pub bytes: Arc<Mmap>,
    pub tick: i32,
    pub huf: Arc<Vec<(u32, u8)>>,
    pub settings: ParserInputs,
    pub serializers: AHashMap<String, Serializer>,
    pub cls_by_id: AHashMap<u32, Class>,
    pub ge_list: Option<AHashMap<i32, Descriptor_t>>,
    pub string_tables: Vec<StringTable>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub convars: AHashMap<String, String>,
    pub player_md: Vec<PlayerEndMetaData>,
    pub maps_ready: bool,
    pub start: Instant,
    pub prop_controller: Arc<PropController>,
    pub prop_controller_is_set: bool,

    pub wanted_player_props: Vec<String>,

    pub wanted_ticks: AHashSet<i32, RandomState>,
    pub wanted_player_props_og_names: Vec<String>,
    // Team and rules props
    pub wanted_other_props: Vec<String>,
    pub wanted_other_props_og_names: Vec<String>,
    pub wanted_event: Option<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,

    pub prop_name_to_path: AHashMap<String, [i32; 7]>,
    pub path_to_prop_name: AHashMap<[i32; 7], String>,
    pub wanted_prop_paths: AHashSet<[i32; 7]>,
    pub name_to_id: AHashMap<String, u32>,

    pub qf_mapper: QfMapper,

    pub id: u32,
    pub wanted_prop_ids: Vec<u32>,
    pub controller_ids: SpecialIDs,
    pub player_output_ids: Vec<u8>,
    pub prop_out_id: u8,
    pub id_to_path: AHashMap<u32, [i32; 7]>,
    pub prop_infos: Vec<PropInfo>,

    pub header: AHashMap<String, String>,
}

impl Parser {
    pub fn new(inputs: ParserInputs) -> Self {
        let arc_bytes = inputs.bytes.clone();
        let arc_huf = inputs.huffman_lookup_table.clone();
        Parser {
            real_name_to_og_name: inputs.real_name_to_og_name.clone(),
            prop_controller: Arc::new(PropController::new(vec![], vec![])),
            prop_controller_is_set: false,
            start: Instant::now(),
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
            huf: arc_huf.clone(),
            qf_mapper: QfMapper {
                idx: 0,
                map: AHashMap::default(),
            },
            ge_list: None,
            parse_entities: true,
            serializers: AHashMap::default(),
            parse_projectiles: false,
            wanted_player_props: inputs.wanted_player_props.clone(),
            wanted_event: inputs.wanted_event.clone(),
            settings: inputs,
            wanted_ticks: AHashSet::default(),
            wanted_prop_paths: AHashSet::default(),
            path_to_prop_name: AHashMap::default(),
            prop_name_to_path: AHashMap::default(),
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            cls_by_id: AHashMap::default(),
            controller_ids: SpecialIDs {
                teamnum: None,
                player_name: None,
                steamid: None,
                player_pawn: None,
                player_team_pointer: None,
                weapon_owner_pointer: None,
                cell_x_offset_player: None,
                cell_x_player: None,
                cell_y_offset_player: None,
                cell_y_player: None,
                cell_z_offset_player: None,
                cell_z_player: None,
                team_team_num: None,
                active_weapon: None,
                item_def: None,
            },
            id_to_path: AHashMap::default(),
            id: 0,
            player_output_ids: vec![],
            wanted_prop_ids: vec![],
            prop_out_id: 0,
            prop_infos: vec![],
            header: AHashMap::default(),
        }
    }
}
