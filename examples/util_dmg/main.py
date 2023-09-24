from demoparser2 import DemoParser
import glob


parser = DemoParser("path_to_demo.dem")
df = parser.parse_event("player_hurt")

he_dmg = df[df["weapon"] == "hegrenade"]
molotov_dmg = df[(df["weapon"] == "molotov") | (df["weapon"] == "inferno")]

print(he_dmg)
print(molotov_dmg)
