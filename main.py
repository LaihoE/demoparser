from demoparser import DemoParser
import matplotlib.pyplot as plt
import time


parser = DemoParser("/home/laiho/Documents/demos/cs2/fulls2demo.dem")
df = parser.parse_ticks(["m_unTotalRoundDamageDealt"], wanted_ticks=[x for x in range(100000)])
print(df["m_unTotalRoundDamageDealt"].unique())
