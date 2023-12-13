from demoparser2 import DemoParser

parser = DemoParser("path/to/demo.dem")
# returns steamid, bytes
# bytes include a wav header in the beginning so
# it can be written directly
steamid_bytes_dict = parser.parse_voice()

for steamid, raw_bytes in steamid_bytes_dict.items():    
    with open(f"{steamid}.wav", "wb") as f:
        f.write(raw_bytes)