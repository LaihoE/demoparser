import unittest
from unittest import TestCase

import pandas as pd
from demoparser2 import DemoParser


class SignatureTest(TestCase):
    def test_parse_header_signature(self):
        parser = DemoParser("tests/data/test.dem")
        header = parser.parse_header()
        self.assertIsInstance(header, dict)
        for key, value in header.items():
            self.assertIsInstance(key, str)
            self.assertIsInstance(value, str)

    def test_parse_convars_signature(self):
        parser = DemoParser("tests/data/test.dem")
        convars = parser.parse_convars()
        self.assertIsInstance(convars, dict)
        for key, value in convars.items():
            self.assertIsInstance(key, str)
            self.assertIsInstance(value, str)

    def test_list_game_events_signature(self):
        parser = DemoParser("tests/data/test.dem")
        game_events = parser.list_game_events()
        self.assertIsInstance(game_events, list)
        for event in game_events:
            self.assertIsInstance(event, str)

    def test_parse_grenades_signature(self):
        parser = DemoParser("tests/data/test.dem")
        grenades = parser.parse_grenades()
        self.assertIsInstance(grenades, pd.DataFrame)

    def test_parse_chat_messages_signature(self):
        parser = DemoParser("tests/data/test.dem")
        chat_messages = parser.parse_chat_messages()
        self.assertIsInstance(chat_messages, pd.DataFrame)

    def test_parse_player_info_signature(self):
        parser = DemoParser("tests/data/test.dem")
        player_info = parser.parse_player_info()
        self.assertIsInstance(player_info, pd.DataFrame)

    def test_parse_item_drops_signature(self):
        parser = DemoParser("tests/data/test.dem")
        item_drops = parser.parse_item_drops()
        self.assertIsInstance(item_drops, pd.DataFrame)

    def test_parse_skins_signature(self):
        parser = DemoParser("tests/data/test.dem")
        skins = parser.parse_skins()
        self.assertIsInstance(skins, pd.DataFrame)

    def test_parse_event_signature(self):
        parser = DemoParser("tests/data/test.dem")

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
        parser = DemoParser("tests/data/test.dem")

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

    # def parse_voice(self) -> Dict[str, bytes]: ...
    # def parse_ticks(
    #     self,
    #     wanted_props: Sequence[str],
    #     player: Optional[Sequence[int]] = None,
    #     ticks: Optional[Sequence[int]] = None,
    # ) -> pd.DataFrame:


if __name__ == "__main__":
    unittest.main()
