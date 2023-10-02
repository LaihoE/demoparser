var {parseEvent} = require('@laihoe/demoparser2');

const filePath = "path/to/demo.dem"
const MY_STEAMID = "765611111111111"

let events = parseEvent(filePath, "player_death", ["last_place_name"])
let my_kills = events.filter(x => x.attacker_steamid == MY_STEAMID)
let my_deaths = events.filter(x => x.user_steamid == MY_STEAMID)

// Find all unique zones
let kill_zones = my_kills.map(x => x.attacker_last_place_name)
let death_zones = my_deaths.map(x => x.user_last_place_name)
let all_zones = kill_zones.concat(death_zones)
let all_zones_unique = [...new Set(all_zones)];


for (let i = 0; i < all_zones_unique.length; i++){
    zone = all_zones_unique[i]
    kills = my_kills.filter(e => e.attacker_last_place_name == zone).length
    deaths = my_deaths.filter(e => e.user_last_place_name == zone).length

    console.log(zone, kills, deaths)
}

