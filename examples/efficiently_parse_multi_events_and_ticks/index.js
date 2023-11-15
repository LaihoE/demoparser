var {parseEvents, parseTicks} = require('@laihoe/demoparser2');


const pathToDemo = "/path/to/demfile.dem"

// Define all event names we're interested in
const eventNames = [
    "begin_new_match", "round_start", "round_end", "round_mvp", 
    "player_death", "bomb_planted", "bomb_defused", "hostage_rescued", 
    "weapon_fire", "flashbang_detonate", "hegrenade_detonate", 
    "molotov_detonate", "smokegrenade_detonate", "player_hurt", 
    "player_blind"
  ];
  
  let allEvents = parseEvents(pathToDemo, eventNames,["game_time","team_num"]);  // Fetch all events with any fields needed for any events
  
  // Find match start tick by filtering already parsed events
  const matchStartTick = allEvents.find(event => event.event_name === "begin_new_match")?.tick || 0;
  const roundStartEvents = allEvents.filter(event => event.event_name === "round_start");
  const roundEndEvents = allEvents.filter(event => event.event_name === "round_end" && event.tick >= matchStartTick);
  
  // Filter out events before the match start (warmup, early rounds before a restart e.g. mp_restartgame on the server)
  allEvents = allEvents.filter(event => event.tick >= matchStartTick);
  
  // Get ticks with metadata only for ticks that match events we are interested in with any fields needed for any ticks
  const allTicksArray = parseTicks(pathToDemo, 
    ["equipment_value_this_round", "cash_spent_this_round", "is_alive", "team_num", "player_name", "score", "player_steamid"], 
    allEvents.map(event => event.tick));  // Fetch all tick data once
  
  // Convert the array of tick data into a map for efficient access later
  const allTicksMap = new Map();
  allTicksArray.forEach(tick => {
    if (!allTicksMap.has(tick.tick)) {
      allTicksMap.set(tick.tick, []);
    }
    allTicksMap.get(tick.tick).push(tick);
  });
  
  //then access ticks with get from the map of already parsed ticks with allTicksMap.get()
  const gameEndTick = Math.max(...roundEndEvents.map(event => event.tick));
  const scoreboard = allTicksMap.get(gameEndTick) || [];

  // Get the player store status at the end of the game - this will include additional round level fields you can ignore
  console.log(scoreboard)

  // Get other events by filtering array:
  let shotEvents = allEvents.filter(event => event.event_name === "weapon_fire");
  let hitEvents = allEvents.filter(event => event.event_name === "player_hurt");
  let flashEvents = allEvents.filter(event => event.event_name === "player_blind");
  // ... do the same for any of the other events from eventNames as needed

  console.log(hitEvents)