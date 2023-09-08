use crate::collect_data::PropType;
use crate::parser_settings::Parser;
use crate::parser_thread_settings::ParserThread;
use crate::prop_controller::PropInfo;
use crate::read_bits::DemoParserError;
use crate::stringtables::UserInfo;
use crate::variants::*;
use ahash::AHashMap;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use csgoproto::networkbasetypes::csvcmsg_game_event::Key_t;
use csgoproto::networkbasetypes::CSVCMsg_GameEvent;
use protobuf::Message;
use serde::ser::SerializeMap;
use serde::Serialize;

static INTERNALEVENTFIELDS: &'static [&str] = &[
    "userid",
    "attacker",
    "assister",
    "userid_pawn",
    "attacker_pawn",
    "assister_pawn",
];
const ENTITYIDNONE: i32 = 2047;

impl Parser {
    // Message that should come before first game event
    pub fn parse_game_event_list(&mut self, bytes: &[u8]) -> Result<AHashMap<i32, Descriptor_t>, DemoParserError> {
        let event_list: CSVCMsg_GameEventList = Message::parse_from_bytes(bytes).unwrap();
        let mut hm: AHashMap<i32, Descriptor_t, RandomState> = AHashMap::default();
        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        Ok(hm)
    }
}

impl ParserThread {
    pub fn parse_event(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if self.wanted_events.len() == 0 && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let event: CSVCMsg_GameEvent = Message::parse_from_bytes(&bytes).unwrap();
        // Check if this events id is found in our game event list
        let event_desc = match self.ge_list.get(&event.eventid()) {
            Some(desc) => desc,
            None => {
                return Ok(());
            }
        };
        self.game_events_counter.insert(event_desc.name.as_ref().unwrap().clone());

        // Return early if this is not a wanted event.
        if !self.wanted_events.contains(&event_desc.name().to_string()) && self.wanted_events.first() != Some(&"all".to_string())
        {
            return Ok(());
        }

        let mut event_fields: Vec<EventField> = vec![];
        // Parsing game events is this easy, the complexity comes from adding "extra" fields into events.
        for i in 0..event.keys.len() {
            let ge = &event.keys[i];
            let desc = &event_desc.keys[i];
            let val = parse_key(ge);
            event_fields.push(EventField {
                name: desc.name().to_owned(),
                data: val,
            });
        }
        // Add extra fields
        event_fields.extend(self.find_extra(&event_fields)?);
        // Remove fields that user does nothing with like userid and user_pawn
        event_fields.retain(|ref x| !INTERNALEVENTFIELDS.contains(&x.name.as_str()));

        self.game_events.push(GameEvent {
            fields: event_fields,
            name: event_desc.name().to_string(),
            tick: self.tick,
        });
        Ok(())
    }
    fn find_user_by_userid(&self, userid: i32) -> Option<&UserInfo> {
        for player in self.stringtable_players.values() {
            if player.userid == userid {
                return Some(player);
            }
        }
        return None;
    }
    fn entity_id_from_userid(&self, userid: i32) -> Option<i32> {
        if let Some(userinfo) = self.find_user_by_userid(userid) {
            for player in self.players.values() {
                if player.steamid == Some(userinfo.steamid) {
                    return Some(player.player_entity_id.unwrap());
                }
            }
        }
        return None;
    }
    fn find_extra(&self, fields: &Vec<EventField>) -> Result<Vec<EventField>, DemoParserError> {
        let mut extra_fields = vec![];
        // Always add tick to event
        extra_fields.push(EventField {
            name: "tick".to_owned(),
            data: Some(Variant::I32(self.tick)),
        });
        for field in fields {
            // Fields that refer to players
            let prefix = match field.name.as_str() {
                "attacker" => "attacker",
                "userid" => "user",
                "assister" => "assister",
                _ => continue,
            };
            if let Some(Variant::I32(u)) = field.data {
                let entity_id = match self.entity_id_from_userid(u) {
                    Some(eid) => eid,
                    None => {
                        // player could not be found --> add None to output
                        extra_fields.extend(self.generate_empty_fields(prefix));
                        continue;
                    }
                };
                extra_fields.push(self.create_player_name_field(entity_id, prefix));
                extra_fields.push(self.create_player_steamid_field(entity_id, prefix));
                extra_fields.extend(self.find_extra_props_events(entity_id, prefix));
            }
        }
        // Values from Teams and Rules entity. Not bound to any player so can be added to any event.
        extra_fields.extend(self.find_non_player_props());
        Ok(extra_fields)
    }
    fn generate_empty_fields(&self, prefix: &str) -> Vec<EventField> {
        let mut extra_fields = vec![];
        // when pointer fails for some reason we need to add None to output
        for prop_info in &self.prop_controller.prop_infos {
            // These are meant for entities and should not be collected here
            if prop_info.prop_name == "tick" || prop_info.prop_name == "name" || prop_info.prop_name == "steamid" {
                continue;
            }
            if !prop_info.is_player_prop {
                continue;
            }
            extra_fields.push(EventField {
                name: prefix.to_owned() + "_" + &prop_info.prop_friendly_name,
                data: None,
            });
        }
        extra_fields.push(EventField {
            name: prefix.to_owned() + "_steamid",
            data: None,
        });
        extra_fields.push(EventField {
            name: prefix.to_owned() + "_name",
            data: None,
        });
        extra_fields
    }

    fn find_non_player_props(&self) -> Vec<EventField> {
        let mut extra_fields = vec![];
        for prop_info in &self.prop_controller.prop_infos {
            let fields = match prop_info.prop_type {
                PropType::Team => self.find_other_team_props(&prop_info),
                PropType::Rules => self.find_other_rules_props(&prop_info),
                _ => vec![],
            };
            extra_fields.extend(fields);
        }
        extra_fields
    }

    fn find_other_rules_props(&self, prop_info: &PropInfo) -> Vec<EventField> {
        let mut extra_fields = vec![];
        let prop = match self.rules_entity_id {
            Some(entid) => match self.get_prop_from_ent(&prop_info.id, &entid) {
                Ok(p) => Some(p),
                Err(_e) => None,
            },
            None => None,
        };
        extra_fields.push(EventField {
            name: prop_info.prop_friendly_name.to_owned(),
            data: prop,
        });
        extra_fields
    }
    fn find_other_team_props(&self, prop_info: &PropInfo) -> Vec<EventField> {
        let mut extra_fields = vec![];
        let t = self.teams.team2_entid;
        let ct = self.teams.team3_entid;
        let t_prop = match t {
            Some(entid) => match self.get_prop_from_ent(&prop_info.id, &entid) {
                Ok(p) => Some(p),
                Err(_) => None,
            },
            None => None,
        };
        let ct_prop = match ct {
            Some(entid) => match self.get_prop_from_ent(&prop_info.id, &entid) {
                Ok(p) => Some(p),
                Err(_) => None,
            },
            None => None,
        };
        extra_fields.push(EventField {
            name: "t_".to_owned() + &prop_info.prop_friendly_name,
            data: t_prop,
        });
        extra_fields.push(EventField {
            name: "ct_".to_owned() + &prop_info.prop_friendly_name,
            data: ct_prop,
        });
        extra_fields
    }

    pub fn find_extra_props_events(&self, entity_id: i32, prefix: &str) -> Vec<EventField> {
        let mut extra_pairs = vec![];
        // println!("{:#?}", self.prop_controller.prop_infos);
        for prop_info in &self.prop_controller.prop_infos {
            // These props are collected in find_non_player_props()
            if !prop_info.is_player_prop {
                continue;
            }
            // These are meant for entities and should not be collected here
            if prop_info.prop_name == "tick" || prop_info.prop_name == "name" || prop_info.prop_name == "steamid" {
                continue;
            }
            if entity_id == ENTITYIDNONE {
                extra_pairs.push(EventField {
                    name: prefix.to_owned() + "_" + &prop_info.prop_friendly_name,
                    data: None,
                });
                continue;
            }
            let prop = match self.players.get(&entity_id) {
                Some(player_md) => match self.find_prop(&prop_info, &entity_id, player_md) {
                    Ok(p) => Some(p),
                    Err(_e) => None,
                },
                None => None,
            };
            match prop {
                Some(kd) => {
                    extra_pairs.push(EventField {
                        name: prefix.to_owned() + "_" + &prop_info.prop_friendly_name,
                        data: Some(kd),
                    });
                }
                None => {
                    extra_pairs.push(EventField {
                        name: prefix.to_owned() + "_" + &prop_info.prop_friendly_name,
                        data: None,
                    });
                }
            }
        }
        extra_pairs
    }
    fn create_player_name_field(&self, entity_id: i32, prefix: &str) -> EventField {
        if entity_id == ENTITYIDNONE {
            return EventField {
                name: prefix.to_owned() + "_name",
                data: None,
            };
        }
        let data = match self.players.get(&entity_id) {
            Some(player_md) => match &player_md.name {
                Some(name) => Some(Variant::String(name.clone())),
                None => None,
            },
            None => None,
        };
        EventField {
            name: prefix.to_owned() + "_name",
            data: data,
        }
    }
    fn create_player_steamid_field(&self, entity_id: i32, prefix: &str) -> EventField {
        if entity_id == ENTITYIDNONE {
            return EventField {
                name: prefix.to_owned() + "_steamid",
                data: None,
            };
        }
        let data = match self.players.get(&entity_id) {
            Some(player_md) => match player_md.steamid {
                Some(steamid) => Some(Variant::String(steamid.to_string())),
                None => None,
            },
            None => None,
        };
        EventField {
            name: prefix.to_owned() + "_steamid",
            data: data,
        }
    }
}
// what is this shit
fn parse_key(key: &Key_t) -> Option<Variant> {
    match key.type_() {
        1 => Some(Variant::String(key.val_string().to_owned())),
        2 => Some(Variant::F32(key.val_float())),
        // These seem to return an i32
        3 => Some(Variant::I32(key.val_long())),
        4 => Some(Variant::I32(key.val_short().try_into().unwrap())),
        5 => Some(Variant::I32(key.val_byte().try_into().unwrap())),
        6 => Some(Variant::Bool(key.val_bool())),
        7 => Some(Variant::U64(key.val_uint64())),
        8 => Some(Variant::I32(key.val_long().try_into().unwrap())),
        9 => Some(Variant::I32(key.val_short().try_into().unwrap())),
        _ => {
            return None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventField {
    pub name: String,
    pub data: Option<Variant>,
}
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub name: String,
    pub fields: Vec<EventField>,
    pub tick: i32,
}

impl Serialize for GameEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(&"tick", &self.tick).unwrap();
        map.serialize_entry(&"event_name", &self.name).unwrap();
        for field in &self.fields {
            map.serialize_entry(&field.name, &field.data).unwrap();
        }
        map.end()
    }
}
