from unittest import TestCase
from demoparser2 import DemoParser
import pandas as pd
from pandas.testing import assert_frame_equal
import json
import unittest
import pickle

ALL_PROPS = ['active_weapon_skin', 'FORWARD', 'LEFT', 'RIGHT', 'BACK', 'FIRE', 'RIGHTCLICK', 'RELOAD', 'INSPECT', 'USE', 'ZOOM', 
'SCOREBOARD', 'WALK', 'pitch', 'yaw', 'game_time', 'rank', 'rank_if_win', 'rank_if_loss', 'rank_if_tie', 'mvps', 
'active_weapon_original_owner', 'buttons', 'team_surrendered', 'team_rounds_total', 'team_name', 'team_score_overtime', 
'team_match_stat', 'team_num_map_victories', 'team_score_first_half', 'team_score_second_half', 'team_clan_name', 
'is_freeze_period', 'is_warmup_period', 'warmup_period_end', 'warmup_period_start', 'is_terrorist_timeout', 'is_ct_timeout', 
'terrorist_timeout_remaining', 'ct_timeout_remaining', 'num_terrorist_timeouts', 'num_ct_timeouts', 'is_technical_timeout', 
'is_waiting_for_resume', 'match_start_time', 'round_start_time', 'restart_round_time', 'is_game_restart?', 'game_start_time', 
'time_until_next_phase_start', 'game_phase', 'total_rounds_played', 'rounds_played_this_phase', 'hostages_remaining', 
'any_hostages_reached', 'has_bombites', 'has_rescue_zone', 'has_buy_zone', 'is_matchmaking', 'match_making_mode', 
'is_valve_dedicated_server', 'gungame_prog_weap_ct', 'gungame_prog_weap_t', 'spectator_slot_count', 'is_match_started',
'n_best_of_maps', 'is_bomb_dropped', 'is_bomb_planed', 'round_win_status', 'round_win_reason', 'terrorist_cant_buy', 
'ct_cant_buy', 'num_player_alive_ct', 'num_player_alive_t', 'ct_losing_streak', 't_losing_streak', 'survival_start_time',
'round_in_progress', 'i_bomb_site?', 'is_auto_muted', 'crosshair_code', 'pending_team_num', 'player_color', 'ever_played_on_team', 
'clan_name', 'is_coach_team', 'comp_wins', 'comp_rank_type', 'is_controlling_bot', 'has_controlled_bot_this_round', 'can_control_bot', 
'is_alive', 'armor', 'has_defuser', 'has_helmet', 'spawn_time', 'death_time', 'score', 'is_connected', 'player_name', 'player_steamid', 
'fov', 'balance', 'start_balance', 'total_cash_spent', 'cash_spent_this_round', 'music_kit_id', 'leader_honors', 'teacher_honors', 
'friendly_honors', 'kills_this_round', 'deaths_this_round', 'assists_this_round', 'alive_time_this_round', 'headshot_kills_this_round',
'damage_this_round', 'objective_this_round', 'utility_damage_this_round', 'enemies_flashed_this_round', 'equipment_value_this_round',
'money_saved_this_round', 'kill_reward_this_round', 'cash_earned_this_round', 'kills_total', 'deaths_total', 'assists_total',
'alive_time_total', 'headshot_kills_total', 'ace_rounds_total', '4k_rounds_total', '3k_rounds_total', 'damage_total', 
'objective_total', 'utility_damage_total', 'enemies_flashed_total', 'equipment_value_total', 'money_saved_total',
'kill_reward_total', 'cash_earned_total', 'ping', 'move_collide', 'move_type', 'team_num', 'active_weapon',
'looking_at_weapon', 'holding_look_at_weapon', 'next_attack_time', 'duck_time_ms', 'max_speed', 'max_fall_velo',
'duck_amount', 'duck_speed', 'duck_overrdie', 'old_jump_pressed', 'jump_until', 'jump_velo', 'fall_velo', 'in_crouch',
'crouch_state', 'ducked', 'ducking', 'in_duck_jump', 'allow_auto_movement', 'jump_time_ms', 'last_duck_time', 'is_rescuing',
'weapon_purchases_this_match', 'weapon_purchases_this_round', 'spotted', 'spotted_mask', 'time_last_injury',
'direction_last_injury', 'player_state', 'passive_items', 'is_scoped', 'is_walking', 'resume_zoom', 'is_defusing', 
'is_grabbing_hostage', 'blocking_use_in_progess', 'molotov_damage_time', 'moved_since_spawn', 'in_bomb_zone', 
'in_buy_zone', 'in_no_defuse_area', 'killed_by_taser', 'move_state', 'which_bomb_zone', 'in_hostage_rescue_zone',
'stamina', 'direction', 'shots_fired', 'armor_value', 'velo_modifier', 'ground_accel_linear_frac_last_time',
'flash_duration', 'flash_max_alpha', 'wait_for_no_attack', 'last_place_name', 'is_strafing', 'round_start_equip_value',
'current_equip_value', 'time', 'health', 'life_state', 'X', 'Y', 'Z', 'active_weapon_name', 'active_weapon_ammo',
'total_ammo_left', 'item_def_idx', 'weapon_quality', 'entity_lvl', 'item_id_high', 'item_id_low', 'item_account_id',
'inventory_position', 'is_initialized', 'econ_item_attribute_def_idx', 'econ_raw_val_32', 'initial_value',
'refundable_currency', 'set_bonus', 'custom_name', 'orig_owner_xuid_low', 'orig_owner_xuid_high', 'fall_back_paint_kit', 
'fall_back_seed', 'fall_back_wear', 'fall_back_stat_track', 'm_iState', 'fire_seq_start_time',
'fire_seq_start_time_change', 'is_player_fire_event_primary', 'weapon_mode', 'accuracy_penalty', 'i_recoil_idx',
'fl_recoil_idx', 'is_burst_mode', 'post_pone_fire_ready_time', 'is_in_reload', 'reload_visually_complete',
'dropped_at_time', 'is_hauled_back', 'is_silencer_on', 'time_silencer_switch_complete', 'orig_team_number',
'prev_owner', 'last_shot_time', 'iron_sight_mode', 'num_empty_attacks', 'zoom_lvl', 'burst_shots_remaining',
'needs_bolt_action', 'next_primary_attack_tick', 'next_primary_attack_tick_ratio', 'next_secondary_attack_tick',
'next_secondary_attack_tick_ratio', 'inventory']




def convert_same_dtypes(df, correct_df):
    for (dtyp, col) in zip(correct_df.dtypes, correct_df.columns):
        df[col] = df[col].astype(dtyp)
    return df


class ParserTest(TestCase):
    def test_parse_event(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_death", player=["X"], other=["total_rounds_played", "team_rounds_total"])
        df_correct = pd.read_json("tests/data/python/parse_event.json")
        df = convert_same_dtypes(df, df_correct)
        assert_frame_equal(df, df_correct)

    def test_all_event_all_props(self):
        parser = DemoParser("tests/data/test.dem")
        event_list = parser.parse_events(["all"], player=ALL_PROPS)
        with open('tests/data/python/all_events_all_props.pkl', 'rb') as fp:
            event_list_correct = pickle.load(fp)
            event_list.sort(key = lambda x: x[0])
            event_list_correct.sort(key = lambda x: x[0])
            for (event, event_correct) in zip(event_list, event_list_correct):
                assert_frame_equal(event[1], event_correct[1])
                self.assertEqual(event[0], event_correct[0])
    
    def test_all_ticks_props(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(ALL_PROPS, ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/python/all_tick_props.parquet")
        assert_frame_equal(df, df_correct)

    def test_header(self):
        parser = DemoParser("tests/data/test.dem")
        header = parser.parse_header()
        with open("tests/data/python/header.json", "r") as f:
            d = json.load(f)
            self.assertEqual(d, header)

    def test_list_game_events(self):
        parser = DemoParser("tests/data/test.dem")
        game_events = list(parser.list_game_events())
        with open("tests/data/python/list_game_events.json", "r") as f:
            d = json.load(f)
            self.assertEqual(sorted(d), sorted(game_events))

    def test_parse_grenades(self):
        parser = DemoParser("tests/data/test.dem")
        df_correct = pd.read_parquet("tests/data/python/grenades.parquet")
        df = convert_same_dtypes(parser.parse_grenades(), df_correct)
        assert_frame_equal(df, df_correct)

    def test_custom_even_rank_update(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("rank_update")
        df_correct = convert_same_dtypes(df, pd.read_csv("tests/data/python/rank_update.csv"))
        assert_frame_equal(df, df_correct)
    
    def test_game_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["time_last_injury", "game_time"], ticks=[90942])
        df = df[df["steamid"] == 76561198054530073]
        time_last_injury = df.iloc[0]["time_last_injury"]
        game_time = df.iloc[0]["time_last_injury"]
        # This can also be slightly off if is sub tick
        self.assertEqual(time_last_injury, game_time)



if __name__ == '__main__':
    unittest.main()