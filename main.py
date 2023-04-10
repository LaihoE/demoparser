from demoparser2 import DemoParser
import pyarrow as pa
import polars as pl
import time
import pandas as pd
import matplotlib.pyplot as plt


pd.set_option('display.max_rows', 15000)


parser = DemoParser("/home/laiho/Documents/demos/cs2/fulls2demo.dem")

df = parser.parse_ticks(["m_szTeamname"], ticks=[x for x in range(19000, 20000)])

# df = df[df["steamid"] == 76561198813053167]
print(df["m_szTeamname"].unique())

before = time.time()
print(df)