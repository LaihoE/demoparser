var {parseTicks, parseEvent} = require('../../src/node/index');

const filePath = "path/to/demo.dem"


let events = parseEvent(filePath, "player_hurt")
let heDmg = events.filter(e => e.weapon == "hegrenade")
let molotovDmg = events.filter(e => e.weapon == "molotov" || e.weapon == "inferno")

console.log(heDmg)
console.log(molotovDmg)