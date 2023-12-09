from demoparser2 import DemoParser
import time

parser = DemoParser("test.dem")

before = time.time()
for _ in range(10):
    event_df = parser.parse_event("player_death", player=["X", "Y"], other=["total_rounds_played"])
print("events took:", time.time() - before)

before = time.time()
for _ in range(10):
    ticks_df = parser.parse_ticks(["X", "Y"])
print("ticks took:",time.time() - before)