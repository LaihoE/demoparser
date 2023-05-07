use crate::parser_settings::Parser;
use crate::read_bits::DemoParserError;
use crate::variants::*;
use ahash::AHashMap;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use csgoproto::networkbasetypes::csvcmsg_game_event::Key_t;
use csgoproto::networkbasetypes::CSVCMsg_GameEvent;
use protobuf::Message;

static INTERNALEVENTFIELDS: &'static [&str] = &[
    "userid",
    "attacker",
    "assister",
    "userid_pawn",
    "attacker_pawn",
    "assister_pawn",
];

impl<'a> Parser<'a> {
    // Message that should come before first game event
    pub fn parse_game_event_list(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let event_list: CSVCMsg_GameEventList = Message::parse_from_bytes(bytes).unwrap();
        let mut hm: AHashMap<i32, Descriptor_t, RandomState> = AHashMap::default();
        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.ge_list = Some(hm);
        Ok(())
    }

    pub fn parse_event(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if self.wanted_event.is_none() {
            return Ok(());
        }
        let event: CSVCMsg_GameEvent = Message::parse_from_bytes(&bytes).unwrap();
        let ge_list = match &self.ge_list {
            Some(gel) => gel,
            None => return Err(DemoParserError::GameEventListNotSet),
        };
        // Check if this events id is found in our game event list
        let event_desc = match ge_list.get(&event.eventid()) {
            Some(desc) => desc,
            None => {
                return Err(DemoParserError::GameEventUnknownId(
                    event.eventid().to_string(),
                ))
            }
        };
        // Used to count how many of each event in this demo. Cheap so do it always
        self.game_events_counter
            .entry(event_desc.name.as_ref().unwrap().clone())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        // Return if this is not our wanted event. (user can only request one event per demo)
        // This could change in the future.
        if event_desc.name != self.wanted_event {
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

    fn find_extra(&self, fields: &Vec<EventField>) -> Result<Vec<EventField>, DemoParserError> {
        let mut extra_fields = vec![];
        // Always add tick to event
        extra_fields.push(EventField {
            name: "tick".to_owned(),
            data: Some(Variant::I32(self.tick)),
        });

        for field in fields {
            if field.name.contains("pawn") {
                match field.data {
                    Some(Variant::I32(entid_handle)) => {
                        let entity_id = entid_handle & 0x7FF;
                        // strip out _pawn from name:  userid_pawn => userid
                        // this assumes that "pawn" is not used for other key names, only for handles to players
                        let prefix = match field.name.split("_pawn").next() {
                            Some(prefix) => prefix,
                            None => {
                                return Err(DemoParserError::UnknownPawnPrefix(field.name.clone()))
                            }
                        };
                        extra_fields.push(self.create_player_name_field(entity_id, prefix));
                        extra_fields.push(self.create_player_steamid_field(entity_id, prefix));
                        extra_fields.extend(self.find_extra_props_events(entity_id, prefix)?);
                    }
                    _ => {
                        return Err(DemoParserError::UnknownEntityHandle((
                            field.name.clone(),
                            field.data.clone(),
                        )));
                    }
                }
            }
        }
        Ok(extra_fields)
    }

    pub fn find_extra_props_events(
        &self,
        entity_id: i32,
        prefix: &str,
    ) -> Result<Vec<EventField>, DemoParserError> {
        let mut extra_pairs = vec![];

        for (prop_name, og_name) in self.wanted_props.iter().zip(&self.wanted_prop_og_names) {
            // These are meant for entities not used here
            if prop_name == "tick" || prop_name == "name" || prop_name == "steamid" {
                continue;
            }
            let prop = match self.players.get(&entity_id) {
                Some(player_md) => self.find_prop(prop_name, &entity_id, player_md),
                None => None,
            };
            match prop {
                Some(kd) => {
                    extra_pairs.push(EventField {
                        name: prefix.to_owned() + "_" + og_name,
                        data: Some(kd),
                    });
                }
                None => {
                    extra_pairs.push(EventField {
                        name: prefix.to_owned() + "_" + og_name,
                        data: None,
                    });
                }
            }
        }
        Ok(extra_pairs)
    }
    fn create_player_name_field(&self, entity_id: i32, prefix: &str) -> EventField {
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
        let data = match self.players.get(&entity_id) {
            Some(player_md) => match player_md.steamid {
                Some(steamid) => Some(Variant::U64(steamid)),
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
fn parse_key(key: &Key_t) -> Option<Variant> {
    match key.type_() {
        1 => Some(Variant::String(key.val_string().to_owned())),
        2 => Some(Variant::F32(key.val_float())),
        // These seem to return a i32
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
