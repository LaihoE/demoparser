from demoparser2 import DemoParser
import glob


parser = DemoParser("path/to/demo.dem")
df = parser.parse_event("player_death", other=["game_time", "round_start_time"])
df["player_died_time"] = df["game_time"] - df["round_start_time"]

print(df.loc[:, ["attacker_name", "player_died_time"]])