var {parseEvent} = require('@laihoe/demoparser2');

// The event includes a field called site, but it gives you a big int like 204 etc.
// This will give you fields like "BombsiteA" and "BombsiteB"
let events = parseEvent("path_to_demo.dem", "bomb_planted", ["last_place_name"])
console.log(events[0])