use crate::first_pass::prop_controller::PropInfo;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::stringtables::UserInfo;
use crate::second_pass::collect_data::PropType;
use crate::second_pass::entities::PlayerMetaData;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::variants::*;
use csgoproto::cstrike15_usermessages::CCSUsrMsg_ServerRankUpdate;
use csgoproto::networkbasetypes::csvcmsg_game_event::Key_t;
use csgoproto::networkbasetypes::CNETMsg_SetConVar;
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

static ENTITIES_FIRST_EVENTS: &'static [&str] = &["inferno_startburn", "decoy_started", "inferno_expire"];
static REMOVEDEVENTS: &'static [&str] = &["server_cvar"];

const ENTITYIDNONE: i32 = 2047;
// https://developer.valvesoftware.com/wiki/SteamID
const STEAMID64INDIVIDUALIDENTIFIER: u64 = 0x0110000100000000;

impl<'a> SecondPassParser<'a> {
    pub fn parse_event(&mut self, bytes: &[u8]) -> Result<Option<GameEvent>, DemoParserError> {
        if self.wanted_events.len() == 0 && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(None);
        }
        let event: CSVCMsg_GameEvent = match Message::parse_from_bytes(&bytes) {
            Ok(event) => event,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        // Check if this events id is found in our game event list
        let event_desc = match self.ge_list.get(&event.eventid()) {
            Some(desc) => desc,
            None => {
                return Ok(None);
            }
        };
        if let Some(event_name) = &event_desc.name {
            self.game_events_counter.insert(event_name.to_owned());
        }
        // Return early if this is not a wanted event.
        if !self.wanted_events.contains(&event_desc.name().to_string()) && self.wanted_events.first() != Some(&"all".to_string())
        {
            return Ok(None);
        }
        if REMOVEDEVENTS.contains(&event_desc.name()) {
            return Ok(None);
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
        if ENTITIES_FIRST_EVENTS.contains(&event_desc.name()) {
            let event = GameEvent {
                fields: event_fields,
                name: event_desc.name().to_string(),
                tick: self.tick,
            };
            return Ok(Some(event));
        } else {
            // Add extra fields
            event_fields.extend(self.find_extra(&event_fields)?);
            // Remove fields that user does nothing with like userid and user_pawn
            event_fields.retain(|ref x| !INTERNALEVENTFIELDS.contains(&x.name.as_str()));
            let event = GameEvent {
                fields: event_fields,
                name: event_desc.name().to_string(),
                tick: self.tick,
            };
            self.game_events.push(event);
        }
        Ok(None)
    }
    pub fn resolve_wrong_order_event(&mut self, events: &mut Vec<GameEvent>) -> Result<(), DemoParserError> {
        for event in events {
            event.fields.extend(self.find_extra(&event.fields)?);
            // Remove fields that user does nothing with like userid and user_pawn
            event.fields.retain(|ref x| !INTERNALEVENTFIELDS.contains(&x.name.as_str()));
            let event = GameEvent {
                fields: event.fields.clone(),
                name: event.name.to_string(),
                tick: self.tick,
            };
            self.game_events.push(event);
        }
        Ok(())
    }

    pub fn find_user_by_userid(&self, userid: i32) -> Option<&UserInfo> {
        for player in self.stringtable_players.values() {
            if player.userid & 0xFF == userid {
                return Some(player);
            }
        }
        // Fallback for old demos?
        for player in self.stringtable_players.values() {
            if player.userid == userid {
                return Some(player);
            }
        }
        return None;
    }
    pub fn find_user_by_controller_id(&self, userid: i32) -> Option<&PlayerMetaData> {
        for (_, player) in &self.players {
            if player.controller_entid == Some(userid) {
                return Some(player);
            }
        }
        return None;
    }
    pub fn entity_id_from_userid(&self, userid: i32) -> Option<i32> {
        if let Some(userinfo) = self.find_user_by_userid(userid) {
            for player in self.players.values() {
                if player.steamid == Some(userinfo.steamid) {
                    if let Some(entity_id) = player.player_entity_id {
                        return Some(entity_id);
                    }
                }
            }
        }
        return None;
    }
    pub fn find_extra(&self, fields: &Vec<EventField>) -> Result<Vec<EventField>, DemoParserError> {
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
                // edge case in some events
                "entityid" => {
                    let field_names: Vec<&String> = fields.iter().map(|x| &x.name).collect();
                    if field_names.contains(&&"userid".to_string()) {
                        continue;
                    } else {
                        "user"
                    }
                }
                _ => continue,
            };
            if let Some(Variant::I32(u)) = field.data {
                let entity_id = match field.name.as_str() {
                    "entityid" => self.grenade_owner_entid_from_grenade(&field.data),
                    _ => self.entity_id_from_userid(u),
                };
                let entity_id = match entity_id {
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
    pub fn grenade_owner_entid_from_grenade(&self, id_field: &Option<Variant>) -> Option<i32> {
        let prop_id = match self.prop_controller.special_ids.grenade_owner_id {
            Some(id) => id,
            None => return None,
        };
        if let Some(Variant::I32(id)) = id_field {
            if let Ok(Variant::U32(entity_id)) = self.get_prop_from_ent(&prop_id, &id) {
                return Some((entity_id & 0x7ff) as i32);
            }
        }
        None
    }
    pub fn generate_empty_fields(&self, prefix: &str) -> Vec<EventField> {
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

    pub fn find_non_player_props(&self) -> Vec<EventField> {
        let mut extra_fields = vec![];
        for prop_info in &self.prop_controller.prop_infos {
            let fields = match prop_info.prop_type {
                PropType::Team => self.find_other_team_props(&prop_info),
                PropType::Rules => self.find_other_rules_props(&prop_info),
                PropType::GameTime => vec![EventField {
                    data: Some(Variant::F32(self.net_tick as f32 / 64.0)),
                    name: "game_time".to_string(),
                }],
                _ => vec![],
            };
            extra_fields.extend(fields);
        }
        extra_fields
    }

    pub fn find_other_rules_props(&self, prop_info: &PropInfo) -> Vec<EventField> {
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
    pub fn find_other_team_props(&self, prop_info: &PropInfo) -> Vec<EventField> {
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
    pub fn create_player_name_field(&self, entity_id: i32, prefix: &str) -> EventField {
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
    pub fn create_player_steamid_field(&self, entity_id: i32, prefix: &str) -> EventField {
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
    pub fn player_from_steamid32(&self, steamid32: i32) -> Option<i32> {
        for (_entid, player) in &self.players {
            if let Some(steamid) = player.steamid {
                if steamid - STEAMID64INDIVIDUALIDENTIFIER == steamid32 as u64 {
                    if let Some(entity_id) = player.player_entity_id {
                        return Some(entity_id);
                    }
                }
            }
        }
        None
    }

    pub fn create_custom_event_parse_convars(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("server_cvar".to_string());
        if !self.wanted_events.contains(&"server_cvar".to_string()) {
            return Ok(());
        }
        let convar: CNETMsg_SetConVar = match Message::parse_from_bytes(&bytes) {
            Ok(m) => m,
            Err(_e) => return Err(DemoParserError::MalformedMessage),
        };
        for cv in &convar.convars {
            let mut fields = vec![];
            for var in &cv.cvars {
                fields.push(EventField {
                    data: Some(Variant::String(var.value().to_owned())),
                    name: "value".to_string(),
                });
                fields.push(EventField {
                    data: Some(Variant::String(var.name().to_string())),
                    name: "name".to_string(),
                });
                fields.push(EventField {
                    data: Some(Variant::I32(self.tick)),
                    name: "tick".to_string(),
                });
            }
            let ge = GameEvent {
                name: "server_cvar".to_string(),
                fields: fields,
                tick: self.tick,
            };
            self.game_events.push(ge);
            self.game_events_counter.insert("server_cvar".to_string());
        }
        Ok(())
    }
    pub fn create_custom_event_rank_update(&mut self, msg_bytes: &[u8]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("rank_update".to_string());
        if !self.wanted_events.contains(&"rank_update".to_string()) {
            return Ok(());
        }
        let update_msg: CCSUsrMsg_ServerRankUpdate = match Message::parse_from_bytes(&msg_bytes) {
            Ok(m) => m,
            Err(_e) => return Err(DemoParserError::MalformedMessage),
        };

        for update in update_msg.rank_update {
            let mut fields = vec![];

            let entity_id = match self.player_from_steamid32(update.account_id.unwrap_or(-1)) {
                Some(eid) => eid,
                None => continue,
            };

            fields.push(self.create_player_name_field(entity_id, "user"));
            fields.push(self.create_player_steamid_field(entity_id, "user"));
            fields.extend(self.find_extra_props_events(entity_id, "user"));

            fields.push(EventField {
                data: Some(Variant::I32(update.num_wins())),
                name: "num_wins".to_string(),
            });
            fields.push(EventField {
                data: Some(Variant::I32(update.rank_old())),
                name: "rank_old".to_string(),
            });
            fields.push(EventField {
                data: Some(Variant::I32(update.rank_new())),
                name: "rank_new".to_string(),
            });
            fields.push(EventField {
                data: Some(Variant::F32(update.rank_change())),
                name: "rank_change".to_string(),
            });
            fields.push(EventField {
                data: Some(Variant::I32(update.rank_type_id())),
                name: "rank_type_id".to_string(),
            });
            let ge = GameEvent {
                name: "rank_update".to_string(),
                fields: fields,
                tick: self.tick,
            };
            self.game_events.push(ge);
            self.game_events_counter.insert("rank_update".to_string());
        }

        Ok(())
    }
}
// what is this shit
fn parse_key(key: &Key_t) -> Option<Variant> {
    match key.type_() {
        1 => Some(Variant::String(key.val_string().to_owned())),
        2 => Some(Variant::F32(key.val_float())),
        // These seem to return an i32
        3 => Some(Variant::I32(key.val_long())),
        4 => Some(Variant::I32(key.val_short().try_into().unwrap_or(-1))),
        5 => Some(Variant::I32(key.val_byte().try_into().unwrap_or(-1))),
        6 => Some(Variant::Bool(key.val_bool())),
        7 => Some(Variant::U64(key.val_uint64())),
        8 => Some(Variant::I32(key.val_long().try_into().unwrap_or(-1))),
        9 => Some(Variant::I32(key.val_short().try_into().unwrap_or(-1))),
        _ => {
            return None;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventField {
    pub name: String,
    pub data: Option<Variant>,
}
#[derive(Debug, Clone, PartialEq)]
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
        map.serialize_entry(&"tick", &self.tick)?;
        map.serialize_entry(&"event_name", &self.name)?;
        for field in &self.fields {
            map.serialize_entry(&field.name, &field.data)?;
        }
        map.end()
    }
}
