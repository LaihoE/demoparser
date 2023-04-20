# Demo parser for Counter-Strike 2


Work in progress so expect some bugs here and there!

## Install
```pip install demoparser2```


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
              X            Y       tick            steamid        name
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


##### List of values to try for parse_ticks()
```
"X"
"Y"
"Z"
"is_freeze_period"
"is_warmup_period"
"warmup_period_end"
"warmup_period_start"
"is_terrorist_timeout"
"is_ct_timeout"
"terrorist_timeout_remaining"
"ct_timeout_remaining"
"num_terrorist_timeouts"
"num_ct_timeouts"
"is_technical_timeout"
"is_waiting_for_resume"
"match_start_time"
"round_start_time"
"restart_round_time"
"is_game_restart"
"game_start_time"
"time_until_next_phase_start"
"game_phase"
"total_rounds_played"
"rounds_played_this_phase"
"hostages_remaining"
"any_hostages_reached"
"has_bombites"
"has_rescue_zone"
"has_buy_zone"
"is_matchmaking"
"match_making_mode"
"is_valve_dedicated_server"
"gungame_prog_weap_ct"
"gungame_prog_weap_t"
"spectator_slot_count"
"is_match_started"
"n_best_of_maps"
"is_bomb_dropped"
"is_bomb_planed"
"round_win_status"
"round_win_reason"
"terrorist_cant_buy"
"ct_cant_buy"
"num_player_alive_ct"
"num_player_alive_t"
"ct_losing_streak"
"t_losing_streak"
"survival_start_time"
"round_in_progress"
"is_auto_muted"
"crosshair_code"
"pending_team_num"
"player_color"
"ever_played_on_team"
"clan_name"
"is_coach_team"
"comp_rank"
"comp_wins"
"comp_rank_type"
"is_controlling_bot"
"has_controlled_bot_this_round"
"can_control_bot"
"is_alive"
"health"
"armor"
"has_defuser"
"has_helmet"
"spawn_time"
"death_time"
"score"
"game_time"
"is_connected"
"player_name"
"player_steamid"
"fov"
"balance"
"start_balance"
"total_cash_spent"
"cash_spent_this_round"
"music_kit_id"
"leader_honors"
"teacher_honors"
"friendly_honors"
"kills_this_round"
"deaths_this_round"
"assists_this_round"
"alive_time_this_round"
"headshot_kills_this_round"
"damage_this_round"
"objective_this_round"
"utility_damage_this_round"
"enemies_flashed_this_round"
"equipment_value_this_round"
"money_saved_this_round"
"kill_reward_this_round"
"cash_earned_this_round"
"kills_total"
"deaths_total"
"assists_total"
"alive_time_total"
"headshot_kills_total"
"ace_rounds_total"
"4k_rounds_total"
"3k_rounds_total"
"damage_total"
"objective_total"
"utility_damage_total"
"enemies_flashed_total"
"equipment_value_total"
"money_saved_total"
"kill_reward_total"
"cash_earned_total"
"ping"
"team_surrendered"
"team_rounds_total"
"team_name"
"team_score_overtime"
"team_match_stat"
"team_num_map_victories"
"team_score_first_half"
"team_score_second_half"
"team_clan_name
```



## Acknowledgements
Without Dotabuff's dota 2 parser "manta" this would not have been possible. Check it out: https://github.com/dotabuff/manta

The dota 2 demo format is very similar to CS2 demo format with only a few minor changes.