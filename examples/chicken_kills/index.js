var {parseEvent} = require('@laihoe/demoparser2');


// make sure your demo has chickens
let events = parseEvent("path/to/demo.dem", "other_death")
let chickenKills = events.filter(event => event.othertype == "chicken")

console.log(chickenKills)