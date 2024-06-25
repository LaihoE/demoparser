import pandas as pd
from typing import Dict, Sequence

class DemoParser:
    def __init__(self, path: str) -> None: ...
    def parse_header(self) -> Dict[str, str]: ...
    def parse_convars(self) -> Dict[str, str]: ...
    def Sequence_game_events(self) -> Sequence[str]: ...
    def parse_grenades(self) -> pd.DataFrame: ...
    def parse_chat_messages(self) -> pd.DataFrame: ...
    def parse_player_info(self) -> pd.DataFrame: ...
    def parse_item_drops(self) -> pd.DataFrame: ...
    def parse_skins(self) -> pd.DataFrame: ...
    def parse_event(
        self,
        event_name: str,
        player: Sequence[str] = [],
        other: Sequence[str] = [],
        player_extra: Sequence[str] = [],
        other_extra: Sequence[str] = [],
    ) -> pd.DataFrame: ...
    def parse_events(
        self,
        event_name: Sequence[str],
        player: Sequence[str] = [],
        other: Sequence[str] = [],
        player_extra: Sequence[str] = [],
        other_extra: Sequence[str] = [],
    ) -> pd.DataFrame: ...
    def parse_voice(self) -> Dict[str, bytes]: ...
    def parse_ticks(
        self,
        wanted_props: Sequence[str],
        player: Sequence[str] = [],
        ticks: Sequence[str] = [],
    ) -> pd.DataFrame:
        """Parse the specified props.

        Args:
            wanted_props (Sequence[str]): The props to parse for each player at each tick.
            player (Sequence[int]): Sequence of Steam IDs of the players to parse. An empty Sequence means all players.
            ticks (Sequence[int]): Sequence of ticks to parse. An empty Sequence means all ticks.

        Returns:
            pd.DataFrame: Dataframe of all the parsed props for each player at each tick.
        """
