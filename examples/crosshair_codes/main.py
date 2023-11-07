from demoparser2 import DemoParser
import glob


parser = DemoParser("path/to/demo.dem")

last_tick = parser.parse_event("round_end")["tick"].to_list()[-1]
df = parser.parse_ticks(["crosshair_code"],ticks=[last_tick])
print(df)