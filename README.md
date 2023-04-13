# Demo parser for Counter-Strike 2


Work in progress so expect some bugs here and there!

## Install
```pip install demoparser2```

### Usage:
Game events
```python
from demoparser2 import DemoParser

parser = DemoParser("path_to_demo.dem")
df = parser.parse_events("player_death")
```


# Documentation
This will have to do for now 😂

### Utility functions
```python
parser = DemoParser("path_to_demo.dem")

# returns list like: ["player_death", "weapon_fire"...]
name_list = parser.list_game_events()
# returns dict like: {"m_iHealth": 41487, "m_iPing": 4871} where 
# key is field name and val is how many times it was updated.
freq_dict = parser.list_entity_values()


df = parser.parse_grenades()
df = parser.parse_item_drops()
df = parser.parse_skins()
df = parser.parse_player_info()
dict_output = parser.parse_convars()

```
All of these take no arguments and return the same shape data. Probably easiest to understand by just trying these out.

### Game events
```python
from demoparser2 import DemoParser

parser = DemoParser("path_to_demo.dem")
df = parser.parse_events("player_death")
```
You can find out what events your demo had with:
```event_names = parser.list_game_events()```



This can be helpful: https://wiki.alliedmods.net/Counter-Strike:_Global_Offensive_Events  
List is for CSGO events seem similar to CS2.


### Tick data (entities)
```python
from demoparser2 import DemoParser

wanted_fields = ["X", "Y"]

parser = DemoParser("path_to_demo.dem")
df = parser.parse_ticks(wanted_fields)
```
#####  Example output of parse_ticks():
```
             m_vecX       m_vecY   tick            steamid        name
0        649.795044   633.648438      0  76512345678912345      person1
1        526.207642   794.186157      0  76512345678912345      person2
2        997.494324   583.692871      0  76512345678912345      person3
3        958.421570   498.485657      0  76512345678912345      person4
4        624.525696   556.522217      0  76512345678912345      person5
...             ...          ...    ...                ...         ...
913215   924.593140   308.131622  30452  76512345678912345      person6
913216   598.564514   794.186157  30452  76512345678912345      person7
913217   329.393677   323.219360  30452  76512345678912345      person8
913218   526.207642    81.611084  30452  76512345678912345      person9
913219    36.691650   308.887451  30452  76512345678912345      person10
```


##### List of possible fields for parse_ticks() I've tried and output seems reasonable (more comming soon):
X
Y
round
m_iHealth
m_szLastPlaceName
m_bInBuyZone
m_bIsScoped
m_iAccount
m_iCashSpentThisRound
m_iPing
m_bWarmupPeriod
m_bFreezePeriod





## Acknowledgements
Without Dotabuff's dota 2 parser "manta" this would not have been possible. Check it out: https://github.com/dotabuff/manta

The dota 2 demo format is very similar to CS2 demo format with only a few minor changes.