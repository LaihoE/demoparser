from demoparser2 import DemoParser
import wave
import opuslib
import time
# pip install opuslib
# pip install wave


# Writes one wav per player

decoder = opuslib.Decoder(48000, 1)
parser = DemoParser("path/to/demo.dem")
out = parser.parse_voice()

unique_players = set([p["steamid"] for p in out])
for player in unique_players:
    bytes_this_player = [x["bytes"] for x in out if x["steamid"] == player]

    frames = []
    for b in bytes_this_player:
        frames.append(decoder.decode(b, frame_size=960, decode_fec=False))
    pcm_data = b"".join(frames)

    with wave.open(f"{player}.wav", "wb") as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(48000)
        f.writeframes(pcm_data)