from demoparser2 import DemoParser


parser = DemoParser("path/to/demo.dem")
# The event includes a field called site, but it gives you an int like 204 etc.
# This will give you fields like "BombsiteA" and "BombsiteB"
df = parser.parse_events("bomb_planted", player=["last_place_name"])
print(df)