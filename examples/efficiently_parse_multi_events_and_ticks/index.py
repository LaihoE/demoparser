from demoparser2 import DemoParser

parser = DemoParser("/path/to/demfile.dem")

event_names = [
    "begin_new_match", "round_start", "round_end", "round_mvp", 
    "player_death", "bomb_planted", "bomb_defused", "hostage_rescued", 
    "weapon_fire", "flashbang_detonate", "hegrenade_detonate", 
    "molotov_detonate", "smokegrenade_detonate", "player_hurt", 
    "player_blind"
]

all_events = parser.parse_events(event_names, other=["game_time", "team_num"])

# Find match start tick
begin_new_match_df = next((df for event_name, df in all_events if event_name == 'begin_new_match'), None)
match_start_tick = begin_new_match_df['tick'].iloc[0] if begin_new_match_df is not None else 0

# Filter out events before the match start
filtered_events = [(event_name, df[df['tick'] >= match_start_tick]) for event_name, df in all_events]

wanted_props = ["equipment_value_this_round", "cash_spent_this_round", "is_alive", "team_num", "player_name", "score", "player_steamid"]
tick_values = set()
for _, df in filtered_events:
    tick_values.update(df['tick'].unique())
all_ticks = parser.parse_ticks(wanted_props, ticks=list(tick_values))

# Convert ticks to a map
all_ticks_map = {}
for tick in all_ticks.itertuples():
    if tick.tick not in all_ticks_map:
        all_ticks_map[tick.tick] = []
    all_ticks_map[tick.tick].append(tick)

# Access ticks
game_end_events = next((df for event_name, df in filtered_events if event_name == 'round_end'), None)
game_end_tick = max(game_end_events['tick']) if game_end_events is not None else 0
scoreboard = all_ticks_map.get(game_end_tick, [])

# Additional processing for other events
shot_events = next((df for event_name, df in filtered_events if event_name == 'weapon_fire'), None)
hit_events = next((df for event_name, df in filtered_events if event_name == 'player_hurt'), None)
flash_events = next((df for event_name, df in filtered_events if event_name == 'player_blind'), None)

print(scoreboard)
print(hit_events)
