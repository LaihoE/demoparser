use js_sys::Error;
use parser::parser_settings::rm_user_friendly_names;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::parser_thread_settings::create_huffman_lookup_table;
use parser::variants::soa_to_aos;
use parser::variants::BytesVariant;
use parser::variants::OutputSerdeHelperStruct;
use parser::variants::Variant;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub use wasm_bindgen_rayon::init_thread_pool;
/*
extern crate wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
*/

#[wasm_bindgen]
pub fn parse(file: Vec<u8>) -> u64 {
    let x = Arc::new(file);

    let a = x.par_iter().map(|x| *x as u64).sum();
    let b = x.par_iter().map(|x| *x as u64).min().unwrap();
    let c = x.par_iter().map(|x| *x as u64).max().unwrap();
    let x = std::cmp::max(a, b);
    std::cmp::max(x, c)
}

#[wasm_bindgen]
pub fn parse_events(
    file: Vec<u8>,
    event_name: Option<String>,
    wanted_player_props: Option<Vec<JsValue>>,
    wanted_other_props: Option<Vec<JsValue>>,
) -> Result<JsValue, JsError> {
    console::log_1(&"Hello using web-sys".into());

    // let mut wf = WebSysFile::new(file);
    // let mut buf = Vec::with_capacity(wf.size() as usize);
    // let mut buf = vec![];
    console::log_1(&"Hello using web-sys2".into());

    //wf.rea_to_end(&mut buf).unwrap();
    let x = format!("FILE LENGTH {:?}", file.len().to_string());
    let y = x.as_str();
    let jv = JsValue::from_str(y);
    console::log_1(&jv);

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
        bytes: Arc::new(BytesVariant::Vec(file)),
        wanted_player_props: real_names_player,
        wanted_player_props_og_names: vec![],
        wanted_other_props: vec![],
        wanted_other_props_og_names: real_other_props,
        real_name_to_og_name: real_name_to_og_name.into(),
        wanted_events: vec![event_name.unwrap_or("none".to_string())],
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
        huffman_lookup_table: arc_huf,
    };
    let mut parser = Parser::new(settings);
    parser.is_multithreadable = true;

    let output = match parser.parse_demo() {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    match serde_wasm_bindgen::to_value(&output.game_events) {
        Ok(s) => Ok(s),
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    }
}
/*
#[wasm_bindgen]
pub fn parse_ticks(
    file: web_sys::File,
    wanted_props: Option<Vec<JsValue>>,
    wanted_ticks: Option<Vec<i32>>,
    struct_of_arrays: Option<bool>,
) -> Result<JsValue, JsError> {
    let wanted_props = match wanted_props {
        Some(p) => p.iter().map(|s| s.as_string().unwrap()).collect::<Vec<_>>(),
        None => vec![],
    };

    let mut real_names = match rm_user_friendly_names(&wanted_props) {
        Ok(names) => names,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    let mut wf = WebSysFile::new(file);
    let mut buf = Vec::with_capacity(wf.size() as usize);
    wf.read_to_end(&mut buf).unwrap();

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
        real_name_to_og_name: real_name_to_og_name.into(),
        bytes: Arc::new(BytesVariant::Vec(buf)),
        wanted_player_props: real_names.clone(),
        wanted_player_props_og_names: wanted_props.clone(),
        wanted_other_props: vec![],
        wanted_other_props_og_names: vec![],
        wanted_events: vec![],
        parse_ents: true,
        wanted_ticks: wanted_ticks,
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
        huffman_lookup_table: arc_huf.clone(),
    };
    let mut parser = Parser::new(settings);
    parser.is_multithreadable = false;
    let output = match parser.parse_demo() {
        Ok(output) => output,
        Err(e) => return Err(JsError::new(&format!("{}", e))),
    };
    real_names.push("tick".to_owned());
    real_names.push("steamid".to_owned());
    real_names.push("name".to_owned());

    let mut prop_infos = output.prop_info.prop_infos.clone();
    prop_infos.sort_by_key(|x| x.prop_name.clone());
    real_names.sort();

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
*/
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
