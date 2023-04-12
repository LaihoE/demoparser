use crate::parsing::parser_settings::Parser;
use crate::parsing::variants::keydata_type_from_propdata;
use crate::parsing::variants::*;
use csgoproto::networkbasetypes::csvcmsg_game_event::Key_t;
use csgoproto::networkbasetypes::CSVCMsg_GameEvent;
use derive_more::TryInto;
use itertools::Itertools;
use polars::series::Series;

use super::entities::PlayerMetaData;

impl TryFrom<Option<PropData>> for KeyData {
    type Error = ();
    fn try_from(value: Option<PropData>) -> Result<Self, Self::Error> {
        match value {
            Some(propdata) => match propdata {
                PropData::Bool(b) => Ok(KeyData::Bool(b)),
                PropData::F32(f) => Ok(KeyData::Float(f)),
                PropData::I32(i) => Ok(KeyData::I32(i)),
                PropData::U64(u) => Ok(KeyData::Uint64(u)),
                PropData::String(s) => Ok(KeyData::Str(s)),
                PropData::U32(u) => Ok(KeyData::Uint64(u as u64)),
                _ => panic!("tried to add vector prop to game event."),
            },
            None => Err(()),
        }
    }
}

impl Parser {
    pub fn player_metadata_from_entid(&self, entity_id: i32) -> Option<&PlayerMetaData> {
        self.players.get(&entity_id)
    }

    pub fn find_extra_props(&self, entity_id: i32, prefix: &str) -> Vec<NameDataPair> {
        let mut extra_pairs = vec![];
        for prop_name in &self.wanted_props {
            let prop = self.find_val_for_entity(entity_id, prop_name);
            let keydata_type = keydata_type_from_propdata(&prop);
            let keydata = KeyData::try_from(prop);
            match keydata {
                Ok(kd) => {
                    extra_pairs.push(NameDataPair {
                        name: prefix.to_owned() + "_" + prop_name,
                        data: Some(kd),
                        data_type: keydata_type,
                    });
                }
                Err(_e) => {
                    extra_pairs.push(NameDataPair {
                        name: prefix.to_owned() + "_" + prop_name,
                        data: None,
                        data_type: keydata_type,
                    });
                }
            }
        }
        extra_pairs
    }

    pub fn parse_event(&mut self, event: CSVCMsg_GameEvent) {
        let ge_list = match &self.ge_list {
            Some(gel) => gel,
            None => panic!("Game event before descriptor list was parsed."),
        };
        let event_desc = &ge_list[&event.eventid()];

        self.game_events_counter
            .entry(event_desc.name.as_ref().unwrap().clone())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);

        if event_desc.name != self.wanted_event {
            return;
        }

        let mut kv_pairs: Vec<NameDataPair> = vec![];
        for i in 0..event.keys.len() {
            let ge = &event.keys[i];
            let desc = &event_desc.keys[i];
            let val = parse_key(ge);

            if !(desc.name().contains("userid")
                || desc.name().contains("attacker")
                || desc.name().contains("assister"))
            {
                kv_pairs.push(NameDataPair {
                    name: desc.name().to_owned(),
                    data: val,
                    data_type: ge.type_(),
                });
            }
            // User can ask for extra values if event has a entity id in it
            if desc.name().contains("pawn") {
                let entity_id = ge.val_long() & 0x7FF;
                let v: Vec<&str> = desc.name().split("_pawn").collect();
                let mut prefix = v[0];
                if prefix == "userid" {
                    prefix = "user";
                }
                match self.player_metadata_from_entid(entity_id) {
                    Some(player) => {
                        kv_pairs.push(NameDataPair {
                            name: prefix.to_owned() + "_name",
                            data: Some(KeyData::Str(player.name.clone())),
                            data_type: 1,
                        });
                        kv_pairs.push(NameDataPair {
                            name: prefix.to_owned() + "_steamid",
                            data: Some(KeyData::Uint64(player.steamid)),
                            data_type: 7,
                        });
                    }
                    None => {
                        kv_pairs.push(NameDataPair {
                            name: prefix.to_owned() + "_name",
                            data: None,
                            data_type: 1,
                        });
                        kv_pairs.push(NameDataPair {
                            name: prefix.to_owned() + "_steamid",
                            data: None,
                            data_type: 7,
                        });
                    }
                }
                let extra_props = self.find_extra_props(entity_id, prefix);
                kv_pairs.extend(extra_props);
            }
        }
        kv_pairs.push(NameDataPair {
            name: "tick".to_owned(),
            data: Some(KeyData::Long(self.tick)),
            data_type: 3,
        });
        self.game_events.push(GameEvent {
            fields: kv_pairs,
            name: event_desc.name().to_string(),
            tick: self.tick,
        });
    }
    #[allow(dead_code)]
    pub fn series_from_events(&self, events: &Vec<GameEvent>) -> Vec<Series> {
        let pairs: Vec<NameDataPair> = events.iter().map(|x| x.fields.clone()).flatten().collect();
        let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);
        let mut series = vec![];
        for (name, vals) in per_key_name {
            let s = series_from_pairs(&vals, name);
            series.push(s);
        }
        series.sort_by_key(|x| x.name().to_string());
        series
    }
}
fn parse_key(key: &Key_t) -> Option<KeyData> {
    match key.type_() {
        1 => Some(KeyData::Str(key.val_string().to_owned())),
        2 => Some(KeyData::Float(key.val_float())),
        3 => Some(KeyData::Long(key.val_long())),
        4 => Some(KeyData::Short(key.val_short().try_into().unwrap())),
        5 => Some(KeyData::Byte(key.val_byte().try_into().unwrap())),
        6 => Some(KeyData::Bool(key.val_bool())),
        7 => Some(KeyData::Uint64(key.val_uint64())),
        8 => Some(KeyData::I32(key.val_long().try_into().unwrap())),
        9 => Some(KeyData::I32(key.val_short().try_into().unwrap())),
        _ => {
            println!("Unknown key type for game event key: {:?}", key);
            return None;
        }
    }
}

#[derive(Debug, Clone, TryInto)]
#[try_into(owned, ref)]
pub enum KeyData {
    Str(String),
    Float(f32),
    Long(i32),
    Short(i16),
    Byte(u8),
    Bool(bool),
    Uint64(u64),
    I32(i32),
    Missing,
}

#[derive(Debug, Clone)]
pub struct NameDataPair {
    pub name: String,
    pub data: Option<KeyData>,
    pub data_type: i32,
}
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub name: String,
    pub fields: Vec<NameDataPair>,
    pub tick: i32,
}
