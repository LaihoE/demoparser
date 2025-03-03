import unittest
from unittest import TestCase

import pandas as pd
from demoparser2 import DemoParser, WantedPropState

demo_path = "../parser/test_demo.dem"


class SignatureTest(TestCase):
    def test_parse_header_signature(self):
        parser = DemoParser(demo_path)
        header = parser.parse_header()
        self.assertIsInstance(header, dict)
        for key, value in header.items():
            self.assertIsInstance(key, str)
            self.assertIsInstance(value, str)

    def test_list_game_events_signature(self):
        parser = DemoParser(demo_path)
        game_events = parser.list_game_events()
        self.assertIsInstance(game_events, list)
        for event in game_events:
            self.assertIsInstance(event, str)

    def test_parse_grenades_signature(self):
        parser = DemoParser(demo_path)
        grenades = parser.parse_grenades()
        self.assertIsInstance(grenades, pd.DataFrame)

    def test_parse_player_info_signature(self):
        parser = DemoParser(demo_path)
        player_info = parser.parse_player_info()
        self.assertIsInstance(player_info, pd.DataFrame)

    def test_parse_item_drops_signature(self):
        parser = DemoParser(demo_path)
        item_drops = parser.parse_item_drops()
        self.assertIsInstance(item_drops, pd.DataFrame)

    def test_parse_skins_signature(self):
        parser = DemoParser(demo_path)
        skins = parser.parse_skins()
        self.assertIsInstance(skins, pd.DataFrame)

    def test_parse_event_signature(self):
        parser = DemoParser(demo_path)

        event = parser.parse_event("player_death")
        self.assertIsInstance(event, pd.DataFrame)

        parser.parse_event(
            "player_death",
            player=["X", "Y"],
            other=["game_time", "total_rounds_played"],
        )
        parser.parse_event(
            "player_death",
            player=("X", "Y"),
            other=("game_time", "total_rounds_played"),
        )
        parser.parse_event("player_death", player=None, other=None)
        parser.parse_event("player_death", player=[], other=[])

        with self.assertRaises(TypeError):
            parser.parse_event("player_death", player=5, other=None)

        with self.assertRaises(TypeError):
            parser.parse_event("player_death", player=None, other=5)

        with self.assertRaises(TypeError):
            parser.parse_event("player_death", player="Test")

        with self.assertRaises(TypeError):
            parser.parse_event(5)

    def test_parse_events_signature(self):
        parser = DemoParser(demo_path)

        events = parser.parse_events(["player_death"])
        self.assertIsInstance(events, list)
        for event in events:
            self.assertIsInstance(event, tuple)
            self.assertIsInstance(event[0], str)
            self.assertIsInstance(event[1], pd.DataFrame)

        parser.parse_events(
            ["player_death"],
            player=["X", "Y"],
            other=["game_time", "total_rounds_played"],
        )
        parser.parse_events(
            ["player_death"],
            player=("X", "Y"),
            other=("game_time", "total_rounds_played"),
        )
        parser.parse_events(["player_death"], player=None, other=None)
        parser.parse_events(["player_death"], player=[], other=[])

        with self.assertRaises(TypeError):
            parser.parse_events(["player_death"], player=5, other=None)

        with self.assertRaises(TypeError):
            parser.parse_events(["player_death"], player=None, other=5)

        with self.assertRaises(TypeError):
            parser.parse_events(["player_death"], player="Test")

        with self.assertRaises(TypeError):
            parser.parse_events("player_death")

        with self.assertRaises(TypeError):
            parser.parse_events(5)

    def test_parse_voice_signature(self):
        parser = DemoParser(demo_path)
        voice = parser.parse_voice()
        self.assertIsInstance(voice, dict)
        for key, value in voice.items():
            self.assertIsInstance(key, str)
            self.assertIsInstance(value, bytes)

    def test_parse_ticks_signature(self):
        parser = DemoParser(demo_path)

        ticks = parser.parse_ticks(["X", "Y"])
        self.assertIsInstance(ticks, pd.DataFrame)

        parser.parse_ticks(["X", "Y"], players=[1, 2, 3], ticks=[1, 2, 3])
        parser.parse_ticks(["X", "Y"], players=None, ticks=None)
        parser.parse_ticks(["X", "Y"], players=[], ticks=[])
        parser.parse_ticks(
            ["X", "Y"],
            prop_states=[
                WantedPropState("is_alive", True),
                WantedPropState("is_bomb_planted", True),
            ],
        )

        with self.assertRaises(TypeError):
            parser.parse_ticks(["X", "Y"], players=5, ticks=None)

        with self.assertRaises(TypeError):
            parser.parse_ticks(["X", "Y"], players=None, ticks=5)

        with self.assertRaises(TypeError):
            parser.parse_ticks(["X", "Y"], players="Test")

        with self.assertRaises(TypeError):
            parser.parse_ticks(5)

        with self.assertRaises(TypeError):
            parser.parse_ticks(
                ["X", "Y"], prop_states=[{"prop": "is_alive", "state": True}]
            )

    def test_list_updated_fields(self):
        parser = DemoParser(demo_path)

        updated_fields = parser.list_updated_fields()
        self.assertIsInstance(updated_fields, list)
        for field in updated_fields:
            self.assertIsInstance(field, str)


if __name__ == "__main__":
    unittest.main()
