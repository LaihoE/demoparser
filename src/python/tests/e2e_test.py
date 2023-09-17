from unittest import TestCase
from demoparser2 import DemoParser
import pandas as pd
from pandas.testing import assert_frame_equal
import json
import unittest


def convert_same_dtypes(df, correct_df):
    for (dtyp, col) in zip(correct_df.dtypes, correct_df.columns):
        df[col] = df[col].astype(dtyp)
    return df


class ParserTest(TestCase):
    def test_parse_ticks(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_ticks(["X", "Y"], ticks=[79207])
        df_correct = pd.read_json("tests/data/python/tick_test.json")
        df = convert_same_dtypes(df, df_correct)
        assert_frame_equal(df, df_correct)

    def test_parse_events(self):
        parser = DemoParser("tests/data/test.dem")
        df = parser.parse_event("player_death", player=["X"], other=["total_rounds_played", "team_rounds_total"])
        df_correct = pd.read_json("tests/data/python/parse_events.json")
        df = convert_same_dtypes(df, df_correct)
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
        df_correct = pd.read_json("tests/data/python/parse_grenades.json")
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
        # This can also be slightly off is sub tick
        self.assertEqual(time_last_injury, game_time)


if __name__ == '__main__':
    unittest.main()