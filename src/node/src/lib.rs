#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
use ahash::AHashMap;
use memmap2::MmapOptions;
use napi::bindgen_prelude::*;
use napi::Either;
use napi::JsBigInt;
use napi::JsUnknown;
use parser::first_pass::parser_settings::rm_map_user_friendly_names;
use parser::first_pass::parser_settings::rm_user_friendly_names;
use parser::first_pass::parser_settings::ParserInputs;
use parser::parse_demo::DemoOutput;
use parser::parse_demo::Parser;
use parser::second_pass::parser_settings::create_huffman_lookup_table;
use parser::second_pass::variants::soa_to_aos;
use parser::second_pass::variants::BytesVariant;
use parser::second_pass::variants::OutputSerdeHelperStruct;
use parser::second_pass::variants::Variant;
use parser::second_pass::voice_data::convert_voice_data_to_wav;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::hash::RandomState;
use std::result::Result;

#[napi]
#[derive(Clone)]
pub struct JsVariant(Variant);

impl FromNapiValue for JsVariant {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let js_unknown = JsUnknown::from_napi_value(env, napi_val)?;

    match js_unknown.get_type() {
      Ok(js_unknown_type) => {
        if js_unknown_type == ValueType::Boolean {
          if let Ok(val) = js_unknown.coerce_to_bool() {
            Ok(JsVariant(Variant::Bool(val.get_value()?)))
          } else {
            Err(Error::new(
              Status::InvalidArg,
              "Unspported Boolean type for Variant".to_owned(),
            ))
          }
        } else if js_unknown_type == ValueType::String {
          if let Ok(val) = js_unknown.coerce_to_string() {
            Ok(JsVariant(Variant::String(val.into_utf8()?.into_owned()?)))
          } else {
            Err(Error::new(
              Status::InvalidArg,
              "Unsupported String for Variant".to_owned(),
            ))
          }
        } else if js_unknown_type == ValueType::Number {
          if let Ok(val) = js_unknown.coerce_to_number() {
            let num = val.get_double()?;
            if num.fract() == 0.0 {
              if num >= u8::MIN as f64 && num <= u8::MAX as f64 {
                Ok(JsVariant(Variant::U8(num as u8)))
              } else if let Ok(val) = val.get_int32() {
                let int32_val = val;
                if int32_val >= i16::MIN as i32 && int32_val <= i16::MAX as i32 {
                  Ok(JsVariant(Variant::I16(int32_val as i16)))
                } else {
                  Ok(JsVariant(Variant::I32(int32_val)))
                }
              } else if let Ok(val) = val.get_uint32() {
                Ok(JsVariant(Variant::U32(val)))
              } else {
                Err(Error::new(
                  Status::InvalidArg,
                  "Unsupported number type".to_owned(),
                ))
              }
            } else {
              Ok(JsVariant(Variant::F32(num as f32)))
            }
          } else {
            Err(Error::new(
              Status::InvalidArg,
              "Unsupported number type".to_owned(),
            ))
          }
        } else if js_unknown_type == ValueType::BigInt {
          let bigint_val = js_unknown.cast::<JsBigInt>();
          match bigint_val.get_u64() {
            Ok((val, true)) => Ok(JsVariant(Variant::U64(val))),
            _ => Err(Error::new(
              Status::InvalidArg,
              "Unsupported number type".to_owned(),
            )),
          }
        } else {
          Err(Error::new(
            Status::InvalidArg,
            "Unspported type for Variant".to_owned(),
          ))
        }
      }
      _ => Err(Error::new(
        Status::InvalidArg,
        "Unspported type for Variant".to_owned(),
      )),
    }
  }
}

#[napi]
pub struct WantedPropState {
  pub prop: String,
  pub state: JsVariant,
}

impl FromNapiValue for WantedPropState {
  unsafe fn from_napi_value(
    env: sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let obj: Object = Object::from_napi_value(env, napi_val)?;

    let prop: String = obj.get_named_property("prop")?;
    let state: JsVariant = obj.get_named_property("state")?;

    Ok(WantedPropState { prop, state })
  }
}

fn parse_demo(bytes: BytesVariant, parser: &mut Parser) -> Result<DemoOutput, Error> {
  match bytes {
    BytesVariant::Mmap(m) => match parser.parse_demo(&m) {
      Ok(output) => Ok(output),
      Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
    },
    BytesVariant::Vec(v) => match parser.parse_demo(&v) {
      Ok(output) => Ok(output),
      Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
    },
  }
}
#[napi]
pub fn parse_voice(path_or_buf: Either<String, Buffer>) -> napi::Result<HashMap<String, Vec<u8>>> {
  let bytes = resolve_byte_type(path_or_buf).unwrap();
  let settings = ParserInputs {
    wanted_players: vec![],
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_events: vec![],
    wanted_ticks: vec![],
    wanted_prop_states: AHashMap::default(),
    real_name_to_og_name: AHashMap::default(),
    parse_ents: false,
    parse_projectiles: false,
    only_header: false,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &vec![],
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let out = match convert_voice_data_to_wav(output.voice_data) {
    Ok(out) => out,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  let mut out_hm = HashMap::default();
  for (steamid, bytes) in out {
    out_hm.insert(steamid, bytes);
  }
  Ok(out_hm)
}

#[napi]
pub fn list_game_events(path_or_buf: Either<String, Buffer>) -> napi::Result<Value> {
  let bytes = resolve_byte_type(path_or_buf)?;

  let huf = create_huffman_lookup_table();
  let settings = ParserInputs {
    wanted_players: vec![],
    real_name_to_og_name: AHashMap::default(),
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_prop_states: AHashMap::default(),
    wanted_events: vec!["all".to_string()],
    parse_ents: false,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: false,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;

  let v = Vec::from_iter(output.game_events_counter.iter());
  let s = match serde_json::to_value(v) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}

#[napi]
pub fn parse_grenades(path_or_buf: Either<String, Buffer>) -> napi::Result<Value> {
  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    wanted_players: vec![],
    real_name_to_og_name: AHashMap::default(),
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_events: vec![],
    wanted_prop_states: AHashMap::default(),
    parse_ents: true,
    wanted_ticks: vec![],
    parse_projectiles: true,
    only_header: true,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;

  let s = match serde_json::to_value(&output.projectiles) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}
#[napi]
pub fn parse_header(path_or_buf: Either<String, Buffer>) -> napi::Result<Value> {
  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    real_name_to_og_name: AHashMap::default(),
    wanted_players: vec![],
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_prop_states: AHashMap::default(),
    wanted_events: vec![],
    parse_ents: false,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: true,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let mut hm: HashMap<String, String> = HashMap::default();

  if let Some(header) = output.header {
    hm.extend(header);
  }
  let s = match serde_json::to_value(&hm) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}

#[napi]
pub fn parse_event(
  path_or_buf: Either<String, Buffer>,
  event_name: String,
  player_extra: Option<Vec<String>>,
  other_extra: Option<Vec<String>>,
) -> napi::Result<Value> {
  let player_props = match player_extra {
    Some(p) => p,
    None => vec![],
  };
  let other_props = match other_extra {
    Some(p) => p,
    None => vec![],
  };
  let real_names_player = match rm_user_friendly_names(&player_props) {
    Ok(names) => names,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  let real_other_props = match rm_user_friendly_names(&other_props) {
    Ok(names) => names,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };

  let mut real_name_to_og_name = AHashMap::default();
  for (real_name, user_friendly_name) in real_names_player.iter().zip(&player_props) {
    real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
  }
  for (real_name, user_friendly_name) in real_other_props.iter().zip(&other_props) {
    real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
  }

  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    real_name_to_og_name: real_name_to_og_name,
    wanted_players: vec![],
    wanted_player_props: real_names_player.clone(),
    wanted_other_props: real_other_props,
    wanted_prop_states: AHashMap::default(),
    wanted_events: vec![event_name],
    parse_ents: true,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: true,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let s = match serde_json::to_value(&output.game_events) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}
#[napi]
pub fn parse_events(
  path_or_buf: Either<String, Buffer>,
  event_names: Option<Vec<String>>,
  player_extra: Option<Vec<String>>,
  other_extra: Option<Vec<String>>,
) -> napi::Result<Value> {
  let event_names = match event_names {
    None => return Err(Error::new(Status::InvalidArg, "No events provided!")),
    Some(v) => v,
  };
  let player_props = match player_extra {
    Some(p) => p,
    None => vec![],
  };
  let other_props = match other_extra {
    Some(p) => p,
    None => vec![],
  };
  let real_names_player = match rm_user_friendly_names(&player_props) {
    Ok(names) => names,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  let real_other_props = match rm_user_friendly_names(&other_props) {
    Ok(names) => names,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };

  let mut real_name_to_og_name = AHashMap::default();
  for (real_name, user_friendly_name) in real_names_player.iter().zip(&player_props) {
    real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
  }
  for (real_name, user_friendly_name) in real_other_props.iter().zip(&other_props) {
    real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
  }

  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    real_name_to_og_name: real_name_to_og_name,
    wanted_players: vec![],
    wanted_player_props: real_names_player.clone(),
    wanted_other_props: real_other_props.clone(),
    wanted_prop_states: AHashMap::default(),
    wanted_events: event_names,
    parse_ents: true,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: true,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let s = match serde_json::to_value(&output.game_events) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}

#[napi]
pub fn parse_ticks(
  path_or_buf: Either<String, Buffer>,
  wanted_props: Vec<String>,
  wanted_ticks: Option<Vec<i32>>,
  wanted_players: Option<Vec<String>>,
  struct_of_arrays: Option<bool>,
  order_by_steamid: Option<bool>,
  prop_states: Option<Vec<WantedPropState>>,
) -> napi::Result<Value> {
  let mut real_names = match rm_user_friendly_names(&wanted_props) {
    Ok(names) => names,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  let wanted_players_u64 = match wanted_players {
    Some(v) => v.iter().map(|x| x.parse::<u64>().unwrap_or(0)).collect(),
    None => vec![],
  };
  let wanted_prop_states: AHashMap<String, Variant> = prop_states
    .unwrap_or_default()
    .into_iter()
    .map(|prop| (prop.prop.clone(), prop.state.0.clone()))
    .collect();

  let real_wanted_prop_states = rm_map_user_friendly_names(&wanted_prop_states);
  let real_wanted_prop_states = match real_wanted_prop_states {
    Ok(real_wanted_prop_states) => real_wanted_prop_states,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };

  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();
  let mut real_name_to_og_name = AHashMap::default();

  for (real_name, user_friendly_name) in real_names.iter().zip(&wanted_props) {
    real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
  }
  for (real_name, user_friendly_name) in real_wanted_prop_states
    .keys()
    .zip(wanted_prop_states.keys())
  {
    real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
  }

  let wanted_ticks = match wanted_ticks {
    Some(t) => t,
    None => vec![],
  };
  let order_by_steamid = match order_by_steamid {
    Some(true) => true,
    _ => false,
  };

  let settings = ParserInputs {
    real_name_to_og_name: real_name_to_og_name,
    wanted_players: wanted_players_u64,
    wanted_player_props: real_names.clone(),
    wanted_other_props: vec![],
    wanted_events: vec![],
    wanted_prop_states: real_wanted_prop_states,
    parse_ents: true,
    wanted_ticks: wanted_ticks,
    parse_projectiles: false,
    only_header: false,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: order_by_steamid,
  };

  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  real_names.push("tick".to_owned());
  real_names.push("steamid".to_owned());
  real_names.push("name".to_owned());

  let mut prop_infos = output.prop_controller.prop_infos.clone();
  prop_infos.sort_by_key(|x| x.prop_name.clone());
  real_names.sort();

  let helper = OutputSerdeHelperStruct {
    prop_infos: prop_infos.clone(),
    inner: output.df.clone().into(),
  };

  let is_soa = match struct_of_arrays {
    Some(true) => true,
    _ => false,
  };

  if order_by_steamid {
    let mut helper_hm: HashMap<u64, _, RandomState> = HashMap::default();
    for (k, v) in output.df_per_player {
      let helper = OutputSerdeHelperStruct {
        prop_infos: prop_infos.clone(),
        inner: v.into(),
      };
      helper_hm.insert(k, helper);
    }
    let s = match serde_json::to_value(helper_hm) {
      Ok(s) => s,
      Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
    };
    return Ok(s);
  }
  if is_soa {
    let s = match serde_json::to_value(&helper) {
      Ok(s) => s,
      Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
    };
    return Ok(s);
  } else {
    let result = soa_to_aos(helper);
    let s = match serde_json::to_value(&result) {
      Ok(s) => s,
      Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
    };
    Ok(s)
  }
}

#[napi]
pub fn parse_player_info(path_or_buf: Either<String, Buffer>) -> napi::Result<Value> {
  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    wanted_players: vec![],
    real_name_to_og_name: AHashMap::default(),
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_prop_states: AHashMap::default(),
    wanted_events: vec![],
    parse_ents: false,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: true,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let s = match serde_json::to_value(&output.player_md) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}

#[napi]
pub fn parse_player_skins(path_or_buf: Either<String, Buffer>) -> napi::Result<Value> {
  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    wanted_players: vec![],
    real_name_to_og_name: AHashMap::default(),
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_prop_states: AHashMap::default(),
    wanted_events: vec![],
    parse_ents: true,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: true,
    list_props: false,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let s = match serde_json::to_value(&output.skins) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}
#[napi]
pub fn list_updated_fields(path_or_buf: Either<String, Buffer>) -> napi::Result<Value> {
  let bytes = resolve_byte_type(path_or_buf)?;
  let huf = create_huffman_lookup_table();

  let settings = ParserInputs {
    wanted_players: vec![],
    real_name_to_og_name: AHashMap::default(),
    wanted_player_props: vec![],
    wanted_other_props: vec![],
    wanted_prop_states: AHashMap::default(),
    wanted_events: vec!["none".to_string()],
    parse_ents: true,
    wanted_ticks: vec![],
    parse_projectiles: false,
    only_header: false,
    list_props: true,
    only_convars: false,
    huffman_lookup_table: &huf,
    order_by_steamid: false,
  };
  let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
  let output = parse_demo(bytes, &mut parser)?;
  let s = match serde_json::to_value(&output.uniq_prop_names) {
    Ok(s) => s,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
  };
  Ok(s)
}

fn resolve_byte_type(path_or_buf: Either<String, Buffer>) -> Result<BytesVariant, napi::Error> {
  match path_or_buf {
    Either::A(path) => {
      let file = match File::open(path.clone()) {
        Ok(f) => f,
        Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
      };
      let mmap = match unsafe { MmapOptions::new().map(&file) } {
        Ok(mmap) => mmap,
        Err(e) => return Err(Error::new(Status::InvalidArg, format!("{}", e).to_owned())),
      };
      Ok(BytesVariant::Mmap(mmap))
    }
    Either::B(buf) => Ok(BytesVariant::Vec(buf.into())),
  }
}
