import pandas as pd
from typing import Dict, Sequence, Optional

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
        player: Optional[Sequence[str]] = None,
        other: Optional[Sequence[str]] = None,
    ) -> pd.DataFrame: ...
    def parse_events(
        self,
        event_name: Sequence[str],
        player: Optional[Sequence[str]] = None,
        other: Optional[Sequence[str]] = None,
    ) -> pd.DataFrame: ...
    def parse_voice(self) -> Dict[str, bytes]: ...
    def parse_ticks(
        self,
        wanted_props: Sequence[str],
        player: Optional[Sequence[int]] = None,
        ticks: Optional[Sequence[int]] = None,
    ) -> pd.DataFrame:
        """Parse the specified props.

        Args:
            wanted_props (Sequence[str]): The props to parse for each player at each tick.
            player (Optional[Sequence[int]]): Sequence of Steam IDs of the players to parse.
                `None` or an empty Sequence means all players. Defaults to `None`.
            ticks (Optional[Sequence[int]]): Sequence of ticks to parse.
                `None` or an empty Sequence means all ticks. Defaults to `None`.

        Returns:
            pd.DataFrame: Dataframe of all the parsed props for each player at each tick.
        """