use super::game_events::GameEvent;
use super::read_bits::DemoParserError;
use super::sendtables::Serializer;
use super::stringtables::StringTable;
use super::variants::PropColumn;
use crate::collect_data::ProjectileRecord;
use crate::decoder::QfMapper;
use crate::entities::Entity;
use crate::entities::PlayerMetaData;
use crate::other_netmessages::Class;
use crate::parser::DemoOutput;
use crate::parser::ParserThreadInput;
use crate::prop_controller::PropController;
use crate::sendtables::DebugField;
use crate::sendtables::DebugFieldAndPath;
use crate::sendtables::FieldInfo;
use crate::sendtables::FieldModel;
use crate::stringtables::UserInfo;
use crate::variants::BytesVariant;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::HashMap;
use ahash::RandomState;
use bit_reverse::LookupReverse;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::sync::Arc;

// Wont fit in L1, evaluate if worth to use pointer method
const HUF_LOOKUPTABLE_MAXVALUE: u32 = (1 << 17) - 1;

pub struct ParserThread {
    pub qf_mapper: Arc<QfMapper>,
    pub prop_controller: Arc<PropController>,
    pub cls_by_id: Arc<AHashMap<u32, Class>>,
    pub stringtable_players: AHashMap<u64, UserInfo>,
    pub net_tick: u32,

    pub ptr: usize,
    pub bytes: Arc<BytesVariant>,
    pub parse_all_packets: bool,
    // Parsing state
    pub ge_list: Arc<AHashMap<i32, Descriptor_t>>,
    pub serializers: AHashMap<String, Serializer, RandomState>,
    pub cls_bits: Option<u32>,
    pub entities: AHashMap<i32, Entity, RandomState>,
    pub tick: i32,
    pub players: BTreeMap<i32, PlayerMetaData>,
    pub teams: Teams,
    pub huffman_lookup_table: Arc<Vec<(u32, u8)>>,
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<StringTable>,
    pub rules_entity_id: Option<i32>,
    pub game_events_counter: AHashSet<String>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub field_infos: Vec<FieldInfo>,
    pub projectiles: BTreeSet<i32>,
    pub fullpackets_parsed: u32,
    pub packets_parsed: u32,
    pub cnt: AHashMap<FieldModel, u32>,
    pub projectile_records: Vec<ProjectileRecord>,
    pub wanted_ticks: AHashSet<i32>,

    // Output from parsing
    pub output: AHashMap<u32, PropColumn, RandomState>,
    pub header: HashMap<String, String>,
    pub skins: Vec<EconItem>,
    pub item_drops: Vec<EconItem>,
    pub convars: AHashMap<String, String>,
    pub chat_messages: Vec<ChatMessageRecord>,
    pub player_end_data: Vec<PlayerEndMetaData>,
    // pub projectile_records: ProjectileRecordVec,

    // Settings
    pub wanted_events: Vec<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,
    pub debug_fields: Vec<DebugFieldAndPath>,
    pub is_debug_mode: bool,
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

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageRecord {
    pub entity_idx: Option<i32>,
    pub param1: Option<String>,
    pub param2: Option<String>,
    pub param3: Option<String>,
    pub param4: Option<String>,
}
#[derive(Debug, Clone)]
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

    // Custom
    pub item_name: Option<String>,
    pub skin_name: Option<String>,
}
#[derive(Debug, Clone)]
pub struct PlayerEndMetaData {
    pub steamid: Option<u64>,
    pub name: Option<String>,
    pub team_number: Option<i32>,
}

impl ParserThread {
    pub fn create_output(self) -> DemoOutput {
        DemoOutput {
            chat_messages: self.chat_messages,
            convars: self.convars,
            df: self.output,
            game_events: self.game_events,
            skins: self.skins,
            item_drops: self.item_drops,
            header: None,
            player_md: self.player_end_data,
            game_events_counter: self.game_events_counter,
            prop_info: PropController::new(vec![], vec![], AHashMap::default()),
            projectiles: self.projectile_records,
            ptr: self.ptr,
        }
    }
    pub fn new(input: ParserThreadInput) -> Result<Self, DemoParserError> {
        input
            .settings
            .wanted_player_props
            .clone()
            .extend(vec!["tick".to_owned(), "steamid".to_owned(), "name".to_owned()]);
        let args: Vec<String> = env::args().collect();
        let debug = if args.len() > 2 { args[2] == "true" } else { false };

        // Dont allocate vec in release mode
        let debug_vec_len = match debug {
            true => 8192,
            false => 0,
        };
        Ok(ParserThread {
            net_tick: 0,
            debug_fields: vec![
                DebugFieldAndPath {
                    field: DebugField {
                        decoder: crate::sendtables::Decoder::NoscaleDecoder,
                        full_name: "".to_string(),
                        field: None,
                    },
                    path: [0, 0, 0, 0, 0, 0, 0],
                };
                debug_vec_len
            ],
            stringtable_players: input.stringtable_players,
            is_debug_mode: debug,
            projectile_records: vec![],
            parse_all_packets: input.parse_all_packets,
            wanted_ticks: input.wanted_ticks.clone(),
            prop_controller: Arc::new(input.prop_controller),
            qf_mapper: input.qfmap,
            fullpackets_parsed: 0,
            packets_parsed: 0,
            cnt: AHashMap::default(),
            serializers: AHashMap::default(),
            ptr: input.offset,
            ge_list: input.ge_list.clone(),
            bytes: input.settings.bytes.clone(),
            cls_by_id: input.cls_by_id,
            entities: AHashMap::with_capacity(512),
            cls_bits: None,
            tick: -99999,
            players: BTreeMap::default(),
            output: AHashMap::default(),
            game_events: vec![],
            wanted_events: input.settings.wanted_events.clone(),
            parse_entities: input.settings.parse_ents,
            projectiles: BTreeSet::default(),
            // projectile_records: ProjectileRecordVec::new(),
            baselines: input.baselines.clone(),
            string_tables: input.string_tables.clone(),
            field_infos: vec![
                FieldInfo {
                    decoder: crate::sendtables::Decoder::NoscaleDecoder,
                    should_parse: false,
                    prop_id: 0,
                    controller_prop: None,
                };
                8192
            ],
            teams: Teams::new(),
            game_events_counter: AHashSet::default(),
            parse_projectiles: input.settings.parse_projectiles,
            rules_entity_id: None,
            convars: AHashMap::default(),
            chat_messages: vec![],
            item_drops: vec![],
            skins: vec![],
            player_end_data: vec![],
            huffman_lookup_table: input.settings.huffman_lookup_table.clone(),
            header: HashMap::default(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SpecialIDs {
    pub teamnum: Option<u32>,
    pub player_name: Option<u32>,
    pub steamid: Option<u32>,
    pub player_pawn: Option<u32>,

    pub player_team_pointer: Option<u32>,
    pub weapon_owner_pointer: Option<u32>,
    pub team_team_num: Option<u32>,

    pub cell_x_player: Option<u32>,
    pub cell_y_player: Option<u32>,
    pub cell_z_player: Option<u32>,

    pub cell_x_offset_player: Option<u32>,
    pub cell_y_offset_player: Option<u32>,
    pub cell_z_offset_player: Option<u32>,
    pub active_weapon: Option<u32>,
    pub item_def: Option<u32>,

    pub m_vec_x_grenade: Option<u32>,
    pub m_vec_y_grenade: Option<u32>,
    pub m_vec_z_grenade: Option<u32>,

    pub m_cell_x_grenade: Option<u32>,
    pub m_cell_y_grenade: Option<u32>,
    pub m_cell_z_grenade: Option<u32>,

    pub grenade_owner_id: Option<u32>,
    pub buttons: Option<u32>,
    pub eye_angles: Option<u32>,

    pub orig_own_low: Option<u32>,
    pub orig_own_high: Option<u32>,
}
impl SpecialIDs {
    pub fn new() -> Self {
        SpecialIDs {
            teamnum: None,
            player_name: None,
            steamid: None,
            player_pawn: None,
            player_team_pointer: None,
            weapon_owner_pointer: None,
            team_team_num: None,
            cell_x_player: None,
            cell_y_player: None,
            cell_z_player: None,
            cell_x_offset_player: None,
            cell_y_offset_player: None,
            cell_z_offset_player: None,
            active_weapon: None,
            item_def: None,
            m_cell_x_grenade: None,
            m_cell_y_grenade: None,
            m_cell_z_grenade: None,
            m_vec_x_grenade: None,
            m_vec_y_grenade: None,
            m_vec_z_grenade: None,
            grenade_owner_id: None,
            buttons: None,
            eye_angles: None,
            orig_own_high: None,
            orig_own_low: None,
        }
    }
}

fn msb(mut val: u32) -> u32 {
    let mut cnt = 0;
    while val > 0 {
        cnt = cnt + 1;
        val = val >> 1;
    }
    cnt
}

pub fn create_huffman_lookup_table() -> Vec<(u32, u8)> {
    let mut huffman_table = vec![(999999, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];
    let mut huffman_rev_table = vec![(999999, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];

    huffman_table[0] = (0, 1);
    huffman_table[2] = (39, 2);
    huffman_table[24] = (8, 5);
    huffman_table[50] = (2, 6);
    huffman_table[51] = (29, 6);
    huffman_table[100] = (2, 6);
    huffman_table[101] = (29, 6);
    huffman_table[26] = (4, 5);
    huffman_table[432] = (30, 9);
    huffman_table[866] = (38, 10);
    huffman_table[55488] = (35, 16);
    huffman_table[55489] = (34, 16);
    huffman_table[27745] = (27, 15);
    huffman_table[55492] = (25, 16);
    huffman_table[55493] = (24, 16);
    huffman_table[55494] = (33, 16);
    huffman_table[55495] = (28, 16);
    huffman_table[55496] = (13, 16);
    huffman_table[110994] = (15, 17);
    huffman_table[110995] = (14, 17);
    huffman_table[27749] = (6, 15);
    huffman_table[111000] = (21, 17);
    huffman_table[111001] = (20, 17);
    huffman_table[111002] = (23, 17);
    huffman_table[111003] = (22, 17);
    huffman_table[111004] = (17, 17);
    huffman_table[111005] = (16, 17);
    huffman_table[111006] = (19, 17);
    huffman_table[111007] = (18, 17);
    huffman_table[3469] = (5, 12);
    huffman_table[1735] = (36, 11);
    huffman_table[217] = (10, 8);
    huffman_table[218] = (7, 8);
    huffman_table[438] = (12, 9);
    huffman_table[439] = (37, 9);
    huffman_table[220] = (9, 8);
    huffman_table[442] = (31, 9);
    huffman_table[443] = (26, 9);
    huffman_table[222] = (32, 8);
    huffman_table[223] = (3, 8);
    huffman_table[14] = (1, 4);
    huffman_table[15] = (11, 4);
    huffman_table[0] = (999999, 255);

    const RIGHTSHIFT_BITORDER: u32 = 64 - 17;
    let mut v: Vec<u32> = vec![];
    for (idx, x) in huffman_table.iter().enumerate() {
        if x.0 != 999999 {
            v.push(idx as u32);
        }
    }
    let mut idx_msb_map = Vec::with_capacity(HUF_LOOKUPTABLE_MAXVALUE as usize);
    for i in 0..HUF_LOOKUPTABLE_MAXVALUE {
        idx_msb_map.push(msb(i));
    }
    for x in v {
        let shifta = msb(x);
        for (idx, pair) in idx_msb_map.iter().enumerate() {
            if x == idx as u32 >> pair - shifta {
                let peekbits = (idx as u64).swap_bits() >> RIGHTSHIFT_BITORDER;
                huffman_table[idx as usize] = huffman_table[x as usize];
                huffman_rev_table[peekbits as usize] = huffman_table[x as usize];
            }
        }
    }
    for v in 0..HUF_LOOKUPTABLE_MAXVALUE {
        let p: u64 = (v as u64).swap_bits() >> RIGHTSHIFT_BITORDER;
        if p & 1 == 0 {
            huffman_rev_table[p as usize] = (0, 1);
        }
    }
    huffman_rev_table
}
