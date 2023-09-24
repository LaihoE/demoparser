from unittest import TestCase
from demoparser2 import DemoParser
import pandas as pd
from pandas.testing import assert_frame_equal
import json
import unittest
import pickle


def convert_same_dtypes(df, correct_df):
    for (dtyp, col) in zip(correct_df.dtypes, correct_df.columns):
        df[col] = df[col].astype(dtyp)
    return df


class ParserTest(TestCase):
    def test_parse_event_with_props(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_death", player=["X", "Y"], other=["game_time", "total_rounds_played"])
        df_correct = pd.read_parquet("tests/data/event_with_props.parquet")
        df = convert_same_dtypes(df, df_correct)
        assert_frame_equal(df, df_correct)

    def test_parse_events_with_props(self):
        parser = DemoParser("tests/data/test.dem")
        event_list = parser.parse_events(["all"], player=["X", "Y"], other=["game_time", "total_rounds_played"])
        with open('tests/data/events_with_props.pickle', 'rb') as fp:
            event_list_correct = pickle.load(fp)
            event_list.sort(key = lambda x: x[0])
            event_list_correct.sort(key = lambda x: x[0])
            for (event, event_correct) in zip(event_list, event_list_correct):
                assert_frame_equal(event[1], event_correct[1])
                self.assertEqual(event[0], event_correct[0])
    
    def test_active_weapon_skin(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["active_weapon_skin"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/active_weapon_skin.parquet")
        assert_frame_equal(df, df_correct)

    def test_FORWARD(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["FORWARD"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/FORWARD.parquet")
        assert_frame_equal(df, df_correct)

    def test_LEFT(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["LEFT"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/LEFT.parquet")
        assert_frame_equal(df, df_correct)

    def test_RIGHT(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["RIGHT"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/RIGHT.parquet")
        assert_frame_equal(df, df_correct)

    def test_BACK(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["BACK"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/BACK.parquet")
        assert_frame_equal(df, df_correct)

    def test_FIRE(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["FIRE"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/FIRE.parquet")
        assert_frame_equal(df, df_correct)

    def test_RIGHTCLICK(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["RIGHTCLICK"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/RIGHTCLICK.parquet")
        assert_frame_equal(df, df_correct)

    def test_RELOAD(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["RELOAD"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/RELOAD.parquet")
        assert_frame_equal(df, df_correct)

    def test_INSPECT(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["INSPECT"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/INSPECT.parquet")
        assert_frame_equal(df, df_correct)

    def test_USE(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["USE"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/USE.parquet")
        assert_frame_equal(df, df_correct)

    def test_ZOOM(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ZOOM"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ZOOM.parquet")
        assert_frame_equal(df, df_correct)

    def test_SCOREBOARD(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["SCOREBOARD"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/SCOREBOARD.parquet")
        assert_frame_equal(df, df_correct)

    def test_WALK(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["WALK"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/WALK.parquet")
        assert_frame_equal(df, df_correct)

    def test_pitch(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["pitch"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/pitch.parquet")
        assert_frame_equal(df, df_correct)

    def test_yaw(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["yaw"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/yaw.parquet")
        assert_frame_equal(df, df_correct)

    def test_game_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["game_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/game_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_rank(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["rank"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/rank.parquet")
        assert_frame_equal(df, df_correct)

    def test_rank_if_win(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["rank_if_win"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/rank_if_win.parquet")
        assert_frame_equal(df, df_correct)

    def test_rank_if_loss(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["rank_if_loss"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/rank_if_loss.parquet")
        assert_frame_equal(df, df_correct)

    def test_rank_if_tie(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["rank_if_tie"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/rank_if_tie.parquet")
        assert_frame_equal(df, df_correct)

    def test_mvps(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["mvps"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/mvps.parquet")
        assert_frame_equal(df, df_correct)

    def test_active_weapon_original_owner(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["active_weapon_original_owner"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/active_weapon_original_owner.parquet")
        assert_frame_equal(df, df_correct)

    def test_buttons(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["buttons"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/buttons.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_surrendered(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_surrendered"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_surrendered.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_rounds_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_rounds_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_rounds_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_score_overtime(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_score_overtime"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_score_overtime.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_match_stat(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_match_stat"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_match_stat.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_num_map_victories(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_num_map_victories"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_num_map_victories.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_score_first_half(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_score_first_half"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_score_first_half.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_score_second_half(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_score_second_half"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_score_second_half.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_clan_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_clan_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_clan_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_freeze_period(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_freeze_period"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_freeze_period.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_warmup_period(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_warmup_period"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_warmup_period.parquet")
        assert_frame_equal(df, df_correct)

    def test_warmup_period_end(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["warmup_period_end"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/warmup_period_end.parquet")
        assert_frame_equal(df, df_correct)

    def test_warmup_period_start(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["warmup_period_start"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/warmup_period_start.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_terrorist_timeout(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_terrorist_timeout"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_terrorist_timeout.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_ct_timeout(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_ct_timeout"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_ct_timeout.parquet")
        assert_frame_equal(df, df_correct)

    def test_terrorist_timeout_remaining(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["terrorist_timeout_remaining"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/terrorist_timeout_remaining.parquet")
        assert_frame_equal(df, df_correct)

    def test_ct_timeout_remaining(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ct_timeout_remaining"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ct_timeout_remaining.parquet")
        assert_frame_equal(df, df_correct)

    def test_num_terrorist_timeouts(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["num_terrorist_timeouts"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/num_terrorist_timeouts.parquet")
        assert_frame_equal(df, df_correct)

    def test_num_ct_timeouts(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["num_ct_timeouts"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/num_ct_timeouts.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_technical_timeout(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_technical_timeout"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_technical_timeout.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_waiting_for_resume(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_waiting_for_resume"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_waiting_for_resume.parquet")
        assert_frame_equal(df, df_correct)

    def test_match_start_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["match_start_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/match_start_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_start_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["round_start_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/round_start_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_restart_round_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["restart_round_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/restart_round_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_game_restart(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_game_restart?"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_game_restart?.parquet")
        assert_frame_equal(df, df_correct)

    def test_game_start_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["game_start_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/game_start_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_time_until_next_phase_start(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["time_until_next_phase_start"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/time_until_next_phase_start.parquet")
        assert_frame_equal(df, df_correct)

    def test_game_phase(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["game_phase"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/game_phase.parquet")
        assert_frame_equal(df, df_correct)

    def test_total_rounds_played(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["total_rounds_played"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/total_rounds_played.parquet")
        assert_frame_equal(df, df_correct)

    def test_rounds_played_this_phase(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["rounds_played_this_phase"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/rounds_played_this_phase.parquet")
        assert_frame_equal(df, df_correct)

    def test_hostages_remaining(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["hostages_remaining"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/hostages_remaining.parquet")
        assert_frame_equal(df, df_correct)

    def test_any_hostages_reached(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["any_hostages_reached"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/any_hostages_reached.parquet")
        assert_frame_equal(df, df_correct)

    def test_has_bombites(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["has_bombites"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/has_bombites.parquet")
        assert_frame_equal(df, df_correct)

    def test_has_rescue_zone(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["has_rescue_zone"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/has_rescue_zone.parquet")
        assert_frame_equal(df, df_correct)

    def test_has_buy_zone(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["has_buy_zone"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/has_buy_zone.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_matchmaking(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_matchmaking"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_matchmaking.parquet")
        assert_frame_equal(df, df_correct)

    def test_match_making_mode(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["match_making_mode"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/match_making_mode.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_valve_dedicated_server(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_valve_dedicated_server"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_valve_dedicated_server.parquet")
        assert_frame_equal(df, df_correct)

    def test_gungame_prog_weap_ct(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["gungame_prog_weap_ct"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/gungame_prog_weap_ct.parquet")
        assert_frame_equal(df, df_correct)

    def test_gungame_prog_weap_t(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["gungame_prog_weap_t"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/gungame_prog_weap_t.parquet")
        assert_frame_equal(df, df_correct)

    def test_spectator_slot_count(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["spectator_slot_count"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/spectator_slot_count.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_match_started(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_match_started"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_match_started.parquet")
        assert_frame_equal(df, df_correct)

    def test_n_best_of_maps(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["n_best_of_maps"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/n_best_of_maps.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_bomb_dropped(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_bomb_dropped"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_bomb_dropped.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_bomb_planed(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_bomb_planed"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_bomb_planed.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_win_status(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["round_win_status"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/round_win_status.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_win_reason(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["round_win_reason"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/round_win_reason.parquet")
        assert_frame_equal(df, df_correct)

    def test_terrorist_cant_buy(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["terrorist_cant_buy"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/terrorist_cant_buy.parquet")
        assert_frame_equal(df, df_correct)

    def test_ct_cant_buy(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ct_cant_buy"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ct_cant_buy.parquet")
        assert_frame_equal(df, df_correct)

    def test_num_player_alive_ct(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["num_player_alive_ct"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/num_player_alive_ct.parquet")
        assert_frame_equal(df, df_correct)

    def test_num_player_alive_t(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["num_player_alive_t"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/num_player_alive_t.parquet")
        assert_frame_equal(df, df_correct)

    def test_ct_losing_streak(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ct_losing_streak"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ct_losing_streak.parquet")
        assert_frame_equal(df, df_correct)

    def test_t_losing_streak(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["t_losing_streak"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/t_losing_streak.parquet")
        assert_frame_equal(df, df_correct)

    def test_survival_start_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["survival_start_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/survival_start_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_in_progress(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["round_in_progress"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/round_in_progress.parquet")
        assert_frame_equal(df, df_correct)

    def test_i_bomb_site(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["i_bomb_site?"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/i_bomb_site?.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_auto_muted(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_auto_muted"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_auto_muted.parquet")
        assert_frame_equal(df, df_correct)

    def test_crosshair_code(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["crosshair_code"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/crosshair_code.parquet")
        assert_frame_equal(df, df_correct)

    def test_pending_team_num(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["pending_team_num"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/pending_team_num.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_color(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["player_color"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/player_color.parquet")
        assert_frame_equal(df, df_correct)

    def test_ever_played_on_team(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ever_played_on_team"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ever_played_on_team.parquet")
        assert_frame_equal(df, df_correct)

    def test_clan_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["clan_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/clan_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_coach_team(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_coach_team"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_coach_team.parquet")
        assert_frame_equal(df, df_correct)

    def test_comp_wins(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["comp_wins"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/comp_wins.parquet")
        assert_frame_equal(df, df_correct)

    def test_comp_rank_type(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["comp_rank_type"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/comp_rank_type.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_controlling_bot(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_controlling_bot"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_controlling_bot.parquet")
        assert_frame_equal(df, df_correct)

    def test_has_controlled_bot_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["has_controlled_bot_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/has_controlled_bot_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_can_control_bot(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["can_control_bot"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/can_control_bot.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_alive(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_alive"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_alive.parquet")
        assert_frame_equal(df, df_correct)

    def test_armor(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["armor"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/armor.parquet")
        assert_frame_equal(df, df_correct)

    def test_has_defuser(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["has_defuser"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/has_defuser.parquet")
        assert_frame_equal(df, df_correct)

    def test_has_helmet(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["has_helmet"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/has_helmet.parquet")
        assert_frame_equal(df, df_correct)

    def test_spawn_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["spawn_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/spawn_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_death_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["death_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/death_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_score(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["score"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/score.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_connected(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_connected"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_connected.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["player_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/player_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_steamid(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["player_steamid"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/player_steamid.parquet")
        assert_frame_equal(df, df_correct)

    def test_fov(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fov"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fov.parquet")
        assert_frame_equal(df, df_correct)

    def test_balance(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["balance"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/balance.parquet")
        assert_frame_equal(df, df_correct)

    def test_start_balance(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["start_balance"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/start_balance.parquet")
        assert_frame_equal(df, df_correct)

    def test_total_cash_spent(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["total_cash_spent"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/total_cash_spent.parquet")
        assert_frame_equal(df, df_correct)

    def test_cash_spent_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["cash_spent_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/cash_spent_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_music_kit_id(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["music_kit_id"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/music_kit_id.parquet")
        assert_frame_equal(df, df_correct)

    def test_leader_honors(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["leader_honors"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/leader_honors.parquet")
        assert_frame_equal(df, df_correct)

    def test_teacher_honors(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["teacher_honors"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/teacher_honors.parquet")
        assert_frame_equal(df, df_correct)

    def test_friendly_honors(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["friendly_honors"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/friendly_honors.parquet")
        assert_frame_equal(df, df_correct)

    def test_kills_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["kills_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/kills_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_deaths_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["deaths_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/deaths_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_assists_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["assists_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/assists_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_alive_time_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["alive_time_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/alive_time_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_headshot_kills_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["headshot_kills_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/headshot_kills_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_damage_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["damage_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/damage_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_objective_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["objective_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/objective_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_utility_damage_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["utility_damage_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/utility_damage_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_enemies_flashed_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["enemies_flashed_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/enemies_flashed_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_equipment_value_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["equipment_value_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/equipment_value_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_money_saved_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["money_saved_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/money_saved_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_kill_reward_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["kill_reward_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/kill_reward_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_cash_earned_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["cash_earned_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/cash_earned_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_kills_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["kills_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/kills_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_deaths_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["deaths_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/deaths_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_assists_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["assists_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/assists_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_alive_time_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["alive_time_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/alive_time_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_headshot_kills_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["headshot_kills_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/headshot_kills_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_ace_rounds_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ace_rounds_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ace_rounds_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_4k_rounds_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["4k_rounds_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/4k_rounds_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_3k_rounds_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["3k_rounds_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/3k_rounds_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_damage_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["damage_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/damage_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_objective_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["objective_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/objective_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_utility_damage_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["utility_damage_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/utility_damage_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_enemies_flashed_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["enemies_flashed_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/enemies_flashed_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_equipment_value_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["equipment_value_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/equipment_value_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_money_saved_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["money_saved_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/money_saved_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_kill_reward_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["kill_reward_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/kill_reward_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_cash_earned_total(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["cash_earned_total"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/cash_earned_total.parquet")
        assert_frame_equal(df, df_correct)

    def test_ping(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ping"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ping.parquet")
        assert_frame_equal(df, df_correct)

    def test_move_collide(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["move_collide"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/move_collide.parquet")
        assert_frame_equal(df, df_correct)

    def test_move_type(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["move_type"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/move_type.parquet")
        assert_frame_equal(df, df_correct)

    def test_team_num(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["team_num"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/team_num.parquet")
        assert_frame_equal(df, df_correct)

    def test_active_weapon(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["active_weapon"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/active_weapon.parquet")
        assert_frame_equal(df, df_correct)

    def test_looking_at_weapon(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["looking_at_weapon"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/looking_at_weapon.parquet")
        assert_frame_equal(df, df_correct)

    def test_holding_look_at_weapon(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["holding_look_at_weapon"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/holding_look_at_weapon.parquet")
        assert_frame_equal(df, df_correct)

    def test_next_attack_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["next_attack_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/next_attack_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_duck_time_ms(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["duck_time_ms"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/duck_time_ms.parquet")
        assert_frame_equal(df, df_correct)

    def test_max_speed(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["max_speed"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/max_speed.parquet")
        assert_frame_equal(df, df_correct)

    def test_max_fall_velo(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["max_fall_velo"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/max_fall_velo.parquet")
        assert_frame_equal(df, df_correct)

    def test_duck_amount(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["duck_amount"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/duck_amount.parquet")
        assert_frame_equal(df, df_correct)

    def test_duck_speed(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["duck_speed"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/duck_speed.parquet")
        assert_frame_equal(df, df_correct)

    def test_duck_overrdie(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["duck_overrdie"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/duck_overrdie.parquet")
        assert_frame_equal(df, df_correct)

    def test_old_jump_pressed(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["old_jump_pressed"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/old_jump_pressed.parquet")
        assert_frame_equal(df, df_correct)

    def test_jump_until(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["jump_until"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/jump_until.parquet")
        assert_frame_equal(df, df_correct)

    def test_jump_velo(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["jump_velo"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/jump_velo.parquet")
        assert_frame_equal(df, df_correct)

    def test_fall_velo(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fall_velo"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fall_velo.parquet")
        assert_frame_equal(df, df_correct)

    def test_in_crouch(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["in_crouch"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/in_crouch.parquet")
        assert_frame_equal(df, df_correct)

    def test_crouch_state(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["crouch_state"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/crouch_state.parquet")
        assert_frame_equal(df, df_correct)

    def test_ducked(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ducked"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ducked.parquet")
        assert_frame_equal(df, df_correct)

    def test_ducking(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ducking"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ducking.parquet")
        assert_frame_equal(df, df_correct)

    def test_in_duck_jump(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["in_duck_jump"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/in_duck_jump.parquet")
        assert_frame_equal(df, df_correct)

    def test_allow_auto_movement(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["allow_auto_movement"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/allow_auto_movement.parquet")
        assert_frame_equal(df, df_correct)

    def test_jump_time_ms(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["jump_time_ms"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/jump_time_ms.parquet")
        assert_frame_equal(df, df_correct)

    def test_last_duck_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["last_duck_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/last_duck_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_rescuing(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_rescuing"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_rescuing.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_purchases_this_match(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["weapon_purchases_this_match"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/weapon_purchases_this_match.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_purchases_this_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["weapon_purchases_this_round"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/weapon_purchases_this_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_spotted(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["spotted"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/spotted.parquet")
        assert_frame_equal(df, df_correct)

    def test_approximate_spotted_by(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["approximate_spotted_by"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/approximate_spotted_by.parquet")
        assert_frame_equal(df, df_correct)

    def test_time_last_injury(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["time_last_injury"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/time_last_injury.parquet")
        assert_frame_equal(df, df_correct)

    def test_direction_last_injury(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["direction_last_injury"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/direction_last_injury.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_state(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["player_state"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/player_state.parquet")
        assert_frame_equal(df, df_correct)

    def test_passive_items(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["passive_items"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/passive_items.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_scoped(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_scoped"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_scoped.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_walking(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_walking"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_walking.parquet")
        assert_frame_equal(df, df_correct)

    def test_resume_zoom(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["resume_zoom"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/resume_zoom.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_defusing(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_defusing"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_defusing.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_grabbing_hostage(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_grabbing_hostage"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_grabbing_hostage.parquet")
        assert_frame_equal(df, df_correct)

    def test_blocking_use_in_progess(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["blocking_use_in_progess"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/blocking_use_in_progess.parquet")
        assert_frame_equal(df, df_correct)

    def test_molotov_damage_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["molotov_damage_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/molotov_damage_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_moved_since_spawn(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["moved_since_spawn"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/moved_since_spawn.parquet")
        assert_frame_equal(df, df_correct)

    def test_in_bomb_zone(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["in_bomb_zone"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/in_bomb_zone.parquet")
        assert_frame_equal(df, df_correct)

    def test_in_buy_zone(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["in_buy_zone"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/in_buy_zone.parquet")
        assert_frame_equal(df, df_correct)

    def test_in_no_defuse_area(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["in_no_defuse_area"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/in_no_defuse_area.parquet")
        assert_frame_equal(df, df_correct)

    def test_killed_by_taser(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["killed_by_taser"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/killed_by_taser.parquet")
        assert_frame_equal(df, df_correct)

    def test_move_state(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["move_state"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/move_state.parquet")
        assert_frame_equal(df, df_correct)

    def test_which_bomb_zone(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["which_bomb_zone"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/which_bomb_zone.parquet")
        assert_frame_equal(df, df_correct)

    def test_in_hostage_rescue_zone(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["in_hostage_rescue_zone"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/in_hostage_rescue_zone.parquet")
        assert_frame_equal(df, df_correct)

    def test_stamina(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["stamina"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/stamina.parquet")
        assert_frame_equal(df, df_correct)

    def test_direction(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["direction"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/direction.parquet")
        assert_frame_equal(df, df_correct)

    def test_shots_fired(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["shots_fired"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/shots_fired.parquet")
        assert_frame_equal(df, df_correct)

    def test_armor_value(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["armor_value"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/armor_value.parquet")
        assert_frame_equal(df, df_correct)

    def test_velo_modifier(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["velo_modifier"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/velo_modifier.parquet")
        assert_frame_equal(df, df_correct)

    def test_ground_accel_linear_frac_last_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["ground_accel_linear_frac_last_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/ground_accel_linear_frac_last_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_flash_duration(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["flash_duration"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/flash_duration.parquet")
        assert_frame_equal(df, df_correct)

    def test_flash_max_alpha(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["flash_max_alpha"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/flash_max_alpha.parquet")
        assert_frame_equal(df, df_correct)

    def test_wait_for_no_attack(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["wait_for_no_attack"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/wait_for_no_attack.parquet")
        assert_frame_equal(df, df_correct)

    def test_last_place_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["last_place_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/last_place_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_strafing(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_strafing"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_strafing.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_start_equip_value(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["round_start_equip_value"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/round_start_equip_value.parquet")
        assert_frame_equal(df, df_correct)

    def test_current_equip_value(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["current_equip_value"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/current_equip_value.parquet")
        assert_frame_equal(df, df_correct)

    def test_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/time.parquet")
        assert_frame_equal(df, df_correct)

    def test_health(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["health"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/health.parquet")
        assert_frame_equal(df, df_correct)

    def test_life_state(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["life_state"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/life_state.parquet")
        assert_frame_equal(df, df_correct)

    def test_X(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["X"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/X.parquet")
        assert_frame_equal(df, df_correct)

    def test_Y(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["Y"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/Y.parquet")
        assert_frame_equal(df, df_correct)

    def test_Z(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["Z"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/Z.parquet")
        assert_frame_equal(df, df_correct)

    def test_active_weapon_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["active_weapon_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/active_weapon_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_active_weapon_ammo(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["active_weapon_ammo"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/active_weapon_ammo.parquet")
        assert_frame_equal(df, df_correct)

    def test_total_ammo_left(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["total_ammo_left"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/total_ammo_left.parquet")
        assert_frame_equal(df, df_correct)

    def test_item_def_idx(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["item_def_idx"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/item_def_idx.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_quality(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["weapon_quality"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/weapon_quality.parquet")
        assert_frame_equal(df, df_correct)

    def test_entity_lvl(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["entity_lvl"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/entity_lvl.parquet")
        assert_frame_equal(df, df_correct)

    def test_item_id_high(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["item_id_high"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/item_id_high.parquet")
        assert_frame_equal(df, df_correct)

    def test_item_id_low(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["item_id_low"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/item_id_low.parquet")
        assert_frame_equal(df, df_correct)

    def test_item_account_id(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["item_account_id"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/item_account_id.parquet")
        assert_frame_equal(df, df_correct)

    def test_inventory_position(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["inventory_position"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/inventory_position.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_initialized(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_initialized"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_initialized.parquet")
        assert_frame_equal(df, df_correct)

    def test_econ_item_attribute_def_idx(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["econ_item_attribute_def_idx"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/econ_item_attribute_def_idx.parquet")
        assert_frame_equal(df, df_correct)

    def test_econ_raw_val_32(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["econ_raw_val_32"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/econ_raw_val_32.parquet")
        assert_frame_equal(df, df_correct)

    def test_initial_value(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["initial_value"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/initial_value.parquet")
        assert_frame_equal(df, df_correct)

    def test_refundable_currency(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["refundable_currency"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/refundable_currency.parquet")
        assert_frame_equal(df, df_correct)

    def test_set_bonus(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["set_bonus"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/set_bonus.parquet")
        assert_frame_equal(df, df_correct)

    def test_custom_name(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["custom_name"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/custom_name.parquet")
        assert_frame_equal(df, df_correct)

    def test_orig_owner_xuid_low(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["orig_owner_xuid_low"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/orig_owner_xuid_low.parquet")
        assert_frame_equal(df, df_correct)

    def test_orig_owner_xuid_high(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["orig_owner_xuid_high"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/orig_owner_xuid_high.parquet")
        assert_frame_equal(df, df_correct)

    def test_fall_back_paint_kit(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fall_back_paint_kit"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fall_back_paint_kit.parquet")
        assert_frame_equal(df, df_correct)

    def test_fall_back_seed(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fall_back_seed"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fall_back_seed.parquet")
        assert_frame_equal(df, df_correct)

    def test_fall_back_wear(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fall_back_wear"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fall_back_wear.parquet")
        assert_frame_equal(df, df_correct)

    def test_fall_back_stat_track(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fall_back_stat_track"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fall_back_stat_track.parquet")
        assert_frame_equal(df, df_correct)

    def test_m_iState(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["m_iState"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/m_iState.parquet")
        assert_frame_equal(df, df_correct)

    def test_fire_seq_start_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fire_seq_start_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fire_seq_start_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_fire_seq_start_time_change(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fire_seq_start_time_change"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fire_seq_start_time_change.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_player_fire_event_primary(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_player_fire_event_primary"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_player_fire_event_primary.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_mode(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["weapon_mode"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/weapon_mode.parquet")
        assert_frame_equal(df, df_correct)

    def test_accuracy_penalty(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["accuracy_penalty"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/accuracy_penalty.parquet")
        assert_frame_equal(df, df_correct)

    def test_i_recoil_idx(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["i_recoil_idx"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/i_recoil_idx.parquet")
        assert_frame_equal(df, df_correct)

    def test_fl_recoil_idx(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["fl_recoil_idx"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/fl_recoil_idx.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_burst_mode(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_burst_mode"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_burst_mode.parquet")
        assert_frame_equal(df, df_correct)

    def test_post_pone_fire_ready_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["post_pone_fire_ready_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/post_pone_fire_ready_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_in_reload(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_in_reload"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_in_reload.parquet")
        assert_frame_equal(df, df_correct)

    def test_reload_visually_complete(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["reload_visually_complete"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/reload_visually_complete.parquet")
        assert_frame_equal(df, df_correct)

    def test_dropped_at_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["dropped_at_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/dropped_at_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_hauled_back(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_hauled_back"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_hauled_back.parquet")
        assert_frame_equal(df, df_correct)

    def test_is_silencer_on(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["is_silencer_on"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/is_silencer_on.parquet")
        assert_frame_equal(df, df_correct)

    def test_time_silencer_switch_complete(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["time_silencer_switch_complete"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/time_silencer_switch_complete.parquet")
        assert_frame_equal(df, df_correct)

    def test_orig_team_number(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["orig_team_number"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/orig_team_number.parquet")
        assert_frame_equal(df, df_correct)

    def test_prev_owner(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["prev_owner"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/prev_owner.parquet")
        assert_frame_equal(df, df_correct)

    def test_last_shot_time(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["last_shot_time"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/last_shot_time.parquet")
        assert_frame_equal(df, df_correct)

    def test_iron_sight_mode(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["iron_sight_mode"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/iron_sight_mode.parquet")
        assert_frame_equal(df, df_correct)

    def test_num_empty_attacks(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["num_empty_attacks"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/num_empty_attacks.parquet")
        assert_frame_equal(df, df_correct)

    def test_zoom_lvl(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["zoom_lvl"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/zoom_lvl.parquet")
        assert_frame_equal(df, df_correct)

    def test_burst_shots_remaining(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["burst_shots_remaining"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/burst_shots_remaining.parquet")
        assert_frame_equal(df, df_correct)

    def test_needs_bolt_action(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["needs_bolt_action"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/needs_bolt_action.parquet")
        assert_frame_equal(df, df_correct)

    def test_next_primary_attack_tick(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["next_primary_attack_tick"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/next_primary_attack_tick.parquet")
        assert_frame_equal(df, df_correct)

    def test_next_primary_attack_tick_ratio(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["next_primary_attack_tick_ratio"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/next_primary_attack_tick_ratio.parquet")
        assert_frame_equal(df, df_correct)

    def test_next_secondary_attack_tick(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["next_secondary_attack_tick"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/next_secondary_attack_tick.parquet")
        assert_frame_equal(df, df_correct)

    def test_next_secondary_attack_tick_ratio(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["next_secondary_attack_tick_ratio"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/next_secondary_attack_tick_ratio.parquet")
        assert_frame_equal(df, df_correct)

    def test_inventory(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["inventory"], ticks=[x for x in range(100000) if x % 100 == 0])
        df_correct = pd.read_parquet("tests/data/per_prop/inventory.parquet")
        assert_frame_equal(df, df_correct)

    def test_bomb_planted(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("bomb_planted")
        df_correct = pd.read_parquet("tests/data/per_event/bomb_planted.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_connect_full(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_connect_full")
        df_correct = pd.read_parquet("tests/data/per_event/player_connect_full.parquet")
        assert_frame_equal(df, df_correct)

    def test_item_pickup(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("item_pickup")
        df_correct = pd.read_parquet("tests/data/per_event/item_pickup.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_spawn(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_spawn")
        df_correct = pd.read_parquet("tests/data/per_event/player_spawn.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_time_warning(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_time_warning")
        df_correct = pd.read_parquet("tests/data/per_event/round_time_warning.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_hurt(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_hurt")
        df_correct = pd.read_parquet("tests/data/per_event/player_hurt.parquet")
        assert_frame_equal(df, df_correct)

    def test_cs_win_panel_match(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("cs_win_panel_match")
        df_correct = pd.read_parquet("tests/data/per_event/cs_win_panel_match.parquet")
        assert_frame_equal(df, df_correct)

    def test_cs_round_start_beep(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("cs_round_start_beep")
        df_correct = pd.read_parquet("tests/data/per_event/cs_round_start_beep.parquet")
        assert_frame_equal(df, df_correct)

    def test_hegrenade_detonate(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("hegrenade_detonate")
        df_correct = pd.read_parquet("tests/data/per_event/hegrenade_detonate.parquet")
        assert_frame_equal(df, df_correct)

    def test_smokegrenade_detonate(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("smokegrenade_detonate")
        df_correct = pd.read_parquet("tests/data/per_event/smokegrenade_detonate.parquet")
        assert_frame_equal(df, df_correct)

    def test_hltv_versioninfo(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("hltv_versioninfo")
        df_correct = pd.read_parquet("tests/data/per_event/hltv_versioninfo.parquet")
        assert_frame_equal(df, df_correct)

    def test_announce_phase_end(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("announce_phase_end")
        df_correct = pd.read_parquet("tests/data/per_event/announce_phase_end.parquet")
        assert_frame_equal(df, df_correct)

    def test_cs_round_final_beep(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("cs_round_final_beep")
        df_correct = pd.read_parquet("tests/data/per_event/cs_round_final_beep.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_announce_match_point(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_announce_match_point")
        df_correct = pd.read_parquet("tests/data/per_event/round_announce_match_point.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_fire(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("weapon_fire")
        df_correct = pd.read_parquet("tests/data/per_event/weapon_fire.parquet")
        assert_frame_equal(df, df_correct)

    def test_hltv_chase(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("hltv_chase")
        df_correct = pd.read_parquet("tests/data/per_event/hltv_chase.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_jump(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_jump")
        df_correct = pd.read_parquet("tests/data/per_event/player_jump.parquet")
        assert_frame_equal(df, df_correct)

    def test_buytime_ended(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("buytime_ended")
        df_correct = pd.read_parquet("tests/data/per_event/buytime_ended.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_prestart(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_prestart")
        df_correct = pd.read_parquet("tests/data/per_event/round_prestart.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_end(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_end")
        df_correct = pd.read_parquet("tests/data/per_event/round_end.parquet")
        assert_frame_equal(df, df_correct)

    def test_bomb_beginplant(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("bomb_beginplant")
        df_correct = pd.read_parquet("tests/data/per_event/bomb_beginplant.parquet")
        assert_frame_equal(df, df_correct)

    def test_bomb_dropped(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("bomb_dropped")
        df_correct = pd.read_parquet("tests/data/per_event/bomb_dropped.parquet")
        assert_frame_equal(df, df_correct)

    def test_inferno_startburn(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("inferno_startburn")
        df_correct = pd.read_parquet("tests/data/per_event/inferno_startburn.parquet")
        assert_frame_equal(df, df_correct)

    def test_bomb_planted(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("bomb_planted")
        df_correct = pd.read_parquet("tests/data/per_event/bomb_planted.parquet")
        assert_frame_equal(df, df_correct)

    def test_smokegrenade_expired(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("smokegrenade_expired")
        df_correct = pd.read_parquet("tests/data/per_event/smokegrenade_expired.parquet")
        assert_frame_equal(df, df_correct)

    def test_decoy_started(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("decoy_started")
        df_correct = pd.read_parquet("tests/data/per_event/decoy_started.parquet")
        assert_frame_equal(df, df_correct)

    def test_bomb_pickup(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("bomb_pickup")
        df_correct = pd.read_parquet("tests/data/per_event/bomb_pickup.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_announce_last_round_half(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_announce_last_round_half")
        df_correct = pd.read_parquet("tests/data/per_event/round_announce_last_round_half.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_zoom(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("weapon_zoom")
        df_correct = pd.read_parquet("tests/data/per_event/weapon_zoom.parquet")
        assert_frame_equal(df, df_correct)

    def test_cs_win_panel_round(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("cs_win_panel_round")
        df_correct = pd.read_parquet("tests/data/per_event/cs_win_panel_round.parquet")
        assert_frame_equal(df, df_correct)

    def test_flashbang_detonate(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("flashbang_detonate")
        df_correct = pd.read_parquet("tests/data/per_event/flashbang_detonate.parquet")
        assert_frame_equal(df, df_correct)

    def test_rank_update(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("rank_update")
        df_correct = pd.read_parquet("tests/data/per_event/rank_update.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_announce_match_start(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_announce_match_start")
        df_correct = pd.read_parquet("tests/data/per_event/round_announce_match_start.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_blind(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_blind")
        df_correct = pd.read_parquet("tests/data/per_event/player_blind.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_freeze_end(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_freeze_end")
        df_correct = pd.read_parquet("tests/data/per_event/round_freeze_end.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_disconnect(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_disconnect")
        df_correct = pd.read_parquet("tests/data/per_event/player_disconnect.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_mvp(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_mvp")
        df_correct = pd.read_parquet("tests/data/per_event/round_mvp.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_footstep(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_footstep")
        df_correct = pd.read_parquet("tests/data/per_event/player_footstep.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_team(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_team")
        df_correct = pd.read_parquet("tests/data/per_event/player_team.parquet")
        assert_frame_equal(df, df_correct)

    def test_hltv_message(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("hltv_message")
        df_correct = pd.read_parquet("tests/data/per_event/hltv_message.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_connect(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_connect")
        df_correct = pd.read_parquet("tests/data/per_event/player_connect.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_start(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_start")
        df_correct = pd.read_parquet("tests/data/per_event/round_start.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_poststart(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_poststart")
        df_correct = pd.read_parquet("tests/data/per_event/round_poststart.parquet")
        assert_frame_equal(df, df_correct)

    def test_round_officially_ended(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("round_officially_ended")
        df_correct = pd.read_parquet("tests/data/per_event/round_officially_ended.parquet")
        assert_frame_equal(df, df_correct)

    def test_inferno_expire(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("inferno_expire")
        df_correct = pd.read_parquet("tests/data/per_event/inferno_expire.parquet")
        assert_frame_equal(df, df_correct)

    def test_decoy_detonate(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("decoy_detonate")
        df_correct = pd.read_parquet("tests/data/per_event/decoy_detonate.parquet")
        assert_frame_equal(df, df_correct)

    def test_weapon_reload(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("weapon_reload")
        df_correct = pd.read_parquet("tests/data/per_event/weapon_reload.parquet")
        assert_frame_equal(df, df_correct)

    def test_item_equip(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("item_equip")
        df_correct = pd.read_parquet("tests/data/per_event/item_equip.parquet")
        assert_frame_equal(df, df_correct)

    def test_begin_new_match(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("begin_new_match")
        df_correct = pd.read_parquet("tests/data/per_event/begin_new_match.parquet")
        assert_frame_equal(df, df_correct)

    def test_player_death(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_death")
        df_correct = pd.read_parquet("tests/data/per_event/player_death.parquet")
        assert_frame_equal(df, df_correct)

    def test_cs_pre_restart(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("cs_pre_restart")
        df_correct = pd.read_parquet("tests/data/per_event/cs_pre_restart.parquet")
        assert_frame_equal(df, df_correct)


if __name__ == '__main__':
    unittest.main()