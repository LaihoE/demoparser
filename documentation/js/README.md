## Function signatures
```TypeScript
function parseChatMessages(path: string): any
function listGameEvents(path: string): any
function parseGrenades(path: string): any
function parseHeader(path: string): any
function parsePlayerInfo(path: string): any

function parseEvent(path: string, eventName: string, extraPlayer?: Array<string> | undefined | null, extraOther?: Array<string> | undefined | null): any
function parseEvents(path: string, eventNames?: Array<string> | undefined | null, extraPlayer?: Array<string> | undefined | null, extraOther?: Array<string> | undefined | null): any
function parseTicks(path: string, wantedProps: Array<string>, wantedTicks?: Array<number> | undefined | null): any
```


<br/><br/>

```JavaScript
function parseEvent(path: string, eventName: string, extraPlayer?: Array<string> | undefined | null, extraOther?: Array<string> | undefined | null) -> JSON
```
Returns all events of requested type as JSON. The player_extra argument lets you request extra values that are not present in the real game event. For example the event "bomb_planted" has the following columns:

Output from: ```parseEvent("path_to_demo.dem","bomb_planted")```
```
   site   tick user_name       user_steamid
0   292  17916    player1  76561111111111111
1   299  59535    player2  76561111111111112
```
We notice that there is not that much info here, so we can add extra columns with the "player_extra" argument. This argument lets you ask for values from the players referred to in the event. For example to know where the bomb was planted we can add "X" and "Y" coordinates to the event in the following way: ```parseEvent("path_to_demo.dem", "bomb_planted", ["X", "Y"])```

Now our output has 2 new columns: "user_X" and "user_Y":
```
   site   tick user_name       user_steamid      user_X    user_Y
0   292  17916    player1  76561111111111111      213.4     874.6
1   299  59535    player2  76561111111111112      888.6     877.6
```
The player_extra argument does not expose any special "new" data, it is there mostly for convenience. The same info could be gotten with a combination of parse_events() and parse_ticks()


<br/><br/>
```JavaScript
function parseEvents(path: string, eventNames?: Array<string> | undefined | null, extraPlayer?: Array<string> | undefined | null, extraOther?: Array<string> | undefined | null): any

```
Same as parse_event but lets you query multiple events at a time. 
```parseEvents("path_to_demo.dem", ["player_death", "weapon_fire"])```. It might feel odd, why do we need both parseEvent and parseEvents?. The reason is mostly Python related, and I want to keep the same functions for both languages.



<br/><br/>
```JavaScript
export function parseTicks(path: string, wantedProps: Array<string>, wantedTicks?: Array<number> | undefined | null, structOfArrays?: boolean | undefined | null): any
```
Returns a DataFrame with wanted properties collected from players each tick.

With input
```JavaScript
parseTicks("path_to_demo.dem", ["X", "Y"] [10000, 10001])
```

Will get you the output:
```JavaScript
[
    {
        name: 'person1',
        steamid: '76511111111111111',
        tick: 10000,
        X: 123.456
        Y: 456.789

    },
    {
        name: 'person2',
        steamid: '76511111111111112',
        tick: 10000,
        X: 897.456
        Y: 4877.456
    },
        {
        name: 'person1',
        steamid: '76511111111111111',
        tick: 10001,
        X: 127.456
        Y: 459.789

    },
    {
        name: 'person2',
        steamid: '76511111111111112',
        tick: 10001,
        X: 898.456
        Y: 4889.456
    },
]
```
(inlcuded only 2 players in the example to make it shorter but should be 10 players each tick)
    

"ticks" argument lets you choose which ticks to parse.  
Remove "ticks" argument to get every tick in the demo.

Something unique to JavaScript version is the last argument "StructOfArrays: bool" that lets you choose the orientation of the output. Setting this to true will give you the following output:
```JavaScript
  X: [
    123.456,
    897.456,
    127.456,
    898.456,
  ],
  Y: [
    456.789
    4877.456
    459.789
    4889.456
  ],
  name: [
    "person1", 
    "person2",
    "person1",
    "person2",
  ],
  steamid: [
    '76511111111111111',
    '76511111111111112',
    '76511111111111111',
    '76511111111111112',
  ],
  tick: [
    10000,
    10000,
    10001,
    10001
  ]
```
where each index in each array maps to the same "struct". For example:
```JavaSCript
{
    name: name[2],
    steamid: steamid[2],
    tick: tick[2],
    X: X[2]
    Y: Y[2]
},
```
will get you the "third struct".

This option is mainly left as a performance optimization for people who want to squeeze more performance out of the parser. When parsing ALL TICKS in a demo you can expect 2-3 times faster parsing with this enabled. With smaller number of ticks, say 1000, the difference is tiny.

See more at https://en.wikipedia.org/wiki/AoS_and_SoA



<br/><br/>
```Python
def list_game_events(): -> List[str]
```



Notice that this function is rougly as slow as a call to parse_event/parse_events.

<br/><br/>
```Python
def parse_header(): -> Dict<str, str>
```
Output should be something like this:
```JavaScript
{
  addons: '',
  allow_clientside_entities: 'true',
  allow_clientside_particles: 'true',
  client_name: 'SourceTV Demo',
  demo_file_stamp: 'PBDEMS2\u0000',
  demo_version_guid: '8e9d71ab-04a1-4c01-bb61-acfede27c046',
  demo_version_name: 'valve_demo_2',
  fullpackets_version: '2',
  game_directory: '/opt/srcds/cs2/csgo_v2000111/csgo',
  map_name: 'de_overpass',
  network_protocol: '13928',
  server_name: 'Valve Counter-Strike 2 eu_north Server'
}
```
Mainly people care about the map_name field.

<br/><br/>
```Python
def parse_player_info(): -> DataFrame
```

Example output:
```
[
  { name: 'player1', steamid: '76511111111111111', team_number: 3 },
  { name: 'player2', steamid: '76511111111111112', team_number: 3 },
  { name: 'player3', steamid: '76511111111111113', team_number: 3 },
  { name: 'player4', steamid: '76511111111111114', team_number: 3 },
  { name: 'player5', steamid: '76511111111111115', team_number: 3 },
  { name: 'player6', steamid: '76511111111111116', team_number: 2 },
  { name: 'player7', steamid: '76511111111111117', team_number: 2 },
  { name: 'player8', steamid: '76511111111111118', team_number: 2 },
  { name: 'player9', steamid: '76511111111111119', team_number: 2 },
  { name: 'player10', steamid: '7651111111111110', team_number: 2 }
]
```
<br/><br/>
```Python
def parse_grenades(): -> DataFrame
```
Returns all coordinates of all grenades along with info about thrower.

```JavaScript
[  
  {
    entity_id: 280,
    grenade_type: 'SmokeGrenade',
    name: 'player1',
    steamid: '76561111111111111',
    tick: 5920,
    x: 694.5,
    y: 2116.90625,
    z: 258.71875
  },
  ...
]
```
entity_id refers to the id of the grenade and can be used to identify grenades when multiple grenades with the same name are thrown by a player.