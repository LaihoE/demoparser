var {parseTicks, parseEvent} = require('@laihoe/demoparser2');

const pathToDemo = "path/to/demo.dem";
// 1vX
const X = 4;


function find_if_1vx(deaths, round_idx, round_ends, tickData, X){
    for (let i = 0; i < deaths.length; i++){
        if (deaths[i].total_rounds_played == round_idx){
            
            let tickData_slice = tickData.filter(t => t.tick == deaths[i].tick)
            let ctAlive = tickData_slice.filter(t => t.team_name == "CT" && t.is_alive == true)
            let tAlive = tickData_slice.filter(t => t.team_name == "TERRORIST" && t.is_alive == true)
            // 3 = CT
            if (ctAlive.length == 1 && tAlive.length == X && round_ends[round_idx].winner == "CT"){
                return ctAlive[0].name
            }
            // 2 = T
            if (tAlive.length == 1 && ctAlive.length == X && round_ends[round_idx].winner == "T"){
                return tAlive[0].name
            }
        }
    }
}

let deaths = parseEvent(pathToDemo, "player_death", [], ["total_rounds_played"])
let wantedTicks = deaths.map(x => x.tick)
let round_ends = parseEvent(pathToDemo, "round_end")
let tickData = parseTicks(pathToDemo, ["is_alive", "team_name", "team_rounds_total"], wantedTicks)
let maxRound = Math.max(...deaths.map(x => x.total_rounds_played))

for (let i = 0; i <= maxRound; i++){
    let res = find_if_1vx(deaths, i, round_ends, tickData, X);
    if (res != undefined){
        console.log("Round", i , res, "clutched a 1 v", X);
    }
}
