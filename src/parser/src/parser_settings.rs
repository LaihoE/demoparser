use super::entities_utils::FieldPath;
use super::game_events::GameEvent;
use super::read_bits::DemoParserError;
use super::sendtables::Serializer;
use super::stringtables::StringTable;
use super::variants::PropColumn;
use crate::collect_data::ProjectileRecordVec;
use crate::decoder::QfMapper;
use crate::entities::Entity;
use crate::entities::PlayerMetaData;
use crate::other_netmessages::Class;
use crate::sendtables::FieldInfo;
use crate::sendtables::FieldModel;
use crate::sendtables::PropInfo;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::HashMap;
use ahash::RandomState;
use bit_reverse::LookupReverse;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use dashmap::DashMap;
use memmap2::Mmap;
use phf_macros::phf_map;
use soa_derive::StructOfArray;
use std::collections::BTreeMap;
use std::sync::Arc;

// Wont fit in L1, evaluate if worth to use pointer method
const HUF_LOOKUPTABLE_MAXVALUE: u32 = (1 << 17) - 1;

pub struct Parser<'a> {
    pub ptr: usize,
    pub bytes: Arc<Mmap>,
    // Parsing state
    pub ge_list: Option<AHashMap<i32, Descriptor_t>>,
    pub serializers: AHashMap<String, Serializer, RandomState>,
    pub cls_by_id: &'a AHashMap<u32, Class>,
    pub cls_bits: Option<u32>,
    pub entities: AHashMap<i32, Entity, RandomState>,
    pub tick: i32,
    pub players: BTreeMap<i32, PlayerMetaData>,
    pub teams: Teams,
    pub prop_name_to_path: AHashMap<String, [i32; 7]>,
    pub path_to_prop_name: AHashMap<[i32; 7], String>,
    pub wanted_prop_paths: AHashSet<[i32; 7]>,
    pub huffman_lookup_table: Arc<Vec<(u32, u8)>>,
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<StringTable>,
    pub rules_entity_id: Option<i32>,
    pub game_events_counter: AHashMap<String, i32>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub paths: Vec<FieldInfo>,
    pub projectiles: AHashSet<i32, RandomState>,
    pub prop_infos: Vec<PropInfo>,
    pub controller_ids: SpecialIDs,
    pub fullpackets_parsed: u32,
    pub packets_parsed: u32,
    pub cnt: AHashMap<FieldModel, u32>,
    pub qf_map: QfMapper,

    // Output from parsing
    pub output: AHashMap<u32, PropColumn, RandomState>,
    pub header: HashMap<String, String>,
    pub skins: EconItemVec,
    pub item_drops: EconItemVec,
    pub convars: AHashMap<String, String>,
    pub chat_messages: ChatMessageRecordVec,
    pub player_end_data: PlayerEndDataVec,
    pub projectile_records: ProjectileRecordVec,

    // Settings
    pub wanted_ticks: AHashSet<i32, RandomState>,
    pub wanted_player_props: Vec<String>,
    pub wanted_player_props_og_names: Vec<String>,
    // Team and rules props
    pub wanted_other_props: Vec<String>,
    pub wanted_other_props_og_names: Vec<String>,
    pub wanted_event: Option<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,
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

#[derive(Debug, Clone)]
pub struct ParserInputs {
    pub bytes: Arc<Mmap>,

    pub wanted_player_props: Vec<String>,
    pub wanted_player_props_og_names: Vec<String>,
    pub wanted_other_props: Vec<String>,
    pub wanted_other_props_og_names: Vec<String>,

    pub wanted_ticks: Vec<i32>,
    pub wanted_event: Option<String>,
    pub parse_ents: bool,
    pub parse_projectiles: bool,
    pub only_header: bool,
    pub count_props: bool,
    pub only_convars: bool,
    pub huffman_lookup_table: Arc<Vec<(u32, u8)>>,
}

impl<'a> Parser<'a> {
    pub fn new(
        mut settings: ParserInputs,
        cls_by_id: &'a AHashMap<u32, Class>,
    ) -> Result<Self, DemoParserError> {
        let fp_filler = FieldPath {
            last: 0,
            path: [-1, 0, 0, 0, 0, 0, 0],
        };
        settings.wanted_player_props.extend(vec![
            "tick".to_owned(),
            "steamid".to_owned(),
            "name".to_owned(),
        ]);
        // let huffman_table = create_huffman_lookup_table();
        Ok(Parser {
            qf_map: QfMapper {
                idx: 0,
                map: AHashMap::default(),
            },
            fullpackets_parsed: 0,
            packets_parsed: 0,
            controller_ids: SpecialIDs {
                team_team_num: None,
                teamnum: None,
                player_name: None,
                steamid: None,
                player_pawn: None,
                player_team_pointer: None,
                weapon_owner_pointer: None,
                cell_x_offset_player: None,
                cell_x_player: None,
                cell_y_player: None,
                cell_y_offset_player: None,
                cell_z_offset_player: None,
                cell_z_player: None,
                active_weapon: None,
            },
            cnt: AHashMap::default(),
            serializers: AHashMap::default(),
            ptr: 0,
            ge_list: None,
            bytes: settings.bytes,
            // JUST LOL
            cls_by_id: cls_by_id,
            entities: AHashMap::default(),
            cls_bits: None,
            tick: -99999,
            wanted_player_props: settings.wanted_player_props,
            players: BTreeMap::default(),
            output: AHashMap::default(),
            wanted_ticks: AHashSet::from_iter(settings.wanted_ticks),
            game_events: vec![],
            wanted_event: settings.wanted_event,
            parse_entities: settings.parse_ents,
            projectiles: AHashSet::default(),
            projectile_records: ProjectileRecordVec::new(),
            baselines: AHashMap::default(),
            string_tables: vec![],
            paths: vec![
                FieldInfo {
                    decoder: crate::sendtables::Decoder::NoscaleDecoder,
                    should_parse: false,
                    df_pos: 0,
                    controller_prop: None
                };
                4096
            ],
            prop_infos: vec![],
            teams: Teams::new(),
            game_events_counter: AHashMap::default(),
            parse_projectiles: settings.parse_projectiles,
            rules_entity_id: None,
            convars: AHashMap::default(),
            chat_messages: ChatMessageRecordVec::new(),
            item_drops: EconItemVec::new(),
            skins: EconItemVec::new(),
            player_end_data: PlayerEndDataVec::new(),
            huffman_lookup_table: settings.huffman_lookup_table,
            prop_name_to_path: AHashMap::default(),
            wanted_prop_paths: AHashSet::default(),
            path_to_prop_name: AHashMap::default(),
            header: HashMap::default(),
            wanted_player_props_og_names: settings.wanted_player_props_og_names,
            wanted_other_props: settings.wanted_other_props,
            wanted_other_props_og_names: settings.wanted_other_props_og_names,
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

pub static FRIENDLY_NAMES_MAPPING: phf::Map<&'static str, &'static str> = phf_map! {
    "team_surrendered" => "CCSTeam.m_bSurrendered",
    "team_rounds_total" => "CCSTeam.m_iScore",
    "team_name" => "CCSTeam.m_szTeamname",
    "team_score_overtime" => "CCSTeam.m_scoreOvertime",
    "team_match_stat"=>"CCSTeam.m_szTeamMatchStat",
    "team_num_map_victories"=>"CCSTeam.m_numMapVictories",
    "team_score_first_half"=>"CCSTeam.m_scoreFirstHalf",
    "team_score_second_half"=>"CCSTeam.m_scoreSecondHalf",
    "team_clan_name" =>"CCSTeam.m_szClanTeamname",
    "is_freeze_period"=>"CCSGameRulesProxy.CCSGameRules.m_bFreezePeriod",
    "is_warmup_period"=>"CCSGameRulesProxy.CCSGameRules.m_bWarmupPeriod" ,
    "warmup_period_end"=>"CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodEnd" ,
    "warmup_period_start"=>"CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodStart" ,
    "is_terrorist_timeout"=>"CCSGameRulesProxy.CCSGameRules.m_bTerroristTimeOutActive" ,
    "is_ct_timeout"=>"CCSGameRulesProxy.CCSGameRules.m_bCTTimeOutActive" ,
    "terrorist_timeout_remaining"=>"CCSGameRulesProxy.CCSGameRules.m_flTerroristTimeOutRemaining" ,
    "ct_timeout_remaining"=>"CCSGameRulesProxy.CCSGameRules.m_flCTTimeOutRemaining" ,
    "num_terrorist_timeouts"=>"CCSGameRulesProxy.CCSGameRules.m_nTerroristTimeOuts" ,
    "num_ct_timeouts"=>"CCSGameRulesProxy.CCSGameRules.m_nCTTimeOuts" ,
    "is_technical_timeout"=>"CCSGameRulesProxy.CCSGameRules.m_bTechnicalTimeOut" ,
    "is_waiting_for_resume"=>"CCSGameRulesProxy.CCSGameRules.m_bMatchWaitingForResume" ,
    "match_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_fMatchStartTime" ,
    "round_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_fRoundStartTime" ,
    "restart_round_time"=>"CCSGameRulesProxy.CCSGameRules.m_flRestartRoundTime" ,
    "is_game_restart?"=>"CCSGameRulesProxy.CCSGameRules.m_bGameRestart" ,
    "game_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_flGameStartTime" ,
    "time_until_next_phase_start"=>"CCSGameRulesProxy.CCSGameRules.m_timeUntilNextPhaseStarts" ,
    "game_phase"=>"CCSGameRulesProxy.CCSGameRules.m_gamePhase" ,
    "total_rounds_played"=>"CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed" ,
    "rounds_played_this_phase"=>"CCSGameRulesProxy.CCSGameRules.m_nRoundsPlayedThisPhase" ,
    "hostages_remaining"=>"CCSGameRulesProxy.CCSGameRules.m_iHostagesRemaining" ,
    "any_hostages_reached"=>"CCSGameRulesProxy.CCSGameRules.m_bAnyHostageReached" ,
    "has_bombites"=>"CCSGameRulesProxy.CCSGameRules.m_bMapHasBombTarget" ,
    "has_rescue_zone"=>"CCSGameRulesProxy.CCSGameRules.m_bMapHasRescueZone" ,
    "has_buy_zone"=>"CCSGameRulesProxy.CCSGameRules.m_bMapHasBuyZone" ,
    "is_matchmaking"=>"CCSGameRulesProxy.CCSGameRules.m_bIsQueuedMatchmaking" ,
    "match_making_mode"=>"CCSGameRulesProxy.CCSGameRules.m_nQueuedMatchmakingMode" ,
    "is_valve_dedicated_server"=>"CCSGameRulesProxy.CCSGameRules.m_bIsValveDS" ,
    "gungame_prog_weap_ct"=>"CCSGameRulesProxy.CCSGameRules.m_iNumGunGameProgressiveWeaponsCT" ,
    "gungame_prog_weap_t"=>"CCSGameRulesProxy.CCSGameRules.m_iNumGunGameProgressiveWeaponsT" ,
    "spectator_slot_count"=>"CCSGameRulesProxy.CCSGameRules.m_iSpectatorSlotCount" ,
    "is_match_started"=>"CCSGameRulesProxy.CCSGameRules.m_bHasMatchStarted" ,
    "n_best_of_maps"=>"CCSGameRulesProxy.CCSGameRules.m_numBestOfMaps" ,
    "is_bomb_dropped"=>"CCSGameRulesProxy.CCSGameRules.m_bBombDropped" ,
    "is_bomb_planed"=>"CCSGameRulesProxy.CCSGameRules.m_bBombPlanted" ,
    "round_win_status"=>"CCSGameRulesProxy.CCSGameRules.m_iRoundWinStatus" ,
    "round_win_reason"=>"CCSGameRulesProxy.CCSGameRules.m_eRoundWinReason" ,
    "terrorist_cant_buy"=>"CCSGameRulesProxy.CCSGameRules.m_bTCantBuy" ,
    "ct_cant_buy"=>"CCSGameRulesProxy.CCSGameRules.m_bCTCantBuy" ,
    "num_player_alive_ct"=>"CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_CT" ,
    "num_player_alive_t"=>"CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_T" ,
    "ct_losing_streak"=>"CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveCTLoses" ,
    "t_losing_streak"=>"CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveTerroristLoses" ,
    "survival_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_flSurvivalStartTime" ,
    "round_in_progress"=>"CCSGameRulesProxy.CCSGameRules.m_bRoundInProgress" ,
    "i_bomb_site?"=>"CCSGameRulesProxy.CCSGameRules.m_iBombSite" ,
    "is_auto_muted"=>"CCSPlayerController.m_bHasCommunicationAbuseMute",
    "crosshair_code"=>"CCSPlayerController.m_szCrosshairCodes",
    "pending_team_num"=>"CCSPlayerController.m_iPendingTeamNum",
    "player_color"=>"CCSPlayerController.m_iCompTeammateColor",
    "ever_played_on_team"=>"CCSPlayerController.m_bEverPlayedOnTeam",
    "clan_name"=>"CCSPlayerController.m_szClan",
    "is_coach_team"=>"CCSPlayerController.m_iCoachingTeam",
    "comp_rank"=>"CCSPlayerController.m_iCompetitiveRanking",
    "comp_wins"=>"CCSPlayerController.m_iCompetitiveWins",
    "comp_rank_type"=>"CCSPlayerController.m_iCompetitiveRankType",
    "is_controlling_bot"=>"CCSPlayerController.m_bControllingBot",
    "has_controlled_bot_this_round"=>"CCSPlayerController.m_bHasControlledBotThisRound",
    "can_control_bot"=>"CCSPlayerController.m_bCanControlObservedBot",
    "is_alive"=>"CCSPlayerController.m_bPawnIsAlive",
    "armor"=>"CCSPlayerController.m_iPawnArmor",
    "has_defuser"=>"CCSPlayerController.m_bPawnHasDefuser",
    "has_helmet"=>"CCSPlayerController.m_bPawnHasHelmet",
    "spawn_time"=>"CCSPlayerController.m_iPawnLifetimeStart",
    "death_time"=>"CCSPlayerController.m_iPawnLifetimeEnd",
    "score"=>"CCSPlayerController.m_iScore",
    "game_time"=>"CCSPlayerController.m_flSimulationTime",
    "is_connected"=>"CCSPlayerController.m_iConnected",
    "player_name"=>"CCSPlayerController.m_iszPlayerName",
    "player_steamid"=>"CCSPlayerController.m_steamID",
    "fov"=>"CCSPlayerController.m_iDesiredFOV",
    "balance"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount",
    "start_balance"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iStartAccount",
    "total_cash_spent"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iTotalCashSpent",
    "cash_spent_this_round"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iCashSpentThisRound",
    "music_kit_id"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_unMusicID",
    "leader_honors"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsLeader",
    "teacher_honors"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsTeacher",
    "friendly_honors"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsFriendly",
    "kills_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKills",
    "deaths_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDeaths",
    "assists_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iAssists",
    "alive_time_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iLiveTime",
    "headshot_kills_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iHeadShotKills",
    "damage_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDamage",
    "objective_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iObjective",
    "utility_damage_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iUtilityDamage",
    "enemies_flashed_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEnemiesFlashed",
    "equipment_value_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEquipmentValue",
    "money_saved_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iMoneySaved",
    "kill_reward_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKillReward",
    "cash_earned_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iCashEarned",
    "kills_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKills",
    "deaths_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDeaths",
    "assists_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iAssists",
    "alive_time_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iLiveTime",
    "headshot_kills_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iHeadShotKills",
    "ace_rounds_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy5Ks",
    "4k_rounds_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy4Ks",
    "3k_rounds_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy3Ks",
    "damage_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDamage",
    "objective_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iObjective",
    "utility_damage_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iUtilityDamage",
    "enemies_flashed_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemiesFlashed",
    "equipment_value_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEquipmentValue",
    "money_saved_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iMoneySaved",
    "kill_reward_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKillReward",
    "cash_earned_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iCashEarned",
    "ping"=>"CCSPlayerController.m_iPing",
    "move_collide" => "CCSPlayerPawn.m_MoveCollide",
    "move_type" =>  "CCSPlayerPawn.m_MoveType",
    "team_num" => "CCSPlayerPawn.m_iTeamNum",
    "active_weapon" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon",
    "looking_at_weapon" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_bIsLookingAtWeapon",
    "holding_look_at_weapon" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_bIsHoldingLookAtWeapon",
    "next_attack_time" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_flNextAttack",
    "duck_time_ms" =>"CCSPlayerPawn.CCSPlayer_MovementServices.m_nDuckTimeMsecs",
    "max_speed" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxspeed",
    "max_fall_velo" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxFallVelocity",
    "duck_amount" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckAmount",
    "duck_speed" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckSpeed",
    "duck_overrdie" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDuckOverride",
    "old_jump_pressed" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bOldJumpPressed",
    "jump_until" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpUntil",
    "jump_velo" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpVel",
    "fall_velo" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flFallVelocity",
    "in_crouch" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bInCrouch",
    "crouch_state" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_nCrouchState",
    "ducked" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDucked",
    "ducking" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDucking",
    "in_duck_jump" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bInDuckJump",
    "allow_auto_movement" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bAllowAutoMovement",
    "jump_time_ms" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_nJumpTimeMsecs",
    "last_duck_time" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flLastDuckTime",
    "is_rescuing" => "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_bIsRescuing",
    "weapon_purchases_this_match" => "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_iWeaponPurchasesThisMatch",
    "weapon_purchases_this_round" => "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_iWeaponPurchasesThisRound",
    "spotted" => "CCSPlayerPawn.m_bSpotted",
    "spotted_mask" => "CCSPlayerPawn.m_bSpottedByMask",
    "time_last_injury" => "CCSPlayerPawn.m_flTimeOfLastInjury",
    "direction_last_injury" => "CCSPlayerPawn.m_nRelativeDirectionOfLastInjury",
    "player_state" => "CCSPlayerPawn.m_iPlayerState",
    "passive_items" => "CCSPlayerPawn.m_passiveItems",
    "is_scoped" => "CCSPlayerPawn.m_bIsScoped",
    "is_walking" => "CCSPlayerPawn.m_bIsWalking",
    "resume_zoom" => "CCSPlayerPawn.m_bResumeZoom",
    "is_defusing" =>"CCSPlayerPawn.m_bIsDefusing",
    "is_grabbing_hostage" => "CCSPlayerPawn.m_bIsGrabbingHostage",
    "blocking_use_in_progess" => "CCSPlayerPawn.m_iBlockingUseActionInProgress",
    "molotov_damage_time" => "CCSPlayerPawn.m_fMolotovDamageTime",
    "moved_since_spawn" => "CCSPlayerPawn.m_bHasMovedSinceSpawn",
    "in_bomb_zone" => "CCSPlayerPawn.m_bInBombZone",
    "in_buy_zone" => "CCSPlayerPawn.m_bInBuyZone",
    "in_no_defuse_area" => "CCSPlayerPawn.m_bInNoDefuseArea",
    "killed_by_taser" => "CCSPlayerPawn.m_bKilledByTaser",
    "move_state" => "CCSPlayerPawn.m_iMoveState",
    "which_bomb_zone" => "CCSPlayerPawn.m_nWhichBombZone",
    "in_hostage_rescue_zone" => "CCSPlayerPawn.m_bInHostageRescueZone",
    "stamina" => "CCSPlayerPawn.m_flStamina",
    "direction" => "CCSPlayerPawn.m_iDirection",
    "shots_fired" => "CCSPlayerPawn.m_iShotsFired",
    "armor_value" => "CCSPlayerPawn.m_ArmorValue",
    "velo_modifier" => "CCSPlayerPawn.m_flVelocityModifier",
    "ground_accel_linear_frac_last_time" => "CCSPlayerPawn.m_flGroundAccelLinearFracLastTime",
    "flash_duration" => "CCSPlayerPawn.m_flFlashDuration",
    "flash_max_alpha" => "CCSPlayerPawn.m_flFlashMaxAlpha",
    "wait_for_no_attack" => "CCSPlayerPawn.m_bWaitForNoAttack",
    "last_place_name" => "CCSPlayerPawn.m_szLastPlaceName",
    "is_strafing" => "CCSPlayerPawn.m_bStrafing",
    "round_start_equip_value" => "CCSPlayerPawn.m_unRoundStartEquipmentValue",
    "current_equip_value" => "CCSPlayerPawn.m_unCurrentEquipmentValue",
    "time" => "CCSPlayerPawn.m_flSimulationTime",
    "health" => "CCSPlayerPawn.m_iHealth",
    "life_state" => "CCSPlayerPawn.m_lifeState",
    "X"=> "X",
    "Y"=> "Y",
    "Z"=> "Z",
    "pitch" => "CCSPlayerPawnBase.m_angEyeAngles@0",
    "yaw" => "CCSPlayerPawnBase.m_angEyeAngles@1",
    "active_weapon_name" => "weapon_name",
    "active_weapon_ammo" => "m_iClip1",
    "total_ammo_left" => "m_pReserveAmmo",
    "item_def_idx" => "m_iItemDefinitionIndex",
    "weapon_quality" => "m_iEntityQuality",
    "entity_lvl" => "m_iEntityLevel",
    "item_id_high" => "m_iItemIDHigh",
    "item_id_low" => "m_iItemIDLow",
    "item_account_id" => "m_iAccountID",
    "inventory_position" => "m_iInventoryPosition",
    "is_initialized" => "m_bInitialized",
    "econ_item_attribute_def_idx" => "CEconItemAttribute.m_iAttributeDefinitionIndex",
    "econ_raw_val_32" => "CEconItemAttribute.m_iRawValue32",
    "initial_value" => "CEconItemAttribute.m_flInitialValue",
    "refundable_currency" => "CEconItemAttribute.m_nRefundableCurrency",
    "set_bonus"=> "CEconItemAttribute.m_bSetBonus",
    "custom_name" => "m_szCustomName",
    "orig_owner_xuid_low" => "m_OriginalOwnerXuidLow",
    "orig_owner_xuid_high"=> "m_OriginalOwnerXuidHigh",
    "fall_back_paint_kit" => "m_nFallbackPaintKit",
    "fall_back_seed"=> "m_nFallbackSeed",
    "fall_back_wear"=> "m_flFallbackWear",
    "fall_back_stat_track"=> "m_nFallbackStatTrak",
    "m_iState"=> "m_iState",
    "fire_seq_start_time" => "m_flFireSequenceStartTime",
    "fire_seq_start_time_change" => "m_nFireSequenceStartTimeChange",
    "is_player_fire_event_primary"=>  "m_bPlayerFireEventIsPrimary",
    "weapon_mode"=> "m_weaponMode",
    "accuracy_penalty"=> "m_fAccuracyPenalty",
    "i_recoil_idx"=> "m_iRecoilIndex",
    "fl_recoil_idx"=> "m_flRecoilIndex",
    "is_burst_mode"=> "m_bBurstMode",
    "post_pone_fire_ready_time"=> "m_flPostponeFireReadyTime",
    "is_in_reload"=> "m_bInReload",
    "reload_visually_complete"=> "m_bReloadVisuallyComplete",
    "dropped_at_time"=> "m_flDroppedAtTime",
    "is_hauled_back"=> "m_bIsHauledBack",
    "is_silencer_on"=> "m_bSilencerOn",
    "time_silencer_switch_complete"=> "m_flTimeSilencerSwitchComplete",
    "orig_team_number"=> "m_iOriginalTeamNumber",
    "prev_owner"=> "m_hPrevOwner",
    "last_shot_time"=> "m_fLastShotTime",
    "iron_sight_mode"=> "m_iIronSightMode",
    "num_empty_attacks"=> "m_iNumEmptyAttacks",
    "zoom_lvl"=> "m_zoomLevel",
    "burst_shots_remaining"=> "m_iBurstShotsRemaining",
    "needs_bolt_action"=> "m_bNeedsBoltAction",
    "next_primary_attack_tick"=> "m_nNextPrimaryAttackTick",
    "next_primary_attack_tick_ratio"=> "m_flNextPrimaryAttackTickRatio",
    "next_secondary_attack_tick" => "m_nNextSecondaryAttackTick",
    "next_secondary_attack_tick_ratio"=> "m_flNextSecondaryAttackTickRatio",
};
