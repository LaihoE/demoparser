from demoparser2 import DemoParser


parser = DemoParser("path/to/demo.dem")
wanted_ticks = parser.parse_event("round_freeze_end")["tick"].tolist()
df = parser.parse_ticks(["current_equip_value", "total_rounds_played"], ticks=wanted_ticks)

max_round = df["total_rounds_played"].max()
for round in range(0, max_round + 1):
    subdf = df[df["total_rounds_played"] == round]
    print(f"ROUND: {round}")
    print(subdf)