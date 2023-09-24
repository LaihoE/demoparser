
var {parseEvent, parseEvents,parseTicks, parsePlayerInfo, parseGrenades, listGameEvents, parseHeader} = require('../index');
const fs = require('fs');


const filePath = "../python/tests/test.dem"
const wantedTicks = Array.from({ length: 100000 }, (_, x) => x).filter(x => x % 100 === 0);


test('parse_event_with_props', () => {
    let event_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/eventWithProps.json")));
    let x = parseEvent(filePath, "player_death", ["X", "Y"], ["game_time", "total_rounds_played"])
    let event = JSON.stringify(x);
    expect(event).toBe(event_correct);
});
test('parse_events_with_props', () => {
    let event_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/eventsWithProps.json")));
    let x = parseEvents(filePath, ["all"], ["X", "Y"], ["game_time", "total_rounds_played"])
    let event = JSON.stringify(x);
    expect(event).toBe(event_correct);
});
test('list_game_events', () => {
    let correct_events = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/list_game_events.json")));
    let events_arr = listGameEvents(filePath);
    events_arr.sort();
    expect(JSON.stringify(events_arr)).toBe(correct_events);
});
test('parse_header', () => {
    let correct_events = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/header.json")));
    let event = parseHeader(filePath)
    expect(JSON.stringify(event)).toBe(correct_events);
});

test('parse_grenades', () => {
    let correct_events = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/grenades.json")));
    let event = parseGrenades(filePath)
    expect(JSON.stringify(event)).toBe(correct_events);
});

test('player_info', () => {
    let correct_events = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/player_info.json")));
    let event = parsePlayerInfo(filePath)
    expect(JSON.stringify(event)).toBe(correct_events);
});


// TICKS
test('active_weapon_skin', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/active_weapon_skin.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["active_weapon_skin"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('FORWARD', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/FORWARD.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["FORWARD"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('LEFT', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/LEFT.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["LEFT"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('RIGHT', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/RIGHT.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["RIGHT"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('BACK', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/BACK.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["BACK"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('FIRE', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/FIRE.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["FIRE"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('RIGHTCLICK', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/RIGHTCLICK.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["RIGHTCLICK"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('RELOAD', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/RELOAD.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["RELOAD"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('INSPECT', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/INSPECT.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["INSPECT"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('USE', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/USE.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["USE"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ZOOM', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ZOOM.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ZOOM"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('SCOREBOARD', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/SCOREBOARD.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["SCOREBOARD"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('WALK', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/WALK.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["WALK"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('pitch', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/pitch.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["pitch"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('yaw', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/yaw.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["yaw"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('game_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/game_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["game_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('rank', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/rank.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["rank"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('rank_if_win', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/rank_if_win.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["rank_if_win"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('rank_if_loss', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/rank_if_loss.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["rank_if_loss"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('rank_if_tie', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/rank_if_tie.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["rank_if_tie"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('mvps', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/mvps.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["mvps"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('active_weapon_original_owner', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/active_weapon_original_owner.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["active_weapon_original_owner"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('buttons', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/buttons.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["buttons"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_surrendered', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_surrendered.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_surrendered"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_rounds_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_rounds_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_rounds_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_score_overtime', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_score_overtime.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_score_overtime"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_match_stat', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_match_stat.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_match_stat"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_num_map_victories', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_num_map_victories.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_num_map_victories"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_score_first_half', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_score_first_half.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_score_first_half"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_score_second_half', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_score_second_half.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_score_second_half"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_clan_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_clan_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_clan_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_freeze_period', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_freeze_period.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_freeze_period"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_warmup_period', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_warmup_period.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_warmup_period"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('warmup_period_end', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/warmup_period_end.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["warmup_period_end"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('warmup_period_start', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/warmup_period_start.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["warmup_period_start"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_terrorist_timeout', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_terrorist_timeout.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_terrorist_timeout"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_ct_timeout', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_ct_timeout.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_ct_timeout"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('terrorist_timeout_remaining', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/terrorist_timeout_remaining.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["terrorist_timeout_remaining"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ct_timeout_remaining', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ct_timeout_remaining.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ct_timeout_remaining"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('num_terrorist_timeouts', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/num_terrorist_timeouts.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["num_terrorist_timeouts"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('num_ct_timeouts', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/num_ct_timeouts.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["num_ct_timeouts"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_technical_timeout', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_technical_timeout.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_technical_timeout"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_waiting_for_resume', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_waiting_for_resume.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_waiting_for_resume"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('match_start_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/match_start_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["match_start_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('round_start_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/round_start_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["round_start_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('restart_round_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/restart_round_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["restart_round_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_game_restart?', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_game_restart?.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_game_restart?"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('game_start_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/game_start_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["game_start_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('time_until_next_phase_start', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/time_until_next_phase_start.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["time_until_next_phase_start"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('game_phase', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/game_phase.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["game_phase"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('total_rounds_played', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/total_rounds_played.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["total_rounds_played"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('rounds_played_this_phase', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/rounds_played_this_phase.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["rounds_played_this_phase"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('hostages_remaining', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/hostages_remaining.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["hostages_remaining"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('any_hostages_reached', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/any_hostages_reached.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["any_hostages_reached"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('has_bombites', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/has_bombites.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["has_bombites"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('has_rescue_zone', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/has_rescue_zone.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["has_rescue_zone"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('has_buy_zone', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/has_buy_zone.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["has_buy_zone"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_matchmaking', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_matchmaking.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_matchmaking"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('match_making_mode', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/match_making_mode.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["match_making_mode"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_valve_dedicated_server', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_valve_dedicated_server.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_valve_dedicated_server"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('gungame_prog_weap_ct', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/gungame_prog_weap_ct.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["gungame_prog_weap_ct"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('gungame_prog_weap_t', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/gungame_prog_weap_t.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["gungame_prog_weap_t"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('spectator_slot_count', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/spectator_slot_count.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["spectator_slot_count"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_match_started', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_match_started.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_match_started"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('n_best_of_maps', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/n_best_of_maps.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["n_best_of_maps"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_bomb_dropped', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_bomb_dropped.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_bomb_dropped"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_bomb_planed', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_bomb_planed.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_bomb_planed"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('round_win_status', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/round_win_status.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["round_win_status"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('round_win_reason', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/round_win_reason.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["round_win_reason"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('terrorist_cant_buy', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/terrorist_cant_buy.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["terrorist_cant_buy"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ct_cant_buy', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ct_cant_buy.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ct_cant_buy"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('num_player_alive_ct', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/num_player_alive_ct.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["num_player_alive_ct"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('num_player_alive_t', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/num_player_alive_t.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["num_player_alive_t"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ct_losing_streak', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ct_losing_streak.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ct_losing_streak"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('t_losing_streak', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/t_losing_streak.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["t_losing_streak"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('survival_start_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/survival_start_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["survival_start_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('round_in_progress', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/round_in_progress.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["round_in_progress"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('i_bomb_site?', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/i_bomb_site?.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["i_bomb_site?"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_auto_muted', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_auto_muted.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_auto_muted"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('crosshair_code', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/crosshair_code.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["crosshair_code"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('pending_team_num', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/pending_team_num.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["pending_team_num"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('player_color', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/player_color.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["player_color"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ever_played_on_team', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ever_played_on_team.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ever_played_on_team"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('clan_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/clan_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["clan_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_coach_team', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_coach_team.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_coach_team"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('comp_wins', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/comp_wins.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["comp_wins"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('comp_rank_type', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/comp_rank_type.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["comp_rank_type"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_controlling_bot', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_controlling_bot.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_controlling_bot"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('has_controlled_bot_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/has_controlled_bot_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["has_controlled_bot_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('can_control_bot', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/can_control_bot.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["can_control_bot"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_alive', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_alive.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_alive"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('armor', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/armor.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["armor"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('has_defuser', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/has_defuser.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["has_defuser"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('has_helmet', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/has_helmet.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["has_helmet"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('spawn_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/spawn_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["spawn_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('death_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/death_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["death_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('score', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/score.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["score"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_connected', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_connected.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_connected"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('player_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/player_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["player_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('player_steamid', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/player_steamid.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["player_steamid"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fov', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fov.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fov"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('balance', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/balance.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["balance"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('start_balance', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/start_balance.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["start_balance"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('total_cash_spent', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/total_cash_spent.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["total_cash_spent"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('cash_spent_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/cash_spent_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["cash_spent_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('music_kit_id', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/music_kit_id.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["music_kit_id"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('leader_honors', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/leader_honors.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["leader_honors"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('teacher_honors', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/teacher_honors.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["teacher_honors"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('friendly_honors', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/friendly_honors.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["friendly_honors"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('kills_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/kills_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["kills_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('deaths_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/deaths_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["deaths_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('assists_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/assists_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["assists_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('alive_time_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/alive_time_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["alive_time_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('headshot_kills_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/headshot_kills_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["headshot_kills_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('damage_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/damage_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["damage_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('objective_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/objective_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["objective_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('utility_damage_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/utility_damage_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["utility_damage_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('enemies_flashed_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/enemies_flashed_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["enemies_flashed_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('equipment_value_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/equipment_value_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["equipment_value_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('money_saved_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/money_saved_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["money_saved_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('kill_reward_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/kill_reward_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["kill_reward_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('cash_earned_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/cash_earned_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["cash_earned_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('kills_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/kills_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["kills_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('deaths_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/deaths_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["deaths_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('assists_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/assists_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["assists_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('alive_time_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/alive_time_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["alive_time_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('headshot_kills_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/headshot_kills_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["headshot_kills_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ace_rounds_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ace_rounds_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ace_rounds_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('4k_rounds_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/4k_rounds_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["4k_rounds_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('3k_rounds_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/3k_rounds_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["3k_rounds_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('damage_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/damage_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["damage_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('objective_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/objective_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["objective_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('utility_damage_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/utility_damage_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["utility_damage_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('enemies_flashed_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/enemies_flashed_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["enemies_flashed_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('equipment_value_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/equipment_value_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["equipment_value_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('money_saved_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/money_saved_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["money_saved_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('kill_reward_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/kill_reward_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["kill_reward_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('cash_earned_total', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/cash_earned_total.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["cash_earned_total"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ping', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ping.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ping"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('move_collide', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/move_collide.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["move_collide"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('move_type', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/move_type.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["move_type"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('team_num', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/team_num.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["team_num"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('active_weapon', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/active_weapon.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["active_weapon"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('looking_at_weapon', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/looking_at_weapon.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["looking_at_weapon"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('holding_look_at_weapon', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/holding_look_at_weapon.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["holding_look_at_weapon"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('next_attack_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/next_attack_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["next_attack_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('duck_time_ms', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/duck_time_ms.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["duck_time_ms"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('max_speed', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/max_speed.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["max_speed"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('max_fall_velo', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/max_fall_velo.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["max_fall_velo"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('duck_amount', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/duck_amount.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["duck_amount"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('duck_speed', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/duck_speed.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["duck_speed"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('duck_overrdie', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/duck_overrdie.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["duck_overrdie"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('old_jump_pressed', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/old_jump_pressed.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["old_jump_pressed"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('jump_until', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/jump_until.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["jump_until"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('jump_velo', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/jump_velo.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["jump_velo"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fall_velo', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fall_velo.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fall_velo"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('in_crouch', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/in_crouch.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["in_crouch"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('crouch_state', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/crouch_state.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["crouch_state"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ducked', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ducked.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ducked"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ducking', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ducking.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ducking"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('in_duck_jump', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/in_duck_jump.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["in_duck_jump"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('allow_auto_movement', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/allow_auto_movement.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["allow_auto_movement"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('jump_time_ms', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/jump_time_ms.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["jump_time_ms"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('last_duck_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/last_duck_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["last_duck_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_rescuing', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_rescuing.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_rescuing"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('weapon_purchases_this_match', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/weapon_purchases_this_match.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["weapon_purchases_this_match"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('weapon_purchases_this_round', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/weapon_purchases_this_round.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["weapon_purchases_this_round"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('spotted', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/spotted.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["spotted"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('approximate_spotted_by', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/approximate_spotted_by.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["approximate_spotted_by"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('time_last_injury', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/time_last_injury.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["time_last_injury"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('direction_last_injury', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/direction_last_injury.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["direction_last_injury"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('player_state', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/player_state.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["player_state"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('passive_items', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/passive_items.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["passive_items"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_scoped', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_scoped.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_scoped"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_walking', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_walking.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_walking"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('resume_zoom', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/resume_zoom.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["resume_zoom"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_defusing', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_defusing.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_defusing"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_grabbing_hostage', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_grabbing_hostage.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_grabbing_hostage"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('blocking_use_in_progess', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/blocking_use_in_progess.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["blocking_use_in_progess"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('molotov_damage_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/molotov_damage_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["molotov_damage_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('moved_since_spawn', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/moved_since_spawn.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["moved_since_spawn"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('in_bomb_zone', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/in_bomb_zone.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["in_bomb_zone"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('in_buy_zone', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/in_buy_zone.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["in_buy_zone"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('in_no_defuse_area', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/in_no_defuse_area.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["in_no_defuse_area"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('killed_by_taser', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/killed_by_taser.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["killed_by_taser"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('move_state', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/move_state.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["move_state"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('which_bomb_zone', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/which_bomb_zone.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["which_bomb_zone"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('in_hostage_rescue_zone', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/in_hostage_rescue_zone.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["in_hostage_rescue_zone"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('stamina', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/stamina.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["stamina"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('direction', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/direction.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["direction"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('shots_fired', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/shots_fired.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["shots_fired"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('armor_value', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/armor_value.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["armor_value"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('velo_modifier', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/velo_modifier.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["velo_modifier"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('ground_accel_linear_frac_last_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/ground_accel_linear_frac_last_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["ground_accel_linear_frac_last_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('flash_duration', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/flash_duration.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["flash_duration"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('flash_max_alpha', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/flash_max_alpha.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["flash_max_alpha"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('wait_for_no_attack', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/wait_for_no_attack.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["wait_for_no_attack"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('last_place_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/last_place_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["last_place_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_strafing', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_strafing.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_strafing"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('round_start_equip_value', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/round_start_equip_value.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["round_start_equip_value"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('current_equip_value', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/current_equip_value.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["current_equip_value"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('health', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/health.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["health"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('life_state', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/life_state.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["life_state"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('X', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/X.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["X"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('Y', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/Y.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["Y"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('Z', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/Z.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["Z"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('active_weapon_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/active_weapon_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["active_weapon_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('active_weapon_ammo', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/active_weapon_ammo.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["active_weapon_ammo"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('total_ammo_left', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/total_ammo_left.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["total_ammo_left"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('item_def_idx', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/item_def_idx.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["item_def_idx"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('weapon_quality', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/weapon_quality.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["weapon_quality"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('entity_lvl', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/entity_lvl.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["entity_lvl"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('item_id_high', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/item_id_high.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["item_id_high"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('item_id_low', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/item_id_low.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["item_id_low"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('item_account_id', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/item_account_id.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["item_account_id"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('inventory_position', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/inventory_position.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["inventory_position"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_initialized', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_initialized.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_initialized"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('econ_item_attribute_def_idx', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/econ_item_attribute_def_idx.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["econ_item_attribute_def_idx"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('econ_raw_val_32', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/econ_raw_val_32.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["econ_raw_val_32"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('initial_value', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/initial_value.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["initial_value"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('refundable_currency', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/refundable_currency.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["refundable_currency"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('set_bonus', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/set_bonus.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["set_bonus"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('custom_name', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/custom_name.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["custom_name"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('orig_owner_xuid_low', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/orig_owner_xuid_low.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["orig_owner_xuid_low"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('orig_owner_xuid_high', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/orig_owner_xuid_high.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["orig_owner_xuid_high"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fall_back_paint_kit', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fall_back_paint_kit.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fall_back_paint_kit"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fall_back_seed', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fall_back_seed.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fall_back_seed"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fall_back_wear', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fall_back_wear.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fall_back_wear"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fall_back_stat_track', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fall_back_stat_track.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fall_back_stat_track"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('m_iState', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/m_iState.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["m_iState"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fire_seq_start_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fire_seq_start_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fire_seq_start_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fire_seq_start_time_change', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fire_seq_start_time_change.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fire_seq_start_time_change"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_player_fire_event_primary', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_player_fire_event_primary.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_player_fire_event_primary"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('weapon_mode', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/weapon_mode.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["weapon_mode"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('accuracy_penalty', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/accuracy_penalty.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["accuracy_penalty"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('i_recoil_idx', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/i_recoil_idx.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["i_recoil_idx"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('fl_recoil_idx', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/fl_recoil_idx.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["fl_recoil_idx"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_burst_mode', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_burst_mode.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_burst_mode"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('post_pone_fire_ready_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/post_pone_fire_ready_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["post_pone_fire_ready_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_in_reload', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_in_reload.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_in_reload"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('reload_visually_complete', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/reload_visually_complete.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["reload_visually_complete"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('dropped_at_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/dropped_at_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["dropped_at_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_hauled_back', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_hauled_back.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_hauled_back"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('is_silencer_on', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/is_silencer_on.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["is_silencer_on"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('time_silencer_switch_complete', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/time_silencer_switch_complete.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["time_silencer_switch_complete"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('orig_team_number', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/orig_team_number.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["orig_team_number"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('prev_owner', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/prev_owner.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["prev_owner"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('last_shot_time', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/last_shot_time.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["last_shot_time"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('iron_sight_mode', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/iron_sight_mode.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["iron_sight_mode"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('num_empty_attacks', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/num_empty_attacks.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["num_empty_attacks"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('zoom_lvl', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/zoom_lvl.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["zoom_lvl"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('burst_shots_remaining', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/burst_shots_remaining.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["burst_shots_remaining"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('needs_bolt_action', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/needs_bolt_action.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["needs_bolt_action"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('next_primary_attack_tick', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/next_primary_attack_tick.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["next_primary_attack_tick"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('next_primary_attack_tick_ratio', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/next_primary_attack_tick_ratio.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["next_primary_attack_tick_ratio"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('next_secondary_attack_tick', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/next_secondary_attack_tick.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["next_secondary_attack_tick"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('next_secondary_attack_tick_ratio', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/next_secondary_attack_tick_ratio.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["next_secondary_attack_tick_ratio"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
test('inventory', () => {
    let tick_correct = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_prop/inventory.json")));
    let ticks = JSON.stringify(parseTicks(filePath, ["inventory"], wantedTicks));
    expect(ticks).toBe(tick_correct);
});
// EVENT
// EVENT
// EVENT
// EVENT
// EVENT
test('smokegrenade_expired', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/smokegrenade_expired.json")));
    let event = JSON.stringify(parseEvent(filePath, "smokegrenade_expired"));
    expect(event).toBe(eventCorrect);
});
test('player_disconnect', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_disconnect.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_disconnect"));
    expect(event).toBe(eventCorrect);
});
test('round_freeze_end', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_freeze_end.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_freeze_end"));
    expect(event).toBe(eventCorrect);
});
test('hegrenade_detonate', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/hegrenade_detonate.json")));
    let event = JSON.stringify(parseEvent(filePath, "hegrenade_detonate"));
    expect(event).toBe(eventCorrect);
});
test('weapon_zoom', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/weapon_zoom.json")));
    let event = JSON.stringify(parseEvent(filePath, "weapon_zoom"));
    expect(event).toBe(eventCorrect);
});
test('round_officially_ended', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_officially_ended.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_officially_ended"));
    expect(event).toBe(eventCorrect);
});
test('cs_win_panel_round', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/cs_win_panel_round.json")));
    let event = JSON.stringify(parseEvent(filePath, "cs_win_panel_round"));
    expect(event).toBe(eventCorrect);
});
test('smokegrenade_detonate', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/smokegrenade_detonate.json")));
    let event = JSON.stringify(parseEvent(filePath, "smokegrenade_detonate"));
    expect(event).toBe(eventCorrect);
});
test('inferno_expire', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/inferno_expire.json")));
    let event = JSON.stringify(parseEvent(filePath, "inferno_expire"));
    expect(event).toBe(eventCorrect);
});
test('bomb_planted', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/bomb_planted.json")));
    let event = JSON.stringify(parseEvent(filePath, "bomb_planted"));
    expect(event).toBe(eventCorrect);
});
test('hltv_versioninfo', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/hltv_versioninfo.json")));
    let event = JSON.stringify(parseEvent(filePath, "hltv_versioninfo"));
    expect(event).toBe(eventCorrect);
});
test('announce_phase_end', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/announce_phase_end.json")));
    let event = JSON.stringify(parseEvent(filePath, "announce_phase_end"));
    expect(event).toBe(eventCorrect);
});
test('player_team', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_team.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_team"));
    expect(event).toBe(eventCorrect);
});
test('item_pickup', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/item_pickup.json")));
    let event = JSON.stringify(parseEvent(filePath, "item_pickup"));
    expect(event).toBe(eventCorrect);
});
test('item_equip', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/item_equip.json")));
    let event = JSON.stringify(parseEvent(filePath, "item_equip"));
    expect(event).toBe(eventCorrect);
});
test('bomb_pickup', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/bomb_pickup.json")));
    let event = JSON.stringify(parseEvent(filePath, "bomb_pickup"));
    expect(event).toBe(eventCorrect);
});
test('player_jump', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_jump.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_jump"));
    expect(event).toBe(eventCorrect);
});
test('cs_pre_restart', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/cs_pre_restart.json")));
    let event = JSON.stringify(parseEvent(filePath, "cs_pre_restart"));
    expect(event).toBe(eventCorrect);
});
test('cs_round_start_beep', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/cs_round_start_beep.json")));
    let event = JSON.stringify(parseEvent(filePath, "cs_round_start_beep"));
    expect(event).toBe(eventCorrect);
});
test('player_hurt', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_hurt.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_hurt"));
    expect(event).toBe(eventCorrect);
});
test('round_start', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_start.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_start"));
    expect(event).toBe(eventCorrect);
});
test('cs_round_final_beep', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/cs_round_final_beep.json")));
    let event = JSON.stringify(parseEvent(filePath, "cs_round_final_beep"));
    expect(event).toBe(eventCorrect);
});
test('buytime_ended', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/buytime_ended.json")));
    let event = JSON.stringify(parseEvent(filePath, "buytime_ended"));
    expect(event).toBe(eventCorrect);
});
test('inferno_startburn', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/inferno_startburn.json")));
    let event = JSON.stringify(parseEvent(filePath, "inferno_startburn"));
    expect(event).toBe(eventCorrect);
});
test('flashbang_detonate', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/flashbang_detonate.json")));
    let event = JSON.stringify(parseEvent(filePath, "flashbang_detonate"));
    expect(event).toBe(eventCorrect);
});
test('round_time_warning', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_time_warning.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_time_warning"));
    expect(event).toBe(eventCorrect);
});
test('round_announce_last_round_half', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_announce_last_round_half.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_announce_last_round_half"));
    expect(event).toBe(eventCorrect);
});
test('hltv_message', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/hltv_message.json")));
    let event = JSON.stringify(parseEvent(filePath, "hltv_message"));
    expect(event).toBe(eventCorrect);
});
test('cs_win_panel_match', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/cs_win_panel_match.json")));
    let event = JSON.stringify(parseEvent(filePath, "cs_win_panel_match"));
    expect(event).toBe(eventCorrect);
});
test('weapon_fire', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/weapon_fire.json")));
    let event = JSON.stringify(parseEvent(filePath, "weapon_fire"));
    expect(event).toBe(eventCorrect);
});
test('begin_new_match', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/begin_new_match.json")));
    let event = JSON.stringify(parseEvent(filePath, "begin_new_match"));
    expect(event).toBe(eventCorrect);
});
test('round_announce_match_point', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_announce_match_point.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_announce_match_point"));
    expect(event).toBe(eventCorrect);
});
test('bomb_beginplant', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/bomb_beginplant.json")));
    let event = JSON.stringify(parseEvent(filePath, "bomb_beginplant"));
    expect(event).toBe(eventCorrect);
});
test('decoy_detonate', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/decoy_detonate.json")));
    let event = JSON.stringify(parseEvent(filePath, "decoy_detonate"));
    expect(event).toBe(eventCorrect);
});
test('hltv_chase', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/hltv_chase.json")));
    let event = JSON.stringify(parseEvent(filePath, "hltv_chase"));
    expect(event).toBe(eventCorrect);
});
test('round_prestart', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_prestart.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_prestart"));
    expect(event).toBe(eventCorrect);
});
test('player_spawn', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_spawn.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_spawn"));
    expect(event).toBe(eventCorrect);
});
test('player_footstep', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_footstep.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_footstep"));
    expect(event).toBe(eventCorrect);
});
test('round_announce_match_start', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_announce_match_start.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_announce_match_start"));
    expect(event).toBe(eventCorrect);
});
test('round_end', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_end.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_end"));
    expect(event).toBe(eventCorrect);
});
test('round_mvp', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_mvp.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_mvp"));
    expect(event).toBe(eventCorrect);
});
test('rank_update', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/rank_update.json")));
    let event = JSON.stringify(parseEvent(filePath, "rank_update"));
    expect(event).toBe(eventCorrect);
});
test('player_blind', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_blind.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_blind"));
    expect(event).toBe(eventCorrect);
});
test('round_poststart', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/round_poststart.json")));
    let event = JSON.stringify(parseEvent(filePath, "round_poststart"));
    expect(event).toBe(eventCorrect);
});
test('bomb_dropped', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/bomb_dropped.json")));
    let event = JSON.stringify(parseEvent(filePath, "bomb_dropped"));
    expect(event).toBe(eventCorrect);
});
test('decoy_started', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/decoy_started.json")));
    let event = JSON.stringify(parseEvent(filePath, "decoy_started"));
    expect(event).toBe(eventCorrect);
});
test('weapon_reload', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/weapon_reload.json")));
    let event = JSON.stringify(parseEvent(filePath, "weapon_reload"));
    expect(event).toBe(eventCorrect);
});
test('player_connect_full', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_connect_full.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_connect_full"));
    expect(event).toBe(eventCorrect);
});
test('player_death', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_death.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_death"));
    expect(event).toBe(eventCorrect);
});
test('player_connect', () => {
    let eventCorrect = JSON.stringify(JSON.parse(fs.readFileSync("tests/data/per_event/player_connect.json")));
    let event = JSON.stringify(parseEvent(filePath, "player_connect"));
    expect(event).toBe(eventCorrect);
});
