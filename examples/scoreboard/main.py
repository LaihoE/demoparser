from demoparser2 import DemoParser


parser = DemoParser("path_to_demo.dem")
max_tick = parser.parse_event("round_end")["tick"].max()

wanted_fields = ["kills_total", "deaths_total", "mvps", "headshot_kills_total", "ace_rounds_total", "4k_rounds_total", "3k_rounds_total"]
df = parser.parse_ticks(wanted_fields, ticks=[max_tick])
print(df)
