var {parseEvent, parseTicks} = require('../../src/node/index');

const filePath = "path/to/demo.dem"

let events = parseEvent(filePath, "round_freeze_end")
let wantedTicks = events.map(event => event.tick)
let tickData = parseTicks(filePath, ["current_equip_value", "total_rounds_played"], wantedTicks)

let maxRound = Math.max(...tickData.map(t => t.total_rounds_played))
for (let round = 0; round < maxRound + 1; round++){
    playersThisRound = tickData.filter(t => t.total_rounds_played == round)
    console.log(playersThisRound)
}
