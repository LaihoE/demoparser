from demoparser2 import DemoParser

# 1vX
X = 4
path_to_demo = "path/to/demo.dem"

def find_if_1vx(deaths, round_idx, round_ends, df, X):
    for _, death in deaths.iterrows():
        if death["total_rounds_played"] == round_idx:

            subdf = df[df["tick"] == death["tick"]]
            ct_alive = subdf[(subdf["team_name"] == "CT") & (subdf["is_alive"] == True)]
            t_alive = subdf[(subdf["team_name"] == "TERRORIST") & (subdf["is_alive"] == True)]
            # 3 = CT
            if len(ct_alive) == 1 and len(t_alive) == X and round_ends.iloc[round_idx]["winner"] == 3:
                return ct_alive["name"].iloc[0]
            # 2 = T
            if len(t_alive) == 1 and len(ct_alive) == X and round_ends.iloc[round_idx]["winner"] == 2:
                return t_alive["name"].iloc[0]


parser = DemoParser(path_to_demo)
deaths = parser.parse_event("player_death", other=["total_rounds_played"])
round_ends = parser.parse_event("round_end")
df = parser.parse_ticks(["is_alive", "team_name", "team_rounds_total"], ticks=deaths["tick"].to_list())
max_round = deaths["total_rounds_played"].max() + 1

for round_idx in range(0, max_round):
    clutcher_steamid = find_if_1vx(deaths, round_idx, round_ends, df, X)
    if clutcher_steamid != None:
        print(f"round: {round_idx} {clutcher_steamid} clutched a 1v{X}")

