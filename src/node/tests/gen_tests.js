
var {parseEvent, parseEvents,parseTicks, parsePlayerInfo, parseGrenades, listGameEvents, parseHeader} = require('../index');
const fs = require('fs');
const path = require('path');
const archiver = require('archiver');
import { ALL_PROPS } from './parser.test.js';



const filePath = "../python/tests/data/test.dem"

function genEventsTestData(){
  let events = parseEvents(filePath, ["all"], ALL_PROPS)
  const jsonData = JSON.stringify(events, null, 6);
  const fileName = 'tests/data/events.json';
  fs.writeFileSync(fileName, jsonData);
}
function genEventTestData(){
  let event = parseEvent(filePath, "player_death", ALL_PROPS)
  const jsonData = JSON.stringify(event, null, 6);
  const fileName = 'tests/data/event.json';
  fs.writeFileSync(fileName, jsonData);
}
function genTicksTestData(){
  const wantedTicks = Array.from({ length: 100000 }, (_, x) => x).filter(x => x % 100 === 0);
  let events = parseTicks(filePath, ALL_PROPS, wantedTicks)
  const jsonData = JSON.stringify(events, null, 6);
  const fileName = 'tests/data/ticks.json';
  fs.writeFileSync(fileName, jsonData);
}
function genListGameEvents(){
  let events = listGameEvents(filePath)
  events.sort();
  const jsonData = JSON.stringify(events, null, 6);
  const fileName = 'tests/data/list_game_events.json';
  fs.writeFileSync(fileName, jsonData);
}
function genParseHeader(){
  let events = parseHeader(filePath)
  const jsonData = JSON.stringify(events, null, 6);
  const fileName = 'tests/data/header.json';
  fs.writeFileSync(fileName, jsonData);
}
function genPlayerInfo(){
  let events = parsePlayerInfo(filePath)
  const jsonData = JSON.stringify(events, null, 6);
  const fileName = 'tests/data/player_info.json';
  fs.writeFileSync(fileName, jsonData);
}
function genGrenades(){
  let events = parseGrenades(filePath)
  const jsonData = JSON.stringify(events, null, 6);
  const fileName = 'tests/data/grenades.json';
  fs.writeFileSync(fileName, jsonData);
}
function zipAll(){
  const output = fs.createWriteStream('tests/zipped_testdata.zip');
  const archive = archiver('zip', {
    zlib: { level: 6 }
  });
  archive.pipe(output);
  archive.directory("tests/data/", false);
  archive.finalize();
  output.on('close', () => {
    console.log('Archive created successfully.');
  });
  output.on('error', (err) => {
    console.error('Error creating archive:', err);
  });
}

genEventsTestData();
genListGameEvents();
genTicksTestData();
genEventTestData();
genParseHeader();
genPlayerInfo();
genGrenades();
zipAll();
