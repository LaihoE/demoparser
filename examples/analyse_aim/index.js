var {parseTicks, parseEvent} = require('@laihoe/demoparser2');

const filePath = "path/to/demo.dem"

let events = parseEvent(filePath, "player_hurt")
let tickRecords = parseTicks(filePath, ["pitch", "yaw"])


for (let i = 0; i < events.length; i++){
    let startTick = events[i]["tick"] - 300;
    let endTick = events[i]["tick"];
    let attacker = events[i]["attacker_steamid"]

    // attacker can be none when player gets hurt by c4 etc.
    if (attacker != null){
        const between = (min, max) => (v) => v.tick >= min && v.tick <= max;
        wantedTicks = tickRecords.filter(between(startTick, endTick))
        wantedTicksAndPlayer = wantedTicks.filter(x => x.steamid == attacker)
        console.log(attacker)
    }
}
