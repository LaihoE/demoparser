use crate::first_pass::prop_controller::PropController;
use crate::first_pass::prop_controller::PropInfo;
use crate::first_pass::prop_controller::ITEM_PURCHASE_COST;
use crate::first_pass::prop_controller::ITEM_PURCHASE_COUNT;
use crate::first_pass::prop_controller::ITEM_PURCHASE_DEF_IDX;
use crate::first_pass::prop_controller::ITEM_PURCHASE_NEW_DEF_IDX;
use crate::first_pass::prop_controller::WEAPON_FLOAT;
use crate::first_pass::prop_controller::WEAPON_PAINT_SEED;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Field;
use crate::first_pass::sendtables::FieldInfo;
use crate::first_pass::stringtables::UserInfo;
use crate::maps::HIT_GROUP;
use crate::maps::ROUND_WIN_REASON;
use crate::maps::ROUND_WIN_REASON_TO_WINNER;
use crate::second_pass::collect_data::PropType;
use crate::second_pass::entities::Entity;
use crate::second_pass::entities::PlayerMetaData;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::variants::*;
use csgoproto::csvc_msg_game_event::KeyT;
use csgoproto::maps::WEAPINDICIES;
use csgoproto::CUserMessageSayText;
use csgoproto::CUserMessageSayText2;
use csgoproto::CcsUsrMsgServerRankUpdate;
use csgoproto::CnetMsgSetConVar;
use csgoproto::CsvcMsgGameEvent;
use itertools::Itertools;
use prost::Message;
use serde::ser::SerializeMap;
use serde::Serialize;

static INTERNALEVENTFIELDS: &'static [&str] = &[
    "userid",
    "attacker",
    "assister",
    "userid_pawn",
    "attacker_pawn",
    "assister_pawn",
    "victim",
    "victim_pawn",
];
#[derive(Debug, Clone)]
pub struct RoundEnd {
    pub old_value: Option<Variant>,
    pub new_value: Option<Variant>,
}
#[derive(Debug, Clone)]
pub struct RoundWinReason {
    pub reason: i32,
}
#[derive(Debug, Clone)]
pub enum GameEventInfo {
    RoundEnd(RoundEnd),
    RoundWinReason(RoundWinReason),
    FreezePeriodStart(bool),
    MatchEnd(),
    WeaponCreateHitem((Variant, i32)),
    WeaponCreateNCost((Variant, i32)),
    WeaponCreateDefIdx((Variant, i32, u32)),
    WeaponPurchaseCount((Variant, i32, u32)),
}

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

        let event = match CsvcMsgGameEvent::decode(bytes) {
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
        if !self.wanted_events.contains(&event_desc.name().to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
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
            let mut event = GameEvent {
                fields: event_fields,
                name: event_desc.name().to_string(),
                tick: self.tick,
            };
            self.cleanups(&mut event);
            self.game_events.push(event);
        }
        Ok(None)
    }
    fn cleanups(&self, event: &mut GameEvent) {
        // Contains some fixed like renaming weapons to be consitent.
        for field in &mut event.fields {
            if field.name == "hitgroup" {
                if let Some(Variant::I32(i)) = field.data {
                    if let Some(str) = HIT_GROUP.get(&i) {
                        field.data = Some(Variant::String(str.to_string()))
                    } else {
                        field.data = Some(Variant::String(i.to_string()))
                    }
                }
            }
        }
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
                "victim" => "victim",
                // edge case in some events
                "entityid" => {
                    let field_names: Vec<&String> = fields.iter().map(|x| &x.name).collect();
                    if field_names.contains(&&"userid".to_string()) {
                        continue;
                    } else {
                        "user"
                    }
                }
                // Another edge case
                // Only add iff "userid" is missing in the event...
                "userid_pawn" => {
                    let field_names: Vec<&String> = fields.iter().map(|x| &x.name).collect();
                    if !field_names.contains(&&"userid".to_string()) && !field_names.contains(&&"entityid".to_string()) {
                        "user"
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
            if let Some(Variant::I32(u)) = field.data {
                let entity_id = match field.name.as_str() {
                    "entityid" => self.grenade_owner_entid_from_grenade(&field.data),
                    "userid_pawn" => self.entity_id_from_user_pawn(u),
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
    pub fn entity_id_from_user_pawn(&self, pawn_handle: i32) -> Option<i32> {
        Some(pawn_handle & 0x7FF)
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
            data,
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
        if !self.wanted_events.contains(&"server_cvar".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let convar = match CnetMsgSetConVar::decode(bytes) {
            Ok(m) => m,
            Err(_e) => return Err(DemoParserError::MalformedMessage),
        };
        if let Some(convars) = &convar.convars {
            let mut fields = vec![];
            for var in &convars.cvars {
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
            fields.extend(self.find_non_player_props());
            let ge = GameEvent {
                name: "server_cvar".to_string(),
                fields,
                tick: self.tick,
            };
            self.game_events.push(ge);
            self.game_events_counter.insert("server_cvar".to_string());
        }
        Ok(())
    }
    fn contains_round_end_event(events: &[GameEventInfo]) -> bool {
        events.iter().any(|s| match s {
            &GameEventInfo::RoundEnd(_) => true,
            _ => false,
        })
    }
    fn contains_freeze_period_start(events: &[GameEventInfo]) -> bool {
        events.iter().any(|s| match s {
            &GameEventInfo::FreezePeriodStart(_) => true,
            _ => false,
        })
    }
    fn contains_match_end(events: &[GameEventInfo]) -> bool {
        events.iter().any(|s| match s {
            &GameEventInfo::MatchEnd() => true,
            _ => false,
        })
    }
    fn contains_weapon_create(events: &[GameEventInfo]) -> bool {
        events.iter().any(|s| match s {
            &GameEventInfo::WeaponCreateDefIdx(_) => true,
            _ => false,
        })
    }
    pub fn emit_events(&mut self, events: Vec<GameEventInfo>) -> Result<(), DemoParserError> {
        if SecondPassParser::contains_round_end_event(&events) {
            self.create_custom_event_round_end(&events)?;
        }
        if SecondPassParser::contains_freeze_period_start(&events) {
            self.create_custom_event_round_officially_ended(&events)?;
            self.create_custom_event_round_start(&events)?;
        }
        if SecondPassParser::contains_match_end(&events) {
            self.create_custom_event_match_end(&events)?;
        }
        if SecondPassParser::contains_weapon_create(&events) {
            self.create_custom_event_weapon_purchase(&events);
        }
        self.create_custom_event_weapon_sold(&events);
        Ok(())
    }
    fn create_custom_event_weapon_sold(&mut self, events: &[GameEventInfo]) {
        // This event is always emitted and is always removed in the end.
        events.iter().for_each(|x| match x {
            GameEventInfo::WeaponPurchaseCount((Variant::U32(0), entid, prop_id)) => {
                if let Ok(player) = self.find_player_metadata(*entid) {
                    let mut fields = vec![];
                    fields.push(EventField {
                        data: self.create_name(player).ok(),
                        name: "name".to_string(),
                    });
                    fields.push(EventField {
                        data: Some(Variant::U64(player.steamid.unwrap_or(0))),
                        name: "steamid".to_string(),
                    });
                    fields.push(EventField {
                        data: Some(Variant::I32(self.tick)),
                        name: "tick".to_string(),
                    });
                    let inventory_slot = prop_id - ITEM_PURCHASE_COUNT;
                    let def_idx = self.get_prop_from_ent(&(&ITEM_PURCHASE_NEW_DEF_IDX + inventory_slot), entid).ok();

                    let name = match def_idx {
                        Some(Variant::U32(id)) => {
                            if let Some(name) = WEAPINDICIES.get(&id) {
                                Some(Variant::String(name.to_string()))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    fields.push(EventField {
                        data: name,
                        name: "weapon_name".to_string(),
                    });

                    fields.push(EventField {
                        data: self.get_prop_from_ent(&(&ITEM_PURCHASE_COST + inventory_slot), entid).ok(),
                        name: "cost".to_string(),
                    });
                    fields.push(EventField {
                        data: Some(Variant::U32(inventory_slot)),
                        name: "inventory_slot".to_string(),
                    });
                    fields.extend(self.find_extra_props_events(*entid, "user"));
                    fields.extend(self.find_non_player_props());
                    let ge = GameEvent {
                        name: "item_sold".to_string(),
                        fields,
                        tick: self.tick,
                    };
                    self.game_events.push(ge);
                    self.game_events_counter.insert("item_sold".to_string());
                }
            }
            _ => {}
        });
    }
    fn combine_purchase_events(events: &[GameEventInfo]) -> Vec<PurchaseEvent> {
        // Vec<Gameventinfo> --> Vec<(def_idx, weapon_cost)>
        // Filter purchase events
        let filtered_events = events
            .iter()
            .filter(|x| match x {
                GameEventInfo::WeaponCreateDefIdx(_) => true,
                GameEventInfo::WeaponCreateNCost(_) => true,
                GameEventInfo::WeaponCreateHitem(_) => true,
                _ => false,
            })
            .collect_vec();
        let mut purchases = vec![];
        let mut ptr = 0;
        while ptr < filtered_events.len() {
            let entry_1 = filtered_events.get(ptr);
            let entry_2 = filtered_events.get(ptr + 1);
            let entry_3 = filtered_events.get(ptr + 2);

            match (entry_1, entry_2, entry_3) {
                (
                    Some(GameEventInfo::WeaponCreateDefIdx((Variant::U32(def), entid, prop_id))),
                    Some(GameEventInfo::WeaponCreateNCost((Variant::I32(cost), _))),
                    Some(GameEventInfo::WeaponCreateHitem((Variant::U64(handle), _))),
                ) => {
                    match WEAPINDICIES.get(&(*def as u32)) {
                        Some(name) => {
                            purchases.push(PurchaseEvent {
                                cost: *cost,
                                name: Some(name.to_string()),
                                entid: *entid,
                                weapon_entid: (handle & 0x7ff) as i32,
                                inventory_slot: (prop_id - ITEM_PURCHASE_DEF_IDX),
                            });
                        }
                        None => {
                            purchases.push(PurchaseEvent {
                                cost: *cost,
                                name: None,
                                entid: *entid,
                                weapon_entid: (handle & 0x7ff) as i32,
                                inventory_slot: (prop_id - ITEM_PURCHASE_DEF_IDX),
                            });
                        }
                    }
                    ptr += 3;
                }
                (
                    Some(GameEventInfo::WeaponCreateDefIdx((Variant::U32(def), entid, prop_id))),
                    Some(GameEventInfo::WeaponCreateNCost((Variant::I32(cost), _))),
                    _,
                ) => {
                    match WEAPINDICIES.get(&(*def as u32)) {
                        Some(name) => {
                            purchases.push(PurchaseEvent {
                                cost: *cost,
                                name: Some(name.to_string()),
                                entid: *entid,
                                weapon_entid: ENTITYIDNONE,
                                inventory_slot: (prop_id - ITEM_PURCHASE_DEF_IDX),
                            });
                        }
                        None => {
                            purchases.push(PurchaseEvent {
                                cost: *cost,
                                name: None,
                                entid: *entid,
                                weapon_entid: ENTITYIDNONE,
                                inventory_slot: (prop_id - ITEM_PURCHASE_DEF_IDX),
                            });
                        }
                    }
                    ptr += 2;
                }
                _ => ptr += 1,
            }
        }
        purchases
    }
    fn create_custom_event_weapon_purchase(&mut self, events: &[GameEventInfo]) {
        self.game_events_counter.insert("item_purchase".to_string());
        if !self.wanted_events.contains(&"item_purchase".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return;
        }
        let purchases = SecondPassParser::combine_purchase_events(events);
        for purchase in purchases {
            let mut fields = vec![];
            if let Some(buy_zone_id) = self.prop_controller.special_ids.in_buy_zone {
                if let Ok(Variant::Bool(true)) = self.get_prop_from_ent(&buy_zone_id, &purchase.entid) {
                    if let Ok(player) = self.find_player_metadata(purchase.entid) {
                        match purchase.name {
                            Some(name) => {
                                fields.push(EventField {
                                    data: Some(Variant::String(name)),
                                    name: "item_name".to_string(),
                                });
                            }
                            None => {
                                fields.push(EventField {
                                    data: None,
                                    name: "item_name".to_string(),
                                });
                            }
                        }
                        fields.push(EventField {
                            data: self.create_name(player).ok(),
                            name: "name".to_string(),
                        });
                        fields.push(EventField {
                            data: Some(Variant::U64(player.steamid.unwrap_or(0))),
                            name: "steamid".to_string(),
                        });
                        fields.push(EventField {
                            data: Some(Variant::U32(purchase.inventory_slot)),
                            name: "inventory_slot".to_string(),
                        });
                        fields.push(EventField {
                            data: Some(Variant::I32(purchase.cost)),
                            name: "cost".to_string(),
                        });
                        fields.push(EventField {
                            data: Some(Variant::I32(self.tick)),
                            name: "tick".to_string(),
                        });
                        fields.push(EventField {
                            data: self.get_prop_from_ent(&WEAPON_FLOAT, &purchase.weapon_entid).ok(),
                            name: "float".to_string(),
                        });
                        fields.push(EventField {
                            data: self.find_weapon_skin(&purchase.weapon_entid).ok(),
                            name: "skin".to_string(),
                        });
                        fields.push(EventField {
                            data: self.find_weapon_skin_id(&purchase.weapon_entid).ok(),
                            name: "skin_id".to_string(),
                        });
                        fields.push(EventField {
                            data: self.get_prop_from_ent(&WEAPON_PAINT_SEED, &purchase.weapon_entid).ok(),
                            name: "paint_seed".to_string(),
                        });
                        fields.push(EventField {
                            data: self.find_stickers(&purchase.weapon_entid).ok(),
                            name: "stickers".to_string(),
                        });
                        let custom_name = if let Some(custom_name_id) = self.prop_controller.special_ids.custom_name {
                            self.get_prop_from_ent(&custom_name_id, &purchase.weapon_entid).ok()
                        } else {
                            None
                        };
                        fields.push(EventField {
                            data: custom_name,
                            name: "custom_name".to_string(),
                        });
                        fields.extend(self.find_extra_props_events(purchase.entid, "user"));
                        fields.extend(self.find_non_player_props());
                        let ge = GameEvent {
                            name: "item_purchase".to_string(),
                            fields,
                            tick: self.tick,
                        };
                        self.game_events.push(ge);
                        self.game_events_counter.insert("item_purchase".to_string());
                    }
                }
            }
        }
    }
    fn extract_win_reason(&self, events: &[GameEventInfo]) -> Option<Variant> {
        for event in events {
            if let GameEventInfo::RoundWinReason(reason) = event {
                match ROUND_WIN_REASON.get(&reason.reason) {
                    Some(name) => {
                        return Some(Variant::String(name.to_string()));
                    }
                    _ => return Some(Variant::String(reason.reason.to_string())),
                }
            }
        }
        None
    }
    fn extract_winner(&self, events: &[GameEventInfo]) -> Option<Variant> {
        for event in events {
            if let GameEventInfo::RoundWinReason(reason) = event {
                match ROUND_WIN_REASON_TO_WINNER.get(&reason.reason) {
                    Some(name) => {
                        return Some(Variant::String(name.to_string()));
                    }
                    _ => return Some(Variant::String(reason.reason.to_string())),
                }
            }
        }
        None
    }
    fn extract_round_end(&self, events: &[GameEventInfo]) -> Option<RoundEnd> {
        for event in events {
            if let GameEventInfo::RoundEnd(round_end) = event {
                return Some(round_end.clone());
            }
        }
        None
    }
    pub fn create_custom_event_round_end(&mut self, events: &[GameEventInfo]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("round_end".to_string());
        if !self.wanted_events.contains(&"round_end".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let event = match self.extract_round_end(&events) {
            Some(event) => event,
            None => return Ok(()),
        };
        if let (Some(Variant::U32(old)), Some(Variant::U32(new))) = (&event.old_value, &event.new_value) {
            if new - old != 1 {
                return Ok(());
            }
            let mut fields = vec![];
            fields.extend(self.find_non_player_props());
            fields.push(EventField {
                data: Some(Variant::U32(old + 1)),
                name: "round".to_string(),
            });
            fields.push(EventField {
                data: SecondPassParser::extract_win_reason(&self, &events),
                name: "reason".to_string(),
            });
            fields.push(EventField {
                data: SecondPassParser::extract_winner(&self, &events),
                name: "winner".to_string(),
            });
            fields.push(EventField {
                data: Some(Variant::I32(self.tick)),
                name: "tick".to_string(),
            });
            let ge = GameEvent {
                name: "round_end".to_string(),
                fields,
                tick: self.tick,
            };
            self.game_events.push(ge);
            self.game_events_counter.insert("rank_update".to_string());
        }

        Ok(())
    }

    pub fn create_custom_event_round_officially_ended(&mut self, _events: &[GameEventInfo]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("round_officially_ended".to_string());
        if !self.wanted_events.contains(&"round_officially_ended".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }

        // if round is 1 then we shouldn't publish `round_officially_ended`
        // as there is no prior round
        // keep an eye on this for potential bugs, possibly during match medic
        if let Some(Variant::I32(x)) = self.find_current_round() {
            if x <= 1 {
                return Ok(());
            }
        }

        let mut fields = vec![];
        fields.extend(self.find_non_player_props());

        fields.push(EventField {
            data: Some(Variant::I32(self.tick)),
            name: "tick".to_string(),
        });
        let ge = GameEvent {
            name: "round_officially_ended".to_string(),
            fields,
            tick: self.tick,
        };
        self.game_events.push(ge);

        Ok(())
    }

    pub fn create_custom_event_match_end(&mut self, _events: &[GameEventInfo]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("cs_win_panel_match".to_string());
        if !self.wanted_events.contains(&"cs_win_panel_match".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }

        let mut fields = vec![];
        fields.extend(self.find_non_player_props());
        fields.push(EventField {
            data: Some(Variant::I32(self.tick)),
            name: "tick".to_string(),
        });
        let ge = GameEvent {
            name: "cs_win_panel_match".to_string(),
            fields,
            tick: self.tick,
        };
        self.game_events.push(ge);

        Ok(())
    }

    pub fn find_current_round(&self) -> Option<Variant> {
        if let Some(prop_id) = self.prop_controller.special_ids.total_rounds_played {
            match self.rules_entity_id {
                Some(entid) => match self.get_prop_from_ent(&prop_id, &entid).ok() {
                    Some(Variant::I32(val)) => return Some(Variant::I32(val + 1)),
                    _ => {}
                },
                None => return None,
            }
        }
        None
    }
    pub fn create_custom_event_chat_message(&mut self, msg_bytes: &[u8]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("chat_message".to_string());
        if !self.wanted_events.contains(&"chat_message".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let chat_msg = match CUserMessageSayText2::decode(msg_bytes) {
            Ok(msg) => msg,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        let mut fields = vec![];
        let controller_id = chat_msg.entityindex();
        let res = self.find_user_by_controller_id(controller_id);
        let entity_id = match res {
            Some(metadata) => metadata.player_entity_id.unwrap_or(i32::MAX),
            _ => i32::MAX,
        };
        fields.push(self.create_player_name_field(entity_id, "user"));
        fields.push(self.create_player_steamid_field(entity_id, "user"));
        fields.extend(self.find_extra_props_events(entity_id, "user"));
        fields.push(EventField {
            data: Some(Variant::String(chat_msg.param2().to_owned())),
            name: "chat_message".to_string(),
        });
        fields.push(EventField {
            data: Some(Variant::I32(self.tick)),
            name: "tick".to_string(),
        });
        fields.extend(self.find_non_player_props());
        let ge = GameEvent {
            name: "chat_message".to_string(),
            fields,
            tick: self.tick,
        };
        self.game_events.push(ge);
        Ok(())
    }
    pub fn create_custom_event_server_message(&mut self, msg_bytes: &[u8]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("server_message".to_string());
        if !self.wanted_events.contains(&"server_message".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let chat_msg = match CUserMessageSayText::decode(msg_bytes) {
            Ok(msg) => msg,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        let mut fields = vec![];
        fields.push(EventField {
            data: Some(Variant::String(chat_msg.text().to_owned())),
            name: "server_message".to_string(),
        });
        fields.push(EventField {
            data: Some(Variant::I32(self.tick)),
            name: "tick".to_string(),
        });
        fields.extend(self.find_non_player_props());
        let ge = GameEvent {
            name: "server_message".to_string(),
            fields,
            tick: self.tick,
        };
        self.game_events.push(ge);
        Ok(())
    }

    pub fn create_custom_event_round_start(&mut self, _events: &[GameEventInfo]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("round_start".to_string());
        if !self.wanted_events.contains(&"round_start".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let mut fields = vec![];
        fields.push(EventField {
            data: self.find_current_round(),
            name: "round".to_string(),
        });
        fields.push(EventField {
            data: Some(Variant::I32(self.tick)),
            name: "tick".to_string(),
        });
        fields.extend(self.find_non_player_props());
        let ge = GameEvent {
            name: "round_start".to_string(),
            fields,
            tick: self.tick,
        };
        self.game_events.push(ge);
        Ok(())
    }

    pub fn create_custom_event_rank_update(&mut self, msg_bytes: &[u8]) -> Result<(), DemoParserError> {
        self.game_events_counter.insert("rank_update".to_string());
        if !self.wanted_events.contains(&"rank_update".to_string()) && self.wanted_events.first() != Some(&"all".to_string()) {
            return Ok(());
        }
        let update_msg = match CcsUsrMsgServerRankUpdate::decode(msg_bytes) {
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
            fields.push(EventField {
                data: Some(Variant::I32(self.tick)),
                name: "tick".to_string(),
            });
            fields.extend(self.find_non_player_props());
            let ge = GameEvent {
                name: "rank_update".to_string(),
                fields,
                tick: self.tick,
            };
            self.game_events.push(ge);
        }
        Ok(())
    }
    pub fn listen_for_events(
        entity: &mut Entity,
        result: &Variant,
        _field: &Field,
        field_info: Option<FieldInfo>,
        prop_controller: &PropController,
    ) -> Vec<GameEventInfo> {
        // Might want to start splitting this function
        let mut events = vec![];
        if let Some(fi) = field_info {
            // round end
            if let Some(id) = prop_controller.special_ids.round_end_count {
                if fi.prop_id == id {
                    events.push(GameEventInfo::RoundEnd(RoundEnd {
                        old_value: entity.props.get(&id).cloned(),
                        new_value: Some(result.clone()),
                    }));
                }
            }
            // Round win reason
            if let Some(id) = prop_controller.special_ids.round_win_reason {
                if fi.prop_id == id {
                    if let Variant::I32(reason) = result {
                        events.push(GameEventInfo::RoundWinReason(RoundWinReason { reason: *reason }));
                    }
                }
            }
            // freeze period start
            if let Some(id) = prop_controller.special_ids.round_start_count {
                if fi.prop_id == id {
                    events.push(GameEventInfo::FreezePeriodStart(true));
                }
            }
            if let Some(id) = prop_controller.special_ids.match_end_count {
                if fi.prop_id == id {
                    events.push(GameEventInfo::MatchEnd());
                }
            }
            use crate::first_pass::prop_controller::FLATTENED_VEC_MAX_LEN;
            use crate::first_pass::prop_controller::ITEM_PURCHASE_HANDLE;
            if fi.prop_id >= ITEM_PURCHASE_COST && fi.prop_id < ITEM_PURCHASE_COST + FLATTENED_VEC_MAX_LEN {
                events.push(GameEventInfo::WeaponCreateNCost((result.clone(), entity.entity_id)));
            }
            if fi.prop_id >= ITEM_PURCHASE_HANDLE && fi.prop_id < ITEM_PURCHASE_HANDLE + FLATTENED_VEC_MAX_LEN {
                events.push(GameEventInfo::WeaponCreateHitem((result.clone(), entity.entity_id)));
            }
            if fi.prop_id >= ITEM_PURCHASE_COUNT && fi.prop_id < ITEM_PURCHASE_COUNT + FLATTENED_VEC_MAX_LEN {
                events.push(GameEventInfo::WeaponPurchaseCount((result.clone(), entity.entity_id, fi.prop_id)));
            }
            if fi.prop_id >= ITEM_PURCHASE_DEF_IDX && fi.prop_id < ITEM_PURCHASE_DEF_IDX + FLATTENED_VEC_MAX_LEN {
                events.push(GameEventInfo::WeaponCreateDefIdx((result.clone(), entity.entity_id, fi.prop_id)));
            }
        }
        events
    }
}
// what is this shit
fn parse_key(key: &KeyT) -> Option<Variant> {
    match key.r#type() {
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
pub struct PurchaseEvent {
    pub entid: i32,
    pub cost: i32,
    pub name: Option<String>,
    pub weapon_entid: i32,
    pub inventory_slot: u32,
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
