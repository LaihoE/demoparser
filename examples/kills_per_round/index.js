var {parseEvent} = require('../../src/node/index');

const filePath = "path/to/demo.dem"
let kills = parseEvent(filePath, "player_death", ["last_place_name", "team_name"], ["total_rounds_played", "is_warmup_period"])


// Here we could add more filters like weapons and zones etc.
// remove team-kills and warmup kills
let killsNoWarmup = kills.filter(kill => kill.is_warmup_period == false)
let filteredKills = killsNoWarmup.filter(kill => kill.attacker_team_name != kill.user_team_name)
let maxRound = Math.max(...kills.map(o => o.total_rounds_played))


for (let round = 0; round <= maxRound; round++){
    const killsPerPlayer = {};
    let killsThisRound = filteredKills.filter(kill => kill.total_rounds_played == round)
    killsThisRound.forEach(item => {
        const attackerName = item.attacker_name;
        const kills = killsPerPlayer[attackerName] || 0;
        killsPerPlayer[attackerName] = kills + 1;
    });
    console.log("round:", round)
    console.log(killsPerPlayer)
}
