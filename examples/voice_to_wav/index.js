var {parseVoice} = require('@laihoe/demoparser2');
const fs = require('fs');

// Returns <steamid, Uint8Array> map
// The bytes already include a wav header so you can
// directly write it to a file
steamid_bytes_map = parseVoice("path/to/demo.dem")

for (const [steamid, bytes] of Object.entries(steamid_bytes_map)) {
    const filePath = steamid.toString();
    fs.writeFileSync(filePath, Buffer.from(bytes));
}