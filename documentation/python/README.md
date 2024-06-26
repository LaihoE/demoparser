## Function signatures
```Python
def __init__(self, path: str) -> None: ...


# takes no arguments
def parse_header(self) -> Dict[str, str]: ...
def parse_convars(self) -> Dict[str, str]: ...
def Sequence_game_events(self) -> List[str]: ...
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
) -> List[Tuple[str, pd.DataFrame]]: ...
def parse_voice(self) -> Dict[str, bytes]: ...
def parse_ticks(
   self,
   wanted_props: Sequence[str],
   player: Optional[Sequence[int]] = None,
   ticks: Optional[Sequence[int]] = None,
) -> pd.DataFrame:
```
See below for more in-depth explanations of above functions.

<br/><br/>

```Python
def parse_event(event_name: str, player=List[str], other=List[str]): -> DataFrame
```
Returns a DataFrame with events of the specified type. The "player" argument lets you request extra values that are not present in the real game event. For example the event "bomb_planted" has the following columns:

Output from: ```parse_event("bomb_planted")```
```
   site   tick user_name       user_steamid
0   292  17916    player1  76561111111111111
1   299  59535    player2  76561111111111112
```
We notice that there is not that much info here, so we can add extra columns with the "player" argument. This argument lets you ask for values from the players referred to in the event. For example to know where the bomb was planted we can add "X" and "Y" coordinates to the event in the following way: ```parse_event("bomb_planted", player=["X", "Y"])```

Now our output has 2 new columns: "user_X" and "user_Y":
```
   site   tick user_name       user_steamid      user_X    user_Y
0   292  17916    player1  76561111111111111      213.4     874.6
1   299  59535    player2  76561111111111112      888.6     877.6
```
Sometimes you don't want to get a value from the perspective of a player, but rather the "state" of the game. An example of this would be to find out how many rounds have been played. This can be done with the "other" argument:
```parse_event("bomb_planted", player=["X", "Y"], other=["total_rounds_played"])```
Notice that it is only valid to request "game state" props in the "other" argument.


<br/><br/>
```Python
def parse_events(event_name: Sequence[str], player_extra=Sequence[str]): -> [(str, DataFrame)]
```
Same as parse_event but lets you query multiple events at a time. 
```parse_events(["player_death", "weapon_fire"])``` will give you the following output: [("player_death", df), ("weapon_fire", df)]



<br/><br/>
```Python
def parse_ticks(wanted_props: Sequence[str], ticks=Sequence[int]): -> DataFrame
```
Returns a DataFrame with wanted properties collected from players each tick.

With input
```Python
parse_ticks(wanted_props: ["X", "Y", "Z"], ticks=[983])
```

We get the output:

    
              X           Y       Z  tick           steamid         name
     0 -1000.875  123.46875   540.0   983       76511234596897    player1
     1 -2000.875  879.46875   129.0   983       76511234596897    player2
     2 -3000.875  8798.4675   987.0   983       76511234596897    player3
                                      ...   
"ticks" argument lets you choose which ticks to parse.  
Remove "ticks" argument to get every tick in the demo.

<br/><br/>
```Python
def list_game_events(): -> List[str]
```
output along these lines:
['announce_phase_end', 'cs_round_start_beep', 'hltv_message', 'weapon_fire', 'hltv_chase', 'round_end' ... ]

Notice that this function is rougly as slow as a call to parse_event/parse_events.

<br/><br/>
```Python
def parse_header(): -> Dict[str, str]
```
Header should have the following fields:

"addons", "server_name", "demo_file_stamp", "network_protocol",
"map_name", "fullpackets_version", "allow_clientside_entities",
"allow_clientside_particles", "demo_version_name", "demo_version_guid",
"client_name", "game_directory"
<br/><br/>
```Python
def parse_player_info(): -> DataFrame
```

Example output:
```
             steamid          name     team_number
0  76561111111111111         player1        2
1  76561111111111112         player2        2
                        ...
```
<br/><br/>
```Python
def parse_grenades(): -> DataFrame
```
Returns all coordinates of all grenades along with info about thrower. entity_id refers to the id of the grenade and can be used to identify grenades when multiple grenades with the same name are thrown by a player.


    
    Example:
             X           Y       Z  tick     thrower_steamid    grenade_type   entity_id
    0 -388.875  1295.46875 -5120.0   982     76561111111111111    HeGrenade        522
    1 -388.875  1295.46875 -5120.0   983     76561111111111111    HeGrenade        522
    2 -388.875  1295.46875 -5120.0   983     76561111111111111    HeGrenade        522