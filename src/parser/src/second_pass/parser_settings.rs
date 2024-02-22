use crate::first_pass::parser::FirstPassOutput;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Serializer;
use crate::first_pass::stringtables::StringTable;
use crate::first_pass::stringtables::UserInfo;
use crate::second_pass::collect_data::ProjectileRecord;
use crate::second_pass::decoder::QfMapper;
use crate::second_pass::entities::Entity;
use crate::second_pass::entities::PlayerMetaData;
use crate::second_pass::game_events::GameEvent;
use crate::second_pass::other_netmessages::Class;
use crate::second_pass::parser::SecondPassOutput;
use crate::second_pass::path_ops::FieldPath;
use crate::second_pass::variants::PropColumn;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::HashMap;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_VoiceData;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;

const HUF_LOOKUPTABLE_MAXVALUE: u32 = (1 << 17) - 1;
const DEFAULT_MAX_ENTITY_ID: usize = 1024;

pub struct SecondPassParser<'a> {
    pub qf_mapper: &'a QfMapper,
    pub prop_controller: &'a PropController,
    pub cls_by_id: &'a Vec<Class>,
    pub stringtable_players: BTreeMap<u64, UserInfo>,
    pub net_tick: u32,
    pub parse_inventory: bool,
    pub paths: Vec<FieldPath>,
    pub ptr: usize,
    pub parse_all_packets: bool,
    pub ge_list: &'a AHashMap<i32, Descriptor_t>,
    pub serializers: AHashMap<String, Serializer, RandomState>,
    pub cls_bits: Option<u32>,
    pub entities: Vec<Option<Entity>>,
    pub tick: i32,
    pub players: BTreeMap<i32, PlayerMetaData>,
    pub teams: Teams,
    pub huffman_lookup_table: &'a [(u8, u8)],
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<StringTable>,
    pub rules_entity_id: Option<i32>,
    pub c4_entity_id: Option<i32>,
    pub game_events_counter: AHashSet<String>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub projectiles: BTreeSet<i32>,
    pub fullpackets_parsed: u32,
    pub wanted_players: AHashSet<u64>,
    pub wanted_ticks: AHashSet<i32>,
    // Output from parsing
    pub projectile_records: Vec<ProjectileRecord>,
    pub voice_data: Vec<CSVCMsg_VoiceData>,
    pub output: AHashMap<u32, PropColumn, RandomState>,
    pub header: HashMap<String, String>,
    pub skins: Vec<EconItem>,
    pub item_drops: Vec<EconItem>,
    pub convars: AHashMap<String, String>,
    pub chat_messages: Vec<ChatMessageRecord>,
    pub player_end_data: Vec<PlayerEndMetaData>,
    // Settings
    pub wanted_events: Vec<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,
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

#[derive(Debug, Clone)]
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
    pub item_name: Option<String>,
    pub skin_name: Option<String>,
}
#[derive(Debug, Clone)]
pub struct PlayerEndMetaData {
    pub steamid: Option<u64>,
    pub name: Option<String>,
    pub team_number: Option<i32>,
}

impl<'a> SecondPassParser<'a> {
    pub fn create_output(self) -> SecondPassOutput {
        SecondPassOutput {
            voice_data: self.voice_data,
            chat_messages: self.chat_messages,
            convars: self.convars,
            df: self.output,
            game_events: self.game_events,
            skins: self.skins,
            item_drops: self.item_drops,
            header: None,
            player_md: self.player_end_data,
            game_events_counter: self.game_events_counter,
            prop_info: PropController::new(vec![], vec![], AHashMap::default(), false),
            projectiles: self.projectile_records,
            ptr: self.ptr,
        }
    }
    pub fn new(first_pass_output: FirstPassOutput<'a>, offset: usize, parse_all_packets: bool) -> Result<Self, DemoParserError> {
        first_pass_output.settings.wanted_player_props.clone().extend(vec![
            "tick".to_owned(),
            "steamid".to_owned(),
            "name".to_owned(),
        ]);
        let args: Vec<String> = env::args().collect();
        let debug = if args.len() > 2 { args[2] == "true" } else { false };
        Ok(SecondPassParser {
            voice_data: vec![],
            paths: vec![
                FieldPath {
                    last: 0,
                    path: [0, 0, 0, 0, 0, 0, 0],
                };
                8192
            ],
            parse_inventory: first_pass_output
                .prop_controller
                .wanted_player_props
                .contains(&"inventory".to_string()),
            net_tick: 0,
            c4_entity_id: None,
            stringtable_players: first_pass_output.stringtable_players,
            is_debug_mode: debug,
            projectile_records: vec![],
            parse_all_packets: parse_all_packets,
            wanted_players: first_pass_output.wanted_players.clone(),
            wanted_ticks: first_pass_output.wanted_ticks.clone(),
            prop_controller: &first_pass_output.prop_controller,
            qf_mapper: &first_pass_output.qfmap,
            fullpackets_parsed: 0,
            serializers: AHashMap::default(),
            ptr: offset,
            ge_list: first_pass_output.ge_list,
            cls_by_id: &first_pass_output.cls_by_id,
            entities: vec![None; DEFAULT_MAX_ENTITY_ID],
            cls_bits: None,
            tick: -99999,
            players: BTreeMap::default(),
            output: AHashMap::default(),
            game_events: vec![],
            wanted_events: first_pass_output.settings.wanted_events.clone(),
            parse_entities: first_pass_output.settings.parse_ents,
            projectiles: BTreeSet::default(),
            baselines: first_pass_output.baselines.clone(),
            string_tables: first_pass_output.string_tables.clone(),
            teams: Teams::new(),
            game_events_counter: AHashSet::default(),
            parse_projectiles: first_pass_output.settings.parse_projectiles,
            rules_entity_id: None,
            convars: AHashMap::default(),
            chat_messages: vec![],
            item_drops: vec![],
            skins: vec![],
            player_end_data: vec![],
            huffman_lookup_table: &first_pass_output.settings.huffman_lookup_table,
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
    pub life_state: Option<u32>,

    pub h_owner_entity: Option<u32>,
    pub agent_skin_idx: Option<u32>,
    pub total_rounds_played: Option<u32>,

    pub round_win_reason: Option<u32>,
    pub is_freeze_period: Option<u32>,
    pub round_start_time: Option<u32>,
}
impl SpecialIDs {
    pub fn new() -> Self {
        SpecialIDs {
            is_freeze_period: None,
            round_start_time: None,
            round_win_reason: None,
            total_rounds_played: None,
            h_owner_entity: None,
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
            life_state: None,
            agent_skin_idx: None,
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

pub fn create_huffman_lookup_table() -> Vec<(u8, u8)> {
    let mut huffman_table = vec![(255, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];
    let mut huffman_rev_table = vec![(255, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];

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
    huffman_table[0] = (255, 255);

    const RIGHTSHIFT_BITORDER: u32 = 64 - 17;
    let mut v: Vec<u32> = vec![];
    for (idx, x) in huffman_table.iter().enumerate() {
        if x.0 != 255 {
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
                let peekbits = (idx as u64).reverse_bits() >> RIGHTSHIFT_BITORDER;
                huffman_table[idx as usize] = huffman_table[x as usize];
                huffman_rev_table[peekbits as usize] = huffman_table[x as usize];
            }
        }
    }
    for v in 0..HUF_LOOKUPTABLE_MAXVALUE {
        let p: u64 = (v as u64).reverse_bits() >> RIGHTSHIFT_BITORDER;
        if p & 1 == 0 {
            huffman_rev_table[p as usize] = (0, 1);
        }
    }
    huffman_rev_table
}
