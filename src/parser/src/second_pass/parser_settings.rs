use crate::first_pass::frameparser::StartEndOffset;
use crate::first_pass::parser::FirstPassOutput;
use crate::first_pass::prop_controller::{PropController, BUTTONS_PROP_NAME};
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Serializer;
use crate::first_pass::stringtables::StringTable;
use crate::first_pass::stringtables::UserInfo;
use crate::maps::BUTTONMAP;
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
use csgoproto::csvc_msg_game_event_list::DescriptorT;
use csgoproto::CsgoUserCmdPb;
use csgoproto::CsvcMsgVoiceData;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
const HUF_LOOKUPTABLE_MAXVALUE: u32 = (1 << 17) - 1;
const DEFAULT_MAX_ENTITY_ID: usize = 1024;
pub const USER_CMD_RING_SIZE: usize = 150;

#[derive(Debug, Clone)]
pub struct UserCmdRingEntry {
    pub command_number: i32,
    pub command: CsgoUserCmdPb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCmdBaselineSource {
    ExactCommandNumber,
    RingSlotMismatch,
    CurrentCommandFallback,
}

#[derive(Debug, Clone)]
pub struct UserCmdRing {
    entries: Vec<Option<UserCmdRingEntry>>,
}

impl Default for UserCmdRing {
    fn default() -> Self {
        Self {
            entries: (0..USER_CMD_RING_SIZE).map(|_| None).collect(),
        }
    }
}

impl UserCmdRing {
    fn index(command_number: i32) -> usize {
        command_number.rem_euclid(USER_CMD_RING_SIZE as i32) as usize
    }

    pub fn insert(&mut self, command_number: i32, command: CsgoUserCmdPb) {
        self.entries[Self::index(command_number)] = Some(UserCmdRingEntry { command_number, command });
    }

    pub fn get(&self, command_number: i32) -> Option<&CsgoUserCmdPb> {
        self.entries[Self::index(command_number)]
            .as_ref()
            .filter(|entry| entry.command_number == command_number)
            .map(|entry| &entry.command)
    }

    pub fn contains(&self, command_number: i32) -> bool {
        self.get(command_number).is_some()
    }

    pub fn slot_command_number(&self, command_number: i32) -> Option<i32> {
        self.entries[Self::index(command_number)].as_ref().map(|entry| entry.command_number)
    }

    pub fn get_slot(&self, command_number: i32) -> Option<&CsgoUserCmdPb> {
        self.entries[Self::index(command_number)].as_ref().map(|entry| &entry.command)
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserCmdPlayerState {
    pub ring: UserCmdRing,
    pub current_command_number: Option<i32>,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct UserCmdTestRecord {
    pub player_slot: i32,
    pub command_number: i32,
    pub server_tick_executed: i32,
    pub client_tick: i32,
    pub baseline_command_number: Option<i32>,
    pub baseline_source: Option<UserCmdBaselineSource>,
    pub delta_data: Option<Vec<u8>>,
    pub command: CsgoUserCmdPb,
}

#[cfg(test)]
pub(crate) struct UserCmdTestRecordRef<'a> {
    pub player_slot: i32,
    pub command_number: i32,
    pub server_tick_executed: i32,
    pub client_tick: i32,
    pub baseline_command_number: Option<i32>,
    pub baseline_source: Option<UserCmdBaselineSource>,
    pub delta_data: Option<&'a [u8]>,
    pub command: &'a CsgoUserCmdPb,
}

#[cfg(test)]
pub(crate) type UserCmdTestSink = Box<dyn for<'record> FnMut(UserCmdTestRecordRef<'record>)>;

#[cfg(test)]
pub(crate) struct UserCmdTransportTestRecord {
    pub player_slot: i32,
    pub command_number: i32,
    pub server_tick_executed: i32,
    pub client_tick: i32,
    pub server_cmd_has_bits: u32,
}

#[cfg(test)]
pub(crate) type UserCmdTransportTestSink = Box<dyn FnMut(UserCmdTransportTestRecord)>;

impl UserCmdPlayerState {
    pub fn resolve_baseline(&self, requested_command_number: i32) -> Option<(&CsgoUserCmdPb, UserCmdBaselineSource)> {
        self.ring
            .get(requested_command_number)
            .map(|command| (command, UserCmdBaselineSource::ExactCommandNumber))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserCmdDecodeStats {
    pub full_data: u64,
    pub full_decode_failures: u64,
    pub delta_data: u64,
    pub delta_applied: u64,
    pub baseline_missing: u64,
    pub baseline_mismatch: u64,
    pub baseline_fallbacks: u64,
    pub delta_decode_failures: u64,
    pub delta_sanitize_failures: u64,
    pub delta_proto_failures: u64,
    pub delta_repeated_failures: u64,
    pub delta_input_history_failures: u64,
    pub delta_subtick_failures: u64,
    pub delta_repeated_malformed: u64,
    pub delta_repeated_truncated: u64,
    pub delta_repeated_invalid_index: u64,
    pub delta_repeated_out_of_bounds: u64,
    pub delta_repeated_invalid_message: u64,
    pub delta_repeated_invalid_nested: u64,
    pub delta_nested_failures: u64,
    pub decoded_usercmds: u64,
    pub decoded_full_usercmds: u64,
    pub decoded_delta_usercmds: u64,
    pub decoded_nonzero_buttons: u64,
    pub decoded_mouse_movement: u64,
    pub decoded_weapon_selection: u64,
    pub decoded_subtick_moves: u64,
    pub max_subtick_moves: u64,
    pub max_input_history: u64,
    pub decoded_cannot_move: u64,
    pub decoded_dead: u64,
    pub player_slot_mask: u64,
}

impl UserCmdDecodeStats {
    pub fn merge(&mut self, other: &Self) {
        self.full_data += other.full_data;
        self.full_decode_failures += other.full_decode_failures;
        self.delta_data += other.delta_data;
        self.delta_applied += other.delta_applied;
        self.baseline_missing += other.baseline_missing;
        self.baseline_mismatch += other.baseline_mismatch;
        self.baseline_fallbacks += other.baseline_fallbacks;
        self.delta_decode_failures += other.delta_decode_failures;
        self.delta_sanitize_failures += other.delta_sanitize_failures;
        self.delta_proto_failures += other.delta_proto_failures;
        self.delta_repeated_failures += other.delta_repeated_failures;
        self.delta_input_history_failures += other.delta_input_history_failures;
        self.delta_subtick_failures += other.delta_subtick_failures;
        self.delta_repeated_malformed += other.delta_repeated_malformed;
        self.delta_repeated_truncated += other.delta_repeated_truncated;
        self.delta_repeated_invalid_index += other.delta_repeated_invalid_index;
        self.delta_repeated_out_of_bounds += other.delta_repeated_out_of_bounds;
        self.delta_repeated_invalid_message += other.delta_repeated_invalid_message;
        self.delta_repeated_invalid_nested += other.delta_repeated_invalid_nested;
        self.delta_nested_failures += other.delta_nested_failures;
        self.decoded_usercmds += other.decoded_usercmds;
        self.decoded_full_usercmds += other.decoded_full_usercmds;
        self.decoded_delta_usercmds += other.decoded_delta_usercmds;
        self.decoded_nonzero_buttons += other.decoded_nonzero_buttons;
        self.decoded_mouse_movement += other.decoded_mouse_movement;
        self.decoded_weapon_selection += other.decoded_weapon_selection;
        self.decoded_subtick_moves += other.decoded_subtick_moves;
        self.max_subtick_moves = self.max_subtick_moves.max(other.max_subtick_moves);
        self.max_input_history = self.max_input_history.max(other.max_input_history);
        self.decoded_cannot_move += other.decoded_cannot_move;
        self.decoded_dead += other.decoded_dead;
        self.player_slot_mask |= other.player_slot_mask;
    }
}

pub struct SecondPassParser<'a> {
    pub start_end_offset: Option<StartEndOffset>,
    pub qf_mapper: &'a QfMapper,
    pub prop_controller: &'a PropController,
    pub cls_by_id: &'a Vec<Class>,
    pub stringtable_players: BTreeMap<i32, UserInfo>,
    pub net_tick: u32,
    pub parse_inventory: bool,
    pub paths: Vec<FieldPath>,
    pub ptr: usize,
    pub parse_all_packets: bool,
    pub ge_list: &'a AHashMap<i32, DescriptorT>,
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
    pub uniq_prop_names: AHashSet<String>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub projectiles: BTreeSet<i32>,
    pub fullpackets_parsed: u32,
    pub wanted_players: AHashSet<u64>,
    pub wanted_ticks: AHashSet<i32>,
    // Output from parsing
    pub projectile_records: Vec<ProjectileRecord>,
    pub voice_data: Vec<(i32, CsvcMsgVoiceData)>,
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
    pub parse_grenades: bool,
    pub is_debug_mode: bool,
    pub df_per_player: AHashMap<u64, AHashMap<u32, PropColumn>>,
    pub order_by_steamid: bool,
    pub last_tick: i32,
    pub parse_usercmd: bool,
    pub usercmd_states: AHashMap<i32, UserCmdPlayerState>,
    pub usercmd_seen: AHashSet<(i32, i32)>,
    pub usercmd_stats: UserCmdDecodeStats,
    #[cfg(test)]
    pub(crate) usercmd_capture_counts: Option<AHashMap<(i32, i32, i32, i32), usize>>,
    #[cfg(test)]
    pub(crate) usercmd_captured_counts: AHashMap<(i32, i32, i32, i32), usize>,
    #[cfg(test)]
    pub(crate) usercmd_records: Vec<UserCmdTestRecord>,
    #[cfg(test)]
    pub(crate) usercmd_record_sink: Option<UserCmdTestSink>,
    #[cfg(test)]
    pub(crate) usercmd_transport_sink: Option<UserCmdTransportTestSink>,
    pub list_props: bool,
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
            roster: self
                .players
                .values()
                .map(|p| PlayerEndMetaData {
                    steamid: p.steamid,
                    name: p.name.clone(),
                    team_number: p.team_num.map(|t| t as i32),
                })
                .collect(),
            game_events_counter: self.game_events_counter,
            uniq_prop_names: self.uniq_prop_names,
            prop_info: PropController::new(vec![], vec![], AHashMap::default(), AHashMap::default(), false, &["none".to_string()], false),
            projectiles: self.projectile_records,
            ptr: self.ptr,
            df_per_player: self.df_per_player,
            entities: self.entities,
            last_tick: self.tick,
            usercmd_stats: self.usercmd_stats,
        }
    }
    pub fn new(
        first_pass_output: FirstPassOutput<'a>,
        offset: usize,
        parse_all_packets: bool,
        start_end_offset: Option<StartEndOffset>,
    ) -> Result<Self, DemoParserError> {
        first_pass_output
            .settings
            .wanted_player_props
            .clone()
            .extend(vec!["tick".to_owned(), "steamid".to_owned(), "name".to_owned()]);
        let args: Vec<String> = env::args().collect();
        let debug = if args.len() > 2 { args[2] == "true" } else { false };

        Ok(SecondPassParser {
            uniq_prop_names: AHashSet::default(),
            parse_usercmd: wants_usercmd_props(&first_pass_output.settings.wanted_player_props),
            usercmd_states: AHashMap::default(),
            usercmd_seen: AHashSet::default(),
            usercmd_stats: UserCmdDecodeStats::default(),
            #[cfg(test)]
            usercmd_capture_counts: None,
            #[cfg(test)]
            usercmd_captured_counts: AHashMap::default(),
            #[cfg(test)]
            usercmd_records: Vec::new(),
            #[cfg(test)]
            usercmd_record_sink: None,
            #[cfg(test)]
            usercmd_transport_sink: None,
            last_tick: 0,
            start_end_offset: start_end_offset,
            order_by_steamid: first_pass_output.order_by_steamid,
            df_per_player: AHashMap::default(),
            voice_data: vec![],
            paths: vec![
                FieldPath {
                    last: 0,
                    path: [0, 0, 0, 0, 0, 0, 0],
                };
                8192
            ],
            parse_inventory: first_pass_output.prop_controller.wanted_player_props.contains(&"inventory".to_string()),
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
            parse_grenades: first_pass_output.settings.parse_grenades,
            rules_entity_id: None,
            convars: AHashMap::default(),
            chat_messages: vec![],
            item_drops: vec![],
            skins: vec![],
            player_end_data: vec![],
            huffman_lookup_table: &first_pass_output.settings.huffman_lookup_table,
            header: HashMap::default(),
            list_props: first_pass_output.list_props,
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
    pub round_start_count: Option<u32>,
    pub round_end_count: Option<u32>,
    pub match_end_count: Option<u32>,

    pub is_incendiary_grenade: Option<u32>,
    pub sellback_entry_def_idx: Option<u32>,
    pub sellback_entry_n_cost: Option<u32>,
    pub sellback_entry_prev_armor: Option<u32>,
    pub sellback_entry_prev_helmet: Option<u32>,
    pub sellback_entry_h_item: Option<u32>,

    pub weapon_purchase_count: Option<u32>,
    pub in_buy_zone: Option<u32>,
    pub custom_name: Option<u32>,

    pub is_airborn: Option<u32>,
    pub initial_velocity: Option<u32>,
}
impl SpecialIDs {
    pub fn new() -> Self {
        SpecialIDs {
            round_start_count: None,
            round_end_count: None,
            match_end_count: None,
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
            is_incendiary_grenade: None,
            sellback_entry_def_idx: None,
            sellback_entry_h_item: None,
            sellback_entry_n_cost: None,
            sellback_entry_prev_armor: None,
            sellback_entry_prev_helmet: None,
            weapon_purchase_count: None,
            in_buy_zone: None,
            custom_name: None,
            is_airborn: None,
            initial_velocity: None,
        }
    }
}

pub fn create_huffman_lookup_table() -> Vec<(u8, u8)> {
    let buf = include_bytes!("huf.b");
    let mut huf2 = Vec::with_capacity(HUF_LOOKUPTABLE_MAXVALUE as usize);
    for chunk in buf.chunks_exact(2) {
        huf2.push((chunk[0], chunk[1]));
    }
    huf2.push((0, 0));
    return huf2;
}

pub fn wants_usercmd_props(names: &[String]) -> bool {
    names
        .iter()
        .any(|name| name.contains("usercmd") || BUTTONMAP.get(name.as_str()).is_some() || name == BUTTONS_PROP_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::first_pass::parser_settings::rm_user_friendly_names;

    #[test]
    fn ring_requires_an_exact_command_number_match() {
        let mut ring = UserCmdRing::default();
        ring.insert(1, CsgoUserCmdPb::default());
        assert!(ring.contains(1));
        assert!(!ring.contains(151));

        ring.insert(151, CsgoUserCmdPb::default());
        assert!(!ring.contains(1));
        assert!(ring.contains(151));
        assert_eq!(ring.slot_command_number(1), Some(151));
    }

    #[test]
    fn baseline_resolution_rejects_missing_or_mismatched_ring_entries() {
        let mut state = UserCmdPlayerState::default();
        state.ring.insert(100, CsgoUserCmdPb::default());
        state.current_command_number = Some(100);

        assert!(state.resolve_baseline(99).is_none());

        state.ring.insert(249, CsgoUserCmdPb::default());
        assert!(state.resolve_baseline(99).is_none());
    }

    #[test]
    fn buttons_request_enables_usercmd_decode() {
        assert!(wants_usercmd_props(&[BUTTONS_PROP_NAME.to_string()]));
        assert!(wants_usercmd_props(&["usercmd_mouse_dx".to_string()]));
        assert!(!wants_usercmd_props(&["CCSPlayerPawn.m_iHealth".to_string()]));
    }

    #[test]
    fn buttons_friendly_name_resolves_to_the_legacy_mask_prop() {
        assert_eq!(
            rm_user_friendly_names(&vec!["buttons".to_string()]).unwrap(),
            vec![BUTTONS_PROP_NAME.to_string()]
        );
    }
}
