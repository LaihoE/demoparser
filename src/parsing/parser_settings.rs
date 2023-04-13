use super::class::Class;
use super::entities_utils::FieldPath;
use super::entities_utils::HuffmanNode;
use super::game_events::GameEvent;
use super::sendtables::Serializer;
use super::stringtables::StringTable;
use super::variants::PropColumn;
use crate::parsing::collect_data::ProjectileRecordVec;
use crate::parsing::entities::Entity;
use crate::parsing::entities::PlayerMetaData;
use crate::parsing::entities_utils::generate_huffman_tree;
use crate::parsing::sendtables::Decoder;
use ahash::HashMap;
use ahash::HashSet;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use smallvec::smallvec;
use soa_derive::StructOfArray;
use std::fs;

pub struct Parser {
    // todo split into smaller parts
    pub ptr: usize,
    pub bytes: Vec<u8>,
    pub ge_list: Option<HashMap<i32, Descriptor_t>>,
    pub serializers: HashMap<String, Serializer>,
    pub cls_by_id: HashMap<u32, Class>,
    pub cls_by_name: HashMap<String, Class>,
    pub cls_bits: Option<u32>,
    pub entities: HashMap<i32, Entity>,
    pub tick: i32,
    pub huffman_tree: HuffmanNode,
    pub wanted_ticks: HashSet<i32>,
    pub wanted_props: Vec<String>,
    pub wanted_event: Option<String>,
    pub players: HashMap<i32, PlayerMetaData>,
    pub output: HashMap<String, PropColumn>,
    pub game_events: Vec<GameEvent>,
    pub parse_entities: bool,
    pub projectiles: HashSet<i32>,
    pub projectile_records: ProjectileRecordVec,
    pub pattern_cache: HashMap<u64, Decoder>,
    pub baselines: HashMap<u32, Vec<u8>>,
    pub string_tables: Vec<StringTable>,
    pub cache: HashMap<u128, (String, Decoder)>,
    pub paths: Vec<FieldPath>,
    pub teams: Teams,
    pub game_events_counter: HashMap<String, i32>,
    pub props_counter: HashMap<String, i32>,
    pub parse_projectiles: bool,
    pub count_props: bool,
    pub rules_entity_id: Option<i32>,
    pub uniq_message_ids: HashSet<u32>,
    pub convars: HashMap<String, String>,
    pub only_convars: bool,
    pub chat_messages: ChatMessageRecordVec,
    pub item_drops: EconItemVec,
    pub player_end_data: PlayerEndDataVec,
    pub skins: EconItemVec,
}
#[derive(Debug, Clone)]
pub struct Teams {
    pub team1_entid: Option<i32>,
    pub team2_entid: Option<i32>,
    pub team3_entid: Option<i32>,
}
impl Teams {
    pub fn new() -> Self {
        Teams {
            team1_entid: None,
            team2_entid: None,
            team3_entid: None,
        }
    }
}

#[derive(Debug, StructOfArray)]
pub struct ChatMessageRecord {
    pub entity_idx: Option<i32>,
    pub param1: Option<String>,
    pub param2: Option<String>,
    pub param3: Option<String>,
    pub param4: Option<String>,
}
#[derive(Debug, StructOfArray)]
pub struct EconItem {
    pub account_id: Option<u32>,
    pub item_id: Option<u64>,
    pub def_index: Option<u32>,
    pub paint_index: Option<u32>,
    pub rarity: Option<u32>,
    pub quality: Option<u32>,
    pub paint_wear: Option<u32>,
    pub paint_seed: Option<u32>,
    pub quest_id: Option<u32>,
    pub dropreason: Option<u32>,
    pub custom_name: Option<String>,
    pub inventory: Option<u32>,
    pub ent_idx: Option<i32>,
    pub steamid: Option<u64>,
}
#[derive(Debug, StructOfArray)]
pub struct PlayerEndData {
    pub steamid: Option<u64>,
    pub name: Option<String>,
    pub team_number: Option<i32>,
}

#[derive(Debug)]
pub struct ParserInputs {
    pub path: String,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: Vec<i32>,
    pub wanted_event: Option<String>,
    pub parse_ents: bool,
    pub parse_projectiles: bool,
    pub only_header: bool,
    pub count_props: bool,
    pub only_convars: bool,
}

impl Parser {
    pub fn new(mut settings: ParserInputs) -> Self {
        let bytes = fs::read(settings.path).unwrap();
        let tree = generate_huffman_tree().unwrap();
        let fp_filler = FieldPath {
            last: 0,
            path: smallvec![],
            done: false,
        };
        settings.wanted_props.extend(vec![
            "tick".to_owned(),
            "steamid".to_owned(),
            "name".to_owned(),
        ]);
        Parser {
            serializers: HashMap::default(),
            ptr: 0,
            ge_list: None,
            bytes: bytes,
            cls_by_id: HashMap::default(),
            cls_by_name: HashMap::default(),
            entities: HashMap::default(),
            cls_bits: None,
            tick: -99999,
            huffman_tree: tree,
            wanted_props: settings.wanted_props,
            players: HashMap::default(),
            output: HashMap::default(),
            wanted_ticks: HashSet::from_iter(settings.wanted_ticks),
            game_events: vec![],
            wanted_event: settings.wanted_event,
            parse_entities: settings.parse_ents,
            projectiles: HashSet::default(),
            projectile_records: ProjectileRecordVec::new(),
            pattern_cache: HashMap::default(),
            baselines: HashMap::default(),
            string_tables: vec![],
            cache: HashMap::default(),
            paths: vec![fp_filler; 10000],
            teams: Teams::new(),
            game_events_counter: HashMap::default(),
            props_counter: HashMap::default(),
            parse_projectiles: settings.parse_projectiles,
            count_props: settings.count_props,
            rules_entity_id: None,
            uniq_message_ids: HashSet::default(),
            convars: HashMap::default(),
            only_convars: settings.only_convars,
            chat_messages: ChatMessageRecordVec::new(),
            item_drops: EconItemVec::new(),
            skins: EconItemVec::new(),
            player_end_data: PlayerEndDataVec::new(),
        }
    }
}
