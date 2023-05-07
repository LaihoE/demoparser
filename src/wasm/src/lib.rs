use itertools::Itertools;
use parser::game_events::EventField;
use parser::game_events::GameEvent;
use parser::parser_settings::ParserInputs;
use parser::variants::Variant;
use std::collections::HashMap;
use std::{io::Read, time::Instant};
use wasm_bindgen::prelude::*;
use wasm_bindgen_file_reader::WebSysFile;
use web_sys::console;
// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
use parser::parser_settings::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Example {
    pub output: Vec<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessages {
    pub output: Vec<HashMap<String, Option<String>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Skins {
    pub output: Vec<HashMap<String, Option<String>>>,
}
#[derive(Serialize, Deserialize)]
pub struct Ticks {
    pub output: HashMap<String, Vec<String>>,
}

#[wasm_bindgen]
pub fn parse_chat_messages(file: web_sys::File) -> Result<JsValue, JsError> {
    let mut wf = WebSysFile::new(file);
    let mut buf = vec![];
    wf.read_to_end(&mut buf).unwrap();

    let settings = ParserInputs {
        bytes: &buf,
        wanted_props: vec![],
        wanted_prop_og_names: vec![],
        wanted_event: Some("-".to_owned()),
        parse_ents: false,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: true,
        count_props: false,
        only_convars: false,
    };

    let mut parser = match Parser::new(settings) {
        Ok(parser) => parser,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    match parser.start() {
        Ok(_) => {}
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let mut messages = vec![];
    for i in 0..parser.chat_messages.param1.len() {
        let mut hm: HashMap<String, Option<String>> = HashMap::default();
        hm.insert(
            "entid".to_string(),
            Some(parser.chat_messages.entity_idx[i].unwrap_or(0).to_string()),
        );
        hm.insert("param1".to_string(), parser.chat_messages.param1[i].clone());
        hm.insert("param2".to_string(), parser.chat_messages.param2[i].clone());
        hm.insert("param3".to_string(), parser.chat_messages.param3[i].clone());
        hm.insert("param4".to_string(), parser.chat_messages.param4[i].clone());
        messages.push(hm);
    }

    Ok(serde_wasm_bindgen::to_value(&ChatMessages { output: messages }).unwrap())
}
#[wasm_bindgen]
pub fn parse_skins(file: web_sys::File) -> Result<JsValue, JsError> {
    let mut wf = WebSysFile::new(file);
    let mut buf = vec![];
    wf.read_to_end(&mut buf).unwrap();

    let settings = ParserInputs {
        bytes: &buf,
        wanted_props: vec![],
        wanted_prop_og_names: vec![],
        wanted_event: Some("-".to_owned()),
        parse_ents: false,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: true,
        count_props: false,
        only_convars: false,
    };

    let mut parser = match Parser::new(settings) {
        Ok(parser) => parser,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    match parser.start() {
        Ok(_) => {}
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };

    let mut messages = vec![];
    for i in 0..parser.skins.ent_idx.len() {
        let mut hm: HashMap<String, Option<String>> = HashMap::default();
        hm.insert(
            "def_index".to_string(),
            Some(parser.chat_messages.entity_idx[i].unwrap_or(0).to_string()),
        );
        hm.insert(
            "item_id".to_string(),
            Some(parser.skins.item_id[i].unwrap_or(0).to_string().clone()),
        );
        hm.insert(
            "paint_index".to_string(),
            Some(parser.skins.paint_index[i].unwrap_or(0).to_string().clone()),
        );
        hm.insert(
            "paint_seed".to_string(),
            Some(parser.skins.paint_seed[i].unwrap_or(0).to_string().clone()),
        );
        hm.insert(
            "paint_wear".to_string(),
            Some(parser.skins.paint_wear[i].unwrap_or(0).to_string().clone()),
        );
        hm.insert(
            "steamid".to_string(),
            Some(parser.skins.steamid[i].unwrap_or(0).to_string().clone()),
        );
        hm.insert(
            "custom_name".to_string(),
            parser.skins.custom_name[i].clone(),
        );
        messages.push(hm);
    }
    Ok(serde_wasm_bindgen::to_value(&Skins { output: messages }).unwrap())
}

#[wasm_bindgen]
pub fn parse_events(file: web_sys::File, event_name: Option<String>) -> Result<JsValue, JsError> {
    let mut wf = WebSysFile::new(file);
    let mut buf = vec![];
    wf.read_to_end(&mut buf).unwrap();

    let settings = ParserInputs {
        bytes: &buf,
        wanted_props: vec![],
        wanted_prop_og_names: vec![],
        wanted_event: Some(event_name.unwrap()),
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
    };

    let mut parser = match Parser::new(settings) {
        Ok(parser) => parser,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    match parser.start() {
        Ok(_) => {}
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };

    let mut js_events: Vec<HashMap<String, String>> = vec![];

    for event in parser.game_events {
        let mut js_hm_this_event: HashMap<String, String> = HashMap::default();
        for f in event.fields {
            js_hm_this_event.insert(f.name, to_string_js(f.data.unwrap_or(Variant::I32(0))));
        }
        js_events.push(js_hm_this_event);
    }
    return Ok(serde_wasm_bindgen::to_value(&Example { output: js_events }).unwrap());
}

pub fn to_string_js(val: Variant) -> String {
    match val {
        Variant::String(f) => f.to_string(),
        Variant::F32(f) => f.to_string(),
        Variant::U64(f) => f.to_string(),
        Variant::Bool(f) => f.to_string(),
        Variant::I32(f) => f.to_string(),
        _ => "Missing".to_string(),
    }
}

/// Logs a string to the browser's console
fn log_to_browser(log_msg: String) {
    console::log_1(&log_msg.into());
}
