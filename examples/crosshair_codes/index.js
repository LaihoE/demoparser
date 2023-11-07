var {parseEvent, parseTicks} = require('@laihoe/demoparser2');


const pathToDemo = "path/to/demo.dem";

let gameEndTick = Math.max(...parseEvent(pathToDemo, "round_end").map(x => x.tick))
let players = parseTicks(pathToDemo, ["crosshair_code"], [gameEndTick])

console.log(players)
