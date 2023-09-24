from demoparser2 import DemoParser


parser = DemoParser("path/to/demo.dem")
player_hurt_events = parser.parse_event("player_hurt")
df = parser.parse_ticks(["pitch", "yaw"])

for (idx, event) in player_hurt_events.iterrows():
    start_tick = event["tick"] - 300
    end_tick = event["tick"]
    attacker = event["attacker_steamid"]
    # attacker can be none when player gets hurt by c4 etc.
    if attacker != None:
        subdf = df[(df["tick"].between(start_tick, end_tick)) & (df["steamid"] == int(attacker))]
        print(subdf)