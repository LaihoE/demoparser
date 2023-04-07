from demoparser2 import DemoParser
import pyarrow as pa
import polars as pl
import time
import pandas as pd
import matplotlib.pyplot as plt


pd.set_option('display.max_rows', 15000)


parser = DemoParser("/home/laiho/Documents/demos/cs2/s2.dem")

x = parser.parse_grenades()#parse_ticks(["X"])


before = time.time()
df = x.to_pandas(use_pyarrow_extension_array = True)
print(df)
#df = df[df["grenade_type"] == "SmokeGrenade"]

#df = df[df["thrower_steamid"] == 76561198140741242]
print(df)
# print(time.time() - before)
# print(df.isna().sum())

wf = parser.parse_events("weapon_fire", props=["m_vecX", "m_vecY"])

#print(parser.parse_events("smokegrenade_detonate"))

"""
0        390    145    Cannibal  76561198140741242  1237.980713  2089.861084    2.109430
1        188   7640    mT'Fryta  76561198968844069 -1274.307617  2246.787354    4.297961
2        162  11955    DJ =DDD!  76561197969209908   531.683960   604.522705    3.401668
3        215  12064        yoru  76561198892591430 -1949.859619  1351.326294   29.492683
4        245  12394    mT'Fryta  76561198968844069    34.789402  1481.812134    1.458601
5        327  12886   (-_-)︻デ 一  76561198800126586  -190.961578  1081.599609    2.204125
6        266  15782        yoru  76561198892591430   530.476135   651.179199    3.384104
7        388  16314    Cannibal  76561198140741242 -1420.038696  2181.730469    2.029643
8        429  17721  Ａｍｓｃｈｅｌ Ｒ.  76561199092115465 -1126.852539  2397.028809   17.418221
9        130  22018        yoru  76561198892591430 -1972.055542  1474.250366   30.059690
10       149  23598      Alexik  76561198034907504  -447.875793  -608.458313    3.846407
11       252  25980        yoru  76561198892591430   597.223328   757.237854    0.926600
12       481  27746    DJ =DDD!  76561197969209908  -481.099457  1404.997070 -126.419304
"""
# wf = wf[wf["weapon"] == "weapon_smokegrenade"]
det = parser.parse_events("flashbang_detonate")


plt.scatter(x=df["X"], y=df["Y"], s=1)
plt.plot(det["x"], det["y"], "ro")



print(wf.columns)
plt.plot(wf["user_m_vecX"], wf["user_m_vecY"], "bo")
print(wf)
plt.show()

