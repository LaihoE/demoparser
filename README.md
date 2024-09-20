# Demo parser for Counter-Strike 2

We now have a discord channel: [Discord](https://discord.gg/rNUjS4vfnX)


Demoparser is a tool for analysing CS2 replay files "demos". All the heavy lifting is done in Rust, but you use it from the comfort of Python/JavaScript.
The parser takes a slightly different approach to exposing the information in the demo. Rather than letting you hook onto events in a streaming fashion, it lets you "query" the demo similarly to how you would interact with a database. 


### Install
Python: ```pip install demoparser2``` (python >= 3.8)

NodeJS: ```npm i @laihoe/demoparser2```

WASM: ```npm i demoparser2```


### Getting started
#### Python
```python
from demoparser2 import DemoParser

parser = DemoParser("path_to_demo.dem")
event_df = parser.parse_event("player_death", player=["X", "Y"], other=["total_rounds_played"])
ticks_df = parser.parse_ticks(["X", "Y"])
```
#### NodeJS
```JavaScript
var {parseEvent, parseTicks} = require('@laihoe/demoparser2');

let event_json = parseEvent("path_to_demo.dem", "player_death", ["X", "Y"], ["total_rounds_played"])
let ticks_json = parseTicks("path_to_demo.dem", ["X", "Y"])
```

### Examples in Python and JavaScript
- [Examples](./examples)

### Scuffed Documentaion
- [Documentaion](./documentation)


### Performance
The benchmark finds the coordinates of all player deaths. Code: [Benchmark](./benchmark)  

Benchmarked 50 demos containing a mix of (MM, Faceit, HLTV) demos. Total size: 4.6GB
|device|CPU cores|total time|MB/s|
| :-------------------:| :-----------------------------------  | :----------------------------------- |:----------------------------------- |
gaming PC|12|6.14s|749|
ThinkPad T14 gen 2|4|14.00s|328|

Gaming pc specs: CPU: Ryzen 5900x, SSD: Samsung 980 pro NVME  
Thinkpad specs: CPU: i5-1335g7, SSD: Toshiba XG6 NVME  
Both devices run Ubuntu 20.04  
Python/JS are rougly as fast.

### List of fields the parser supports:

#### Player data
|         Name          | "Real" name                                                                                                                               |
| :-------------------: | :----------------------------------- |
| X |  m_vec + m_cell |
| Y |  m_vec + m_cell |
| Z |  m_vec + m_cell |
| health  | m_iHealth |
| score | m_iScore |
| mvps| m_iMVPs |
| is_alive | m_bPawnIsAlive |
| balance | m_iAccount |
| inventory| _ |
| inventory_as_ids  | - |
| life_state  | m_lifeState |
| pitch  | m_angEyeAngles[0] |
| yaw  | m_angEyeAngles[1] |
| is_auto_muted | m_bHasCommunicationAbuseMute |
| crosshair_code | m_szCrosshairCodes |
| pending_team_num | m_iPendingTeamNum |
| player_color | m_iCompTeammateColor |
| ever_played_on_team | m_bEverPlayedOnTeam |
| is_coach_team | m_iCoachingTeam |
| rank | m_iCompetitiveRanking |
| rank_if_win| m_iCompetitiveRankingPredicted_Win |
| rank_if_loss| m_iCompetitiveRankingPredicted_Loss |
| rank_if_tie| m_iCompetitiveRankingPredicted_Tie |
| comp_wins | m_iCompetitiveWins |
| comp_rank_type | m_iCompetitiveRankType |
| is_controlling_bot | m_bControllingBot |
| has_controlled_bot_this_round | m_bHasControlledBotThisRound |
| can_control_bot | m_bCanControlObservedBot |
| has_defuser | m_bPawnHasDefuser |
| has_helmet | m_bPawnHasHelmet |
| spawn_time | m_iPawnLifetimeStart |
| death_time | m_iPawnLifetimeEnd |
| game_time | net_tick |
| is_connected | m_iConnected |
| player_name | m_iszPlayerName |
| player_steamid | m_steamID |
| fov | m_iDesiredFOV |
| start_balance | m_iStartAccount |
| total_cash_spent | m_iTotalCashSpent |
| cash_spent_this_round | m_iCashSpentThisRound |
| music_kit_id | m_unMusicID |
| leader_honors | m_nPersonaDataPublicCommendsLeader |
| teacher_honors | m_nPersonaDataPublicCommendsTeacher |
| friendly_honors | m_nPersonaDataPublicCommendsFriendly |
| ping | m_iPing |
| move_collide  | m_MoveCollide |
| move_type  | m_MoveType |
| team_num  | m_iTeamNum |
| active_weapon  | m_hActiveWeapon |
| looking_at_weapon  | m_bIsLookingAtWeapon |
| holding_look_at_weapon  | m_bIsHoldingLookAtWeapon |
| next_attack_time  | m_flNextAttack |
| duck_time_ms  | m_nDuckTimeMsecs |
| max_speed  | m_flMaxspeed |
| max_fall_velo  | m_flMaxFallVelocity |
| duck_amount  | m_flDuckAmount |
| duck_speed  | m_flDuckSpeed |
| duck_overrdie  | m_bDuckOverride |
| old_jump_pressed  | m_bOldJumpPressed |
| jump_until  | m_flJumpUntil |
| jump_velo  | m_flJumpVel |
| fall_velo  | m_flFallVelocity |
| in_crouch  | m_bInCrouch |
| crouch_state  | m_nCrouchState |
| ducked  | m_bDucked |
| ducking  | m_bDucking |
| in_duck_jump  | m_bInDuckJump |
| allow_auto_movement  | m_bAllowAutoMovement |
| jump_time_ms  | m_nJumpTimeMsecs |
| last_duck_time  | m_flLastDuckTime |
| is_rescuing  | m_bIsRescuing |
| weapon_purchases_this_match  | m_iWeaponPurchasesThisMatch |
| weapon_purchases_this_round  | m_iWeaponPurchasesThisRound |
| spotted  | m_bSpotted |
| approximate_spotted_by  | m_bSpottedByMask |
| time_last_injury  | m_flTimeOfLastInjury |
| direction_last_injury  | m_nRelativeDirectionOfLastInjury |
| player_state  | m_iPlayerState |
| passive_items  | m_passiveItems |
| is_scoped  | m_bIsScoped |
| is_walking  | m_bIsWalking |
| resume_zoom  | m_bResumeZoom |
| is_defusing  | m_bIsDefusing |
| is_grabbing_hostage  | m_bIsGrabbingHostage |
| blocking_use_in_progess  | m_iBlockingUseActionInProgress |
| molotov_damage_time  | m_fMolotovDamageTime |
| moved_since_spawn  | m_bHasMovedSinceSpawn |
| in_bomb_zone  | m_bInBombZone |
| in_buy_zone  | m_bInBuyZone |
| in_no_defuse_area  | m_bInNoDefuseArea |
| killed_by_taser  | m_bKilledByTaser |
| move_state  | m_iMoveState |
| which_bomb_zone  | m_nWhichBombZone |
| in_hostage_rescue_zone  | m_bInHostageRescueZone |
| stamina  | m_flStamina |
| direction  | m_iDirection |
| shots_fired  | m_iShotsFired |
| armor_value  | m_ArmorValue |
| velo_modifier  | m_flVelocityModifier |
| ground_accel_linear_frac_last_time  | m_flGroundAccelLinearFracLastTime |
| flash_duration  | m_flFlashDuration |
| flash_max_alpha  | m_flFlashMaxAlpha |
| wait_for_no_attack  | m_bWaitForNoAttack |
| last_place_name  | m_szLastPlaceName |
| is_strafing  | m_bStrafing |
| round_start_equip_value  | m_unRoundStartEquipmentValue |
| current_equip_value  | m_unCurrentEquipmentValue |
| velocity  | - |
| velocity_X  | - |
| velocity_Y  | - |
| velocity_Z  | - |
| agent_skin  | - |
| user_id  | - |
| entity_id  | - |
| is_airborne  | m_hGroundEntity |
| aim_punch_angle  | CCSPlayerPawn.m_aimPunchAngle |
| aim_punch_angle_vel  | CCSPlayerPawn.m_aimPunchAngleVel |

#### Buttons 
True/Flase if player is pressing button.
|         Name          | Real name                                                                                                                               |
| :-------------------: | :----------------------------------- |
|FORWARD|m_nButtonDownMaskPrev|
|LEFT|m_nButtonDownMaskPrev|
|RIGHT |m_nButtonDownMaskPrev|
|BACK|m_nButtonDownMaskPrev|
|FIRE|m_nButtonDownMaskPrev|
|RIGHTCLICK |m_nButtonDownMaskPrev|
|RELOAD |m_nButtonDownMaskPrev|
|INSPECT|m_nButtonDownMaskPrev|
|USE|m_nButtonDownMaskPrev|
|ZOOM |m_nButtonDownMaskPrev|
|SCOREBOARD|m_nButtonDownMaskPrev|
|WALK|m_nButtonDownMaskPrev|
|button|m_nButtonDownMaskPrev|

(buttons is the real value of m_nButtonDownMaskPrev and the others are derived from it)

#### Game State
|         Name          | Real name                                                                                                                               |
| :-------------------: | :----------------------------------- |
|team_rounds_total|m_iScore|
|team_surrendered|m_bSurrendered|
|team_name|m_szTeamname|
|team_score_overtime|m_scoreOvertime|
|team_match_stat|m_szTeamMatchStat|
|team_num_map_victories|m_numMapVictories|
|team_score_first_half| m_scoreFirstHalf |
| team_score_second_half | m_scoreSecondHalf |
| team_clan_name  | m_szClanTeamname |
| is_freeze_period | m_bFreezePeriod |
| is_warmup_period | m_bWarmupPeriod  |
| warmup_period_end | m_fWarmupPeriodEnd  |
| warmup_period_start | m_fWarmupPeriodStart  |
| is_terrorist_timeout | m_bTerroristTimeOutActive  |
| is_ct_timeout | m_bCTTimeOutActive  |
| terrorist_timeout_remaining | m_flTerroristTimeOutRemaining  |
| ct_timeout_remaining | m_flCTTimeOutRemaining  |
| num_terrorist_timeouts | m_nTerroristTimeOuts  |
| num_ct_timeouts | m_nCTTimeOuts  |
| is_technical_timeout | m_bTechnicalTimeOut  |
| is_waiting_for_resume | m_bMatchWaitingForResume  |
| match_start_time | m_fMatchStartTime  |
| round_start_time | m_fRoundStartTime  |
| restart_round_time | m_flRestartRoundTime  |
| is_game_restart | m_bGameRestart  |
| game_start_time | m_flGameStartTime  |
| time_until_next_phase_start | m_timeUntilNextPhaseStarts  |
| game_phase | m_gamePhase  |
| total_rounds_played | m_totalRoundsPlayed  |
| rounds_played_this_phase | m_nRoundsPlayedThisPhase  |
| hostages_remaining | m_iHostagesRemaining  |
| any_hostages_reached | m_bAnyHostageReached  |
| has_bombites | m_bMapHasBombTarget  |
| has_rescue_zone | m_bMapHasRescueZone  |
| has_buy_zone | m_bMapHasBuyZone  |
| is_matchmaking | m_bIsQueuedMatchmaking  |
| match_making_mode | m_nQueuedMatchmakingMode  |
| is_valve_dedicated_server | m_bIsValveDS  |
| gungame_prog_weap_ct | m_iNumGunGameProgressiveWeaponsCT  |
| gungame_prog_weap_t | m_iNumGunGameProgressiveWeaponsT  |
| spectator_slot_count | m_iSpectatorSlotCount  |
| is_match_started | m_bHasMatchStarted  |
| n_best_of_maps | m_numBestOfMaps  |
| is_bomb_dropped | m_bBombDropped  |
| is_bomb_planted | m_bBombPlanted  |
| round_win_status | m_iRoundWinStatus  |
| round_win_reason | m_eRoundWinReason  |
| terrorist_cant_buy | m_bTCantBuy  |
| ct_cant_buy | m_bCTCantBuy  |
| ct_losing_streak | m_iNumConsecutiveCTLoses  |
| t_losing_streak | m_iNumConsecutiveTerroristLoses  |
| survival_start_time | m_flSurvivalStartTime  |
| round_in_progress | m_bRoundInProgress  |

#### Weapon
|         Name          | Real name                                                                                                                               |
| :-------------------: | :----------------------------------- |
| active_weapon_name  |  m_iItemDefinitionIndex + lookup |
| active_weapon_skin  |  m_iRawValue32 + lookup |
| active_weapon_ammo  |  m_iClip1 |
|active_weapon_original_owner| m_OriginalOwnerXuidLow + m_OriginalOwnerXuidHigh|
| total_ammo_left  |  m_pReserveAmmo |
| item_def_idx  |  m_iItemDefinitionIndex |
| weapon_quality  |  m_iEntityQuality |
| entity_lvl  |  m_iEntityLevel |
| item_id_high  |  m_iItemIDHigh |
| item_id_low  |  m_iItemIDLow |
| item_account_id  |  m_iAccountID |
| inventory_position  |  m_iInventoryPosition |
| is_initialized  |  m_bInitialized |
| econ_item_attribute_def_idx  | m_iAttributeDefinitionIndex |
| initial_value  | m_flInitialValue |
| refundable_currency  | m_nRefundableCurrency |
| set_bonus | m_bSetBonus |
| custom_name  |  m_szCustomName |
| orig_owner_xuid_low  |  m_OriginalOwnerXuidLow |
| orig_owner_xuid_high |  m_OriginalOwnerXuidHigh |
| fall_back_paint_kit  |  m_nFallbackPaintKit |
| fall_back_seed |  m_nFallbackSeed |
| fall_back_wear |  m_flFallbackWear |
| fall_back_stat_track |  m_nFallbackStatTrak |
| m_iState |  m_iState |
| fire_seq_start_time  |  m_flFireSequenceStartTime |
| fire_seq_start_time_change  |  m_nFireSequenceStartTimeChange |
| is_player_fire_event_primary |   m_bPlayerFireEventIsPrimary |
| weapon_mode |  m_weaponMode |
| accuracy_penalty |  m_fAccuracyPenalty |
| i_recoil_idx |  m_iRecoilIndex |
| fl_recoil_idx |  m_flRecoilIndex |
| is_burst_mode |  m_bBurstMode |
| post_pone_fire_ready_time |  m_flPostponeFireReadyTime |
| is_in_reload |  m_bInReload |
| reload_visually_complete |  m_bReloadVisuallyComplete |
| dropped_at_time |  m_flDroppedAtTime |
| is_hauled_back |  m_bIsHauledBack |
| is_silencer_on |  m_bSilencerOn |
| time_silencer_switch_complete |  m_flTimeSilencerSwitchComplete |
| orig_team_number |  m_iOriginalTeamNumber |
| prev_owner |  m_hPrevOwner |
| last_shot_time |  m_fLastShotTime |
| iron_sight_mode |  m_iIronSightMode |
| num_empty_attacks |  m_iNumEmptyAttacks |
| zoom_lvl |  m_zoomLevel |
| burst_shots_remaining |  m_iBurstShotsRemaining |
| needs_bolt_action |  m_bNeedsBoltAction |
| next_primary_attack_tick |  m_nNextPrimaryAttackTick |
| next_primary_attack_tick_ratio |  m_flNextPrimaryAttackTickRatio |
| next_secondary_attack_tick  |  m_nNextSecondaryAttackTick |
| next_secondary_attack_tick_ratio |  m_flNextSecondaryAttackTickRatio |
| weapon_float |  m_iRawValue32 |
| weapon_paint_seed |  m_iRawValue32 |
| weapon_stickers |  m_iRawValue32 |



#### usercommands
|         Name          | Real name                                                                                                                               |
| :-------------------: | :----------------------------------- |
| usercmd_viewangle_x | - |
| usercmd_viewangle_y | - |
| usercmd_viewangle_z | - |
| usercmd_buttonstate_1 | - |
| usercmd_buttonstate_2 | - |
| usercmd_buttonstate_3 | - |
| usercmd_consumed_server_angle_changes | - |
| usercmd_forward_move | - |
| usercmd_left_move | - |
| usercmd_impulse | - |
| usercmd_mouse_dx | - |
| usercmd_mouse_dy | - |
| usercmd_left_hand_desired | - |
| usercmd_weapon_select | - |
| usercmd_input_history | - |

#### Aggregate stats (updates one time per round)
|         Name          | Real name                                                                                                                               |
| :-------------------: | :----------------------------------- |
| kills_total | m_iKills |
| deaths_total | m_iDeaths |
| assists_total | m_iAssists |
| alive_time_total | m_iLiveTime |
| headshot_kills_total | m_iHeadShotKills |
| ace_rounds_total | m_iEnemy5Ks |
| 4k_rounds_total | m_iEnemy4Ks |
| 3k_rounds_total | m_iEnemy3Ks |
| damage_total | m_iDamage |
| objective_total | m_iObjective |
| utility_damage_total | m_iUtilityDamage |
| enemies_flashed_total | m_iEnemiesFlashed |
| equipment_value_total | m_iEquipmentValue |
| money_saved_total | m_iMoneySaved |
| kill_reward_total | m_iKillReward |
| cash_earned_total | m_iCashEarned |

## Other parsers
Go: https://github.com/markus-wa/demoinfocs-golang  
C#: https://github.com/saul/demofile-net  
Python: https://github.com/pnxenopoulos/awpy  
Java: https://github.com/skadistats/clarity  

## Acknowledgements
Without Dotabuff's dota 2 parser "manta" this would not have been possible. Check it out: https://github.com/dotabuff/manta

The dota 2 demo format is very similar to CS2 demo format with only a few minor changes.

Thank you to [Clarity](https://github.com/skadistats/clarity/) for an elegant implementation of sendtables  

Thank you to [Dandrews](https://github.com/DandrewsDev/CS2VoiceData) for figuring out voice data extraction  
