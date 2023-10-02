var {parseEvent} = require('@laihoe/demoparser2');

const filePath = "path/to/demo.dem"

let events = parseEvent(filePath, "player_death", [], ["game_time", "round_start_time"])
for (let i = 0; i < events.length; i++){
    let deathTime = events[i].game_time;
    let roundStartTime = events[i].round_start_time;
    console.log(deathTime - roundStartTime)
}
