ok = []

with open("data.txt") as f:
    lines = f.readlines()
    for line in lines:
        line = line.strip("\n")
        if len(line) > 4:
            ok.append(line)

for idx, x in enumerate(ok):
    spl = x.split(",")
    print(idx,"=>", spl[2][1:].split(")")[0]+",")


