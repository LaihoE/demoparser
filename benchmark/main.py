from demoparser2 import DemoParser
import time
import glob

files = glob.glob("/path/to/demos/*")

before = time.time()

for file in files:
    parser = DemoParser(file)
    df = parser.parse_event("player_death", player=["X", "Y"])

print(time.time() - before)