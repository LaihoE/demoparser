from demoparser2 import DemoParser

# make sure your demo has chickens
parser = DemoParser("path/to/demo.dem")
df = parser.parse_event("other_death")
df = df[df["othertype"] == "chicken"]

print(df.loc[:, ["attacker_name","othertype", "tick"]])
