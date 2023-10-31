var {parseEvent, parseTicks} = require('./index');


const pathToDemo = "path/to/demo.dem"

let gameEndTick = Math.max(...parseEvent(pathToDemo,"round_end").map(x => x.tick))

let fields = ["kills_total", "deaths_total", "mvps", "headshot_kills_total", "ace_rounds_total", "score"]
let scoreboard = parseTicks(pathToDemo, fields, [gameEndTick])
console.log(scoreboard);
