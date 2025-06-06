use parser::first_pass::parser_settings::rm_user_friendly_names;
use parser::first_pass::parser_settings::FirstPassParser;
use parser::first_pass::parser_settings::ParserInputs;
use parser::parse_demo::Parser;
use parser::parse_demo::ParsingMode::ForceSingleThreaded;
use parser::second_pass::parser_settings::create_huffman_lookup_table;
use parser::second_pass::variants::soa_to_aos;
use parser::second_pass::variants::OutputSerdeHelperStruct;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::result::Result;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parseEvent(
    file: Vec<u8>,
    event_name: Option<String>,
    wanted_player_props: Option<Vec<JsValue>>,
    wanted_other_props: Option<Vec<JsValue>>,
) -> Result<JsValue, JsError> {
    let player_props = match wanted_player_props {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let other_props = match wanted_other_props {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let real_names_player = match rm_user_friendly_names(&player_props) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let real_other_props = match rm_user_friendly_names(&other_props) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };

    let mut real_name_to_og_name = HashMap::default();
    for (real_name, user_friendly_name) in real_names_player.iter().zip(&player_props) {
        real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
    }
    for (real_name, user_friendly_name) in real_other_props.iter().zip(&other_props) {
        real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
    }
    let arc_huf = Arc::new(create_huffman_lookup_table());
    let settings = ParserInputs {
        wanted_players: vec![],
        wanted_player_props: real_names_player,
        wanted_other_props: real_other_props,
        real_name_to_og_name: real_name_to_og_name.into(),
        wanted_events: vec![event_name.unwrap_or("none".to_string())],
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &arc_huf,
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = Parser::new(settings, ForceSingleThreaded);

    let output = match parser.parse_demo(&file) {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    match serde_wasm_bindgen::to_value(&output.game_events) {
        Ok(s) => Ok(s),
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    }
}
#[wasm_bindgen]
pub fn parseEvents(
    file: Vec<u8>,
    event_names: Option<Vec<JsValue>>,
    wanted_player_props: Option<Vec<JsValue>>,
    wanted_other_props: Option<Vec<JsValue>>,
) -> Result<JsValue, JsError> {
    let event_names = match event_names {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let player_props = match wanted_player_props {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let other_props = match wanted_other_props {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let real_names_player = match rm_user_friendly_names(&player_props) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let real_other_props = match rm_user_friendly_names(&other_props) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };

    let mut real_name_to_og_name = HashMap::default();
    for (real_name, user_friendly_name) in real_names_player.iter().zip(&player_props) {
        real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
    }
    for (real_name, user_friendly_name) in real_other_props.iter().zip(&other_props) {
        real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
    }
    let arc_huf = Arc::new(create_huffman_lookup_table());
    let settings = ParserInputs {
        wanted_players: vec![],
        wanted_player_props: real_names_player,
        wanted_other_props: real_other_props,
        real_name_to_og_name: real_name_to_og_name.into(),
        wanted_events: event_names,
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &arc_huf,
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = Parser::new(settings, ForceSingleThreaded);

    let output = match parser.parse_demo(&file) {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    match serde_wasm_bindgen::to_value(&output.game_events) {
        Ok(s) => Ok(s),
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    }
}

#[wasm_bindgen]
pub fn listGameEvents(fileBytes: Vec<u8>) -> Result<JsValue, JsError> {
    let arc_huf = Arc::new(create_huffman_lookup_table());
    let settings = ParserInputs {
        wanted_players: vec![],
        real_name_to_og_name: HashMap::default().into(),
        wanted_player_props: vec![],
        wanted_other_props: vec![],
        wanted_events: vec!["all".to_string()],
        parse_ents: false,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &arc_huf.clone(),
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = Parser::new(settings, ForceSingleThreaded);

    let output = match parser.parse_demo(&fileBytes) {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let v = Vec::from_iter(output.game_events_counter.iter());
    match serde_wasm_bindgen::to_value(&v) {
        Ok(s) => Ok(s),
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    }
}
#[wasm_bindgen]
pub fn listUpdatedFields(fileBytes: Vec<u8>) -> Result<JsValue, JsError> {
    let arc_huf = Arc::new(create_huffman_lookup_table());
    let settings = ParserInputs {
        wanted_players: vec![],
        real_name_to_og_name: HashMap::default().into(),
        wanted_player_props: vec![],
        wanted_other_props: vec![],
        wanted_events: vec!["none".to_string()],
        parse_ents: false,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        list_props: true,
        only_convars: false,
        huffman_lookup_table: &arc_huf.clone(),
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = Parser::new(settings, ForceSingleThreaded);

    let output = match parser.parse_demo(&fileBytes) {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let v = Vec::from_iter(output.game_events_counter.iter());
    match serde_wasm_bindgen::to_value(&v) {
        Ok(s) => Ok(s),
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    }
}

#[wasm_bindgen]
pub fn parseTicks(
    file: Vec<u8>,
    wanted_props: Option<Vec<JsValue>>,
    wanted_ticks: Option<Vec<i32>>,
    wanted_players: Option<Vec<JsValue>>,
    struct_of_arrays: Option<bool>,
) -> Result<JsValue, JsError> {
    let wanted_props = match wanted_props {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let wanted_players_u64 = match wanted_players {
        Some(v) => v
            .iter()
            .map(|x| x.as_string().unwrap().parse::<u64>().unwrap_or(0))
            .collect(),
        None => vec![],
    };
    let mut real_names = match rm_user_friendly_names(&wanted_props) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let arc_huf = Arc::new(create_huffman_lookup_table());
    let mut real_name_to_og_name = HashMap::default();
    for (real_name, user_friendly_name) in real_names.iter().zip(&wanted_props) {
        real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
    }
    let wanted_ticks = match wanted_ticks {
        Some(t) => t,
        None => vec![],
    };
    let settings = ParserInputs {
        wanted_players: wanted_players_u64,
        real_name_to_og_name: real_name_to_og_name.into(),
        wanted_player_props: real_names.clone(),
        wanted_other_props: vec![],
        wanted_events: vec![],
        parse_ents: true,
        wanted_ticks: wanted_ticks,
        parse_projectiles: false,
        only_header: false,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &arc_huf.clone(),
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = Parser::new(settings, ForceSingleThreaded);

    let output = match parser.parse_demo(&file) {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let mut prop_infos = output.prop_controller.prop_infos.clone();
    prop_infos.sort_by_key(|x| x.prop_name.clone());

    let helper = OutputSerdeHelperStruct {
        prop_infos: prop_infos,
        inner: output.df.into(),
    };

    let is_soa = match struct_of_arrays {
        Some(true) => true,
        _ => false,
    };

    if is_soa {
        let s = match serde_wasm_bindgen::to_value(&helper) {
            Ok(s) => s,
            Err(e) => return Err(JsError::new(&format!("{}", e))),
        };
        return Ok(s);
    } else {
        let result = soa_to_aos(helper);
        let s = match serde_wasm_bindgen::to_value(&result) {
            Ok(s) => s,
            Err(e) => return Err(JsError::new(&format!("{}", e))),
        };
        Ok(s)
    }
}

#[wasm_bindgen]
pub fn parseGrenades(file: Vec<u8>, extra: Option<Vec<JsValue>>) -> Result<JsValue, JsError> {
    let mut extra = match extra {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };
    let mut real_names = match rm_user_friendly_names(&extra) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let arc_huf = Arc::new(create_huffman_lookup_table());
    let mut real_name_to_og_name = HashMap::default();
    for (real_name, user_friendly_name) in real_names.iter().zip(&extra) {
        real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
    }
    let settings = ParserInputs {
        wanted_players: vec![],
        real_name_to_og_name: real_name_to_og_name.into(),
        wanted_player_props: real_names.clone(),
        wanted_other_props: vec![],
        wanted_events: vec![],
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &arc_huf.clone(),
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = Parser::new(settings, ForceSingleThreaded);

    let output = match parser.parse_demo(&file) {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let mut prop_infos = output.prop_controller.prop_infos.clone();
    prop_infos.sort_by_key(|x| x.prop_name.clone());
    let helper = OutputSerdeHelperStruct {
        prop_infos: prop_infos,
        inner: output.df.into(),
    };
    let result = soa_to_aos(helper);
    let s = match serde_wasm_bindgen::to_value(&result) {
        Ok(s) => s,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    Ok(s)
}

#[wasm_bindgen]
pub fn parseHeader(file: Vec<u8>) -> Result<JsValue, JsError> {
    let arc_huf = Arc::new(create_huffman_lookup_table());

    let settings = ParserInputs {
        wanted_players: vec![],
        real_name_to_og_name: HashMap::default().into(),
        wanted_player_props: vec![],
        wanted_other_props: vec![],
        wanted_events: vec![],
        parse_ents: false,
        wanted_ticks: vec![],
        parse_projectiles: true,
        only_header: true,
        list_props: false,
        only_convars: false,
        huffman_lookup_table: &arc_huf.clone(),
        order_by_steamid: false,
        wanted_prop_states: HashMap::default().into(),
        fallback_bytes: None,
        parse_grenades: false
    };
    let mut parser = FirstPassParser::new(&settings);
    let output = parser.parse_header_only(&file).unwrap();
    let mut hm: HashMap<String, String> = HashMap::default();
    hm.extend(output);
    match serde_wasm_bindgen::to_value(&hm) {
        Ok(s) => Ok(s),
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    }
}
