import unittest
from unittest import TestCase
from typing import Union

import pandas as pd
from demoparser2 import DemoParser, WantedPropState

demo_path = "../parser/test_demo.dem"


class MyWantedPropState:
    def __init__(self, prop: str, state: Union[bool, str, int, float]):
        self.prop = prop
        self.state = state


class SignatureTest(TestCase):
    def test_demoparser_creation(self):
        DemoParser(demo_path)
        DemoParser(demo_path=demo_path)


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

    def test_parse_weapons_signature(self):
        parser = DemoParser(demo_path)
        weapons = parser.parse_weapons()
        self.assertIsInstance(weapons, pd.DataFrame)
        for col in ["tick", "entity_id", "weapon", "x", "y", "z", "steamid", "name"]:
            self.assertIn(col, weapons.columns)
        parser.parse_weapons(extra=["m_nDropTick"], ticks=[100, 200])
        with self.assertRaises(TypeError):
            parser.parse_weapons(extra=[5])
        with self.assertRaises(TypeError):
            parser.parse_weapons(ticks="wrong")

    def test_parse_grenades_signature(self):
        parser = DemoParser(demo_path)
        grenades = parser.parse_grenades()
        self.assertIsInstance(grenades, pd.DataFrame)
        parser.parse_grenades(extra=["game_time"], grenades=True)
        parser.parse_grenades(extra=["game_time"], grenades=False)
        with self.assertRaises(TypeError):
            parser.parse_grenades(extra=[5])
        with self.assertRaises(TypeError):
            parser.parse_grenades(extra="game_time")
        with self.assertRaises(TypeError):
            parser.parse_grenades(grenades="wrong")

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
        self.assertIsInstance(voice, list)
        for entry in voice:
            self.assertEqual(set(entry.keys()), {"ticks", "steamid", "bytes"})
            self.assertIsInstance(entry["tick"], int)
            self.assertIsInstance(entry["steamid"], int)
            self.assertIsInstance(entry["bytes"], bytes)

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

        parser.parse_ticks(
            ["X", "Y"],
            prop_states=[
                MyWantedPropState("is_alive", True),
                MyWantedPropState("is_bomb_planted", True),
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

        with self.assertRaises(AttributeError):
            parser.parse_ticks(
                ["X", "Y"], prop_states=[{"prop": "is_alive", "state": True}]
            )

    def test_velocity_across_respawn(self):
        # In test_demo.dem this player drops over 230 units from their death
        # position to their next spawn. Ticks 9420 and 9421 are missing.
        rows = DemoParser(demo_path).parse_ticks(
            ["Z", "velocity_Z"], players=[76561198324843075], ticks=list(range(9416, 9425))
        ).set_index("tick")
        self.assertLess(rows.at[9419, "Z"] - rows.at[9418, "Z"], -230)
        self.assertTrue(pd.isna(rows.at[9422, "velocity_Z"]))
        self.assertTrue(pd.isna(rows.at[9423, "velocity_Z"]))
        self.assertEqual(rows.at[9424, "velocity_Z"], 64 * (rows.at[9423, "Z"] - rows.at[9422, "Z"]))

    def test_sparse_ticks_preserve_velocity(self):
        parser = DemoParser(demo_path)
        players = [76561198324843075]
        props = ["velocity_X", "velocity_Y", "velocity_Z", "velocity"]
        ticks = [10695, 10759, 10823]
        full = parser.parse_ticks(props, players=players).set_index("tick")
        sparse = parser.parse_ticks(props, players=players, ticks=ticks).set_index("tick")
        self.assertEqual(sparse.index.tolist(), ticks)
        for prop in props:
            self.assertEqual(sparse[prop].tolist(), full.loc[ticks, prop].tolist())
        self.assertEqual(sparse.at[10823, "velocity_Z"], -1.34375)

    def test_list_updated_fields(self):
        parser = DemoParser(demo_path)

        updated_fields = parser.list_updated_fields()
        self.assertIsInstance(updated_fields, list)
        for field in updated_fields:
            self.assertIsInstance(field, str)


if __name__ == "__main__":
    unittest.main()
