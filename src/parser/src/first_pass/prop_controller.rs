use crate::first_pass::sendtables::Field;
use crate::first_pass::sendtables::Serializer;
use crate::first_pass::sendtables::ValueField;
use crate::maps::BUTTONMAP;
use crate::maps::TYPEHM;
use crate::second_pass::collect_data::PropType;
use crate::second_pass::parser_settings::SpecialIDs;
use ahash::AHashMap;

pub const PLAYER_ENTITY_HANDLE_MISSING: i32 = 2047;
pub const SPECTATOR_TEAM_NUM: u32 = 1;
pub const BUTTONS_BASEID: u32 = 100000;
pub const NORMAL_PROP_BASEID: u32 = 1000;
pub const WEAPON_SKIN_ID: u32 = 420420420;
pub const WEAPON_ORIGINGAL_OWNER_ID: u32 = 6942000;
pub const MY_WEAPONS_OFFSET: u32 = 500000;
pub const GRENADE_AMMO_ID: u32 = 1111111;
pub const INVENTORY_ID: u32 = 100000000;
pub const IS_ALIVE_ID: u32 = 100000001;
pub const GAME_TIME_ID: u32 = 100000002;
pub const ENTITY_ID_ID: u32 = 100000003;
pub const VELOCITY_X_ID: u32 = 100000004;
pub const VELOCITY_Y_ID: u32 = 100000005;
pub const VELOCITY_Z_ID: u32 = 100000006;
pub const VELOCITY_ID: u32 = 100000007;
pub const USERID_ID: u32 = 100000008;
pub const AGENT_SKIN_ID: u32 = 100000009;
pub const WEAPON_NAME_ID: u32 = 100000010;
pub const YAW_ID: u32 = 100000111;
pub const PITCH_ID: u32 = 100000012;
pub const TICK_ID: u32 = 100000013;
pub const STEAMID_ID: u32 = 100000014;
pub const NAME_ID: u32 = 100000015;
pub const PLAYER_X_ID: u32 = 100000016;
pub const PLAYER_Y_ID: u32 = 100000017;
pub const PLAYER_Z_ID: u32 = 100000018;

#[derive(Clone, Debug)]
pub struct PropController {
    pub id: u32,
    pub wanted_player_props: Vec<String>,
    pub wanted_prop_ids: Vec<u32>,
    pub prop_infos: Vec<PropInfo>,
    pub name_to_id: AHashMap<String, u32>,
    pub id_to_name: AHashMap<u32, String>,
    pub special_ids: SpecialIDs,
    pub real_name_to_og_name: AHashMap<String, String>,
    pub name_to_special_id: AHashMap<String, u32>,
    pub wanted_other_props: Vec<String>,
    pub event_with_velocity: bool,
    pub needs_velocity: bool,
    pub path_to_name: AHashMap<[i32; 7], String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropInfo {
    pub id: u32,
    pub prop_type: PropType,
    pub prop_name: String,
    pub prop_friendly_name: String,
    pub is_player_prop: bool,
}

pub enum PropCollectionType {
    Player,
    Rules,
    Team,
}

impl PropController {
    pub fn new(
        wanted_player_props: Vec<String>,
        wanted_other_props: Vec<String>,
        real_name_to_og_name: AHashMap<String, String>,
        needs_velocty: bool,
    ) -> Self {
        PropController {
            id: NORMAL_PROP_BASEID,
            wanted_player_props: wanted_player_props,
            wanted_prop_ids: vec![],
            prop_infos: vec![],
            name_to_id: AHashMap::default(),
            special_ids: SpecialIDs::new(),
            id_to_name: AHashMap::default(),
            name_to_special_id: AHashMap::default(),
            wanted_other_props: wanted_other_props,
            real_name_to_og_name: real_name_to_og_name,
            event_with_velocity: false,
            path_to_name: AHashMap::default(),
            needs_velocity: needs_velocty,
        }
    }
    pub fn set_custom_propinfos(&mut self) {
        let button_names = BUTTONMAP.keys();
        let mut someid = BUTTONS_BASEID;
        for bn in button_names {
            if self.wanted_player_props.contains(&(bn.to_string())) {
                self.prop_infos.push(PropInfo {
                    id: someid,
                    prop_type: PropType::Button,
                    prop_name: bn.to_string(),
                    prop_friendly_name: bn.to_string(),
                    is_player_prop: true,
                });
                someid += 1;
            }
        }
        if self
            .wanted_player_props
            .contains(&("active_weapon_original_owner".to_string()))
        {
            self.prop_infos.push(PropInfo {
                id: WEAPON_ORIGINGAL_OWNER_ID,
                prop_type: PropType::Custom,
                prop_name: "active_weapon_original_owner".to_string(),
                prop_friendly_name: "active_weapon_original_owner".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("inventory".to_string())) {
            self.prop_infos.push(PropInfo {
                id: INVENTORY_ID,
                prop_type: PropType::Custom,
                prop_name: "inventory".to_string(),
                prop_friendly_name: "inventory".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("user_id".to_string())) {
            self.prop_infos.push(PropInfo {
                id: USERID_ID,
                prop_type: PropType::Custom,
                prop_name: "user_id".to_string(),
                prop_friendly_name: "user_id".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("velocity_X".to_string())) {
            self.prop_infos.push(PropInfo {
                id: VELOCITY_X_ID,
                prop_type: PropType::Custom,
                prop_name: "velocity_X".to_string(),
                prop_friendly_name: "velocity_X".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("velocity_Y".to_string())) {
            self.prop_infos.push(PropInfo {
                id: VELOCITY_Y_ID,
                prop_type: PropType::Custom,
                prop_name: "velocity_Y".to_string(),
                prop_friendly_name: "velocity_Y".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("velocity_Z".to_string())) {
            self.prop_infos.push(PropInfo {
                id: VELOCITY_Z_ID,
                prop_type: PropType::Custom,
                prop_name: "velocity_Z".to_string(),
                prop_friendly_name: "velocity_Z".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("velocity".to_string())) {
            self.prop_infos.push(PropInfo {
                id: VELOCITY_ID,
                prop_type: PropType::Custom,
                prop_name: "velocity".to_string(),
                prop_friendly_name: "velocity".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("is_alive".to_string())) {
            self.prop_infos.push(PropInfo {
                id: IS_ALIVE_ID,
                prop_type: PropType::Custom,
                prop_name: "is_alive".to_string(),
                prop_friendly_name: "is_alive".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("entity_id".to_string())) {
            self.prop_infos.push(PropInfo {
                id: ENTITY_ID_ID,
                prop_type: PropType::Custom,
                prop_name: "entity_id".to_string(),
                prop_friendly_name: "entity_id".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("game_time".to_string())) {
            self.prop_infos.push(PropInfo {
                id: GAME_TIME_ID,
                prop_type: PropType::GameTime,
                prop_name: "game_time".to_string(),
                prop_friendly_name: "game_time".to_string(),
                is_player_prop: true,
            });
        }
        // Can also be non-player prop
        if self.wanted_other_props.contains(&("game_time".to_string())) {
            self.prop_infos.push(PropInfo {
                id: GAME_TIME_ID,
                prop_type: PropType::GameTime,
                prop_name: "game_time".to_string(),
                prop_friendly_name: "game_time".to_string(),
                is_player_prop: false,
            });
        }
        if self.wanted_player_props.contains(&("weapon_skin".to_string())) {
            self.prop_infos.push(PropInfo {
                id: WEAPON_SKIN_ID,
                prop_type: PropType::Custom,
                prop_name: "weapon_skin".to_string(),
                prop_friendly_name: "active_weapon_skin".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("weapon_name".to_string())) {
            self.prop_infos.push(PropInfo {
                id: WEAPON_NAME_ID,
                prop_type: PropType::Custom,
                prop_name: "weapon_name".to_string(),
                prop_friendly_name: "active_weapon_name".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("pitch".to_string())) {
            self.prop_infos.push(PropInfo {
                id: PITCH_ID,
                prop_type: PropType::Custom,
                prop_name: "pitch".to_string(),
                prop_friendly_name: "pitch".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("yaw".to_string())) {
            self.prop_infos.push(PropInfo {
                id: YAW_ID,
                prop_type: PropType::Custom,
                prop_name: "yaw".to_string(),
                prop_friendly_name: "yaw".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("agent_skin".to_string())) {
            self.prop_infos.push(PropInfo {
                id: AGENT_SKIN_ID,
                prop_type: PropType::Custom,
                prop_name: "agent_skin".to_string(),
                prop_friendly_name: "agent_skin".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("X".to_string())) {
            self.prop_infos.push(PropInfo {
                id: PLAYER_X_ID,
                prop_type: PropType::Custom,
                prop_name: "X".to_string(),
                prop_friendly_name: "X".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("Y".to_string())) {
            self.prop_infos.push(PropInfo {
                id: PLAYER_Y_ID,
                prop_type: PropType::Custom,
                prop_name: "Y".to_string(),
                prop_friendly_name: "Y".to_string(),
                is_player_prop: true,
            });
        }
        if self.wanted_player_props.contains(&("Z".to_string())) {
            self.prop_infos.push(PropInfo {
                id: PLAYER_Z_ID,
                prop_type: PropType::Custom,
                prop_name: "Z".to_string(),
                prop_friendly_name: "Z".to_string(),
                is_player_prop: true,
            });
        }
        self.prop_infos.push(PropInfo {
            id: TICK_ID,
            prop_type: PropType::Tick,
            prop_name: "tick".to_string(),
            prop_friendly_name: "tick".to_string(),
            is_player_prop: true,
        });

        self.prop_infos.push(PropInfo {
            id: STEAMID_ID,
            prop_type: PropType::Steamid,
            prop_name: "steamid".to_string(),
            prop_friendly_name: "steamid".to_string(),
            is_player_prop: true,
        });
        self.prop_infos.push(PropInfo {
            id: NAME_ID,
            prop_type: PropType::Name,
            prop_name: "name".to_string(),
            prop_friendly_name: "name".to_string(),
            is_player_prop: true,
        });
    }
    pub fn find_prop_name_paths(&mut self, ser: &mut Serializer) {
        self.traverse_fields(&mut ser.fields, ser.name.clone(), vec![])
    }
    fn set_id(&mut self, weap_prop: &str, f: &mut ValueField, is_grenade_or_weapon: bool) {
        match self.name_to_id.get(weap_prop) {
            // If we already have an id for prop of same name then use that id.
            // Mainly for weapon props. For example CAK47.m_iClip1 and CWeaponSCAR20.m_iClip1
            // are the "same" prop. (they have same path and we want to refer to it with one id not ~20)
            Some(id) => {
                f.prop_id = *id as u32;
                self.id_to_name.insert(*id, weap_prop.to_string());
                self.set_special_ids(&weap_prop, is_grenade_or_weapon, *id);
                return;
            }
            None => {
                self.name_to_id.insert(weap_prop.to_string(), self.id);
                self.id_to_name.insert(self.id, weap_prop.to_string());
                f.prop_id = self.id as u32;
                self.set_special_ids(&weap_prop, is_grenade_or_weapon, self.id);
            }
        }
    }

    fn insert_propinfo(&mut self, prop_name: &str, f: &mut ValueField) {
        if let Some(prop_type) = TYPEHM.get(&prop_name) {
            if self.wanted_player_props.contains(&prop_name.to_string()) {
                self.prop_infos.push(PropInfo {
                    id: f.prop_id as u32,
                    prop_type: *prop_type,
                    prop_name: prop_name.to_string(),
                    prop_friendly_name: self
                        .real_name_to_og_name
                        .get(&prop_name.to_string())
                        .unwrap_or(&prop_name.to_string())
                        .to_string(),
                    is_player_prop: true,
                })
            }
            if self.wanted_other_props.contains(&prop_name.to_string()) {
                self.prop_infos.push(PropInfo {
                    id: f.prop_id as u32,
                    prop_type: *prop_type,
                    prop_name: prop_name.to_string(),
                    prop_friendly_name: self
                        .real_name_to_og_name
                        .get(&prop_name.to_string())
                        .unwrap_or(&(prop_name.to_string()))
                        .to_string(),
                    is_player_prop: false,
                })
            }
        }
    }
    pub fn handle_prop(&mut self, full_name: &str, f: &mut ValueField, path: Vec<i32>) {
        f.full_name = full_name.to_string();
        // CAK47.m_iClip1 => ["CAK47", "m_iClip1"]
        let split_at_dot: Vec<&str> = full_name.split(".").collect();
        let is_weapon_prop = (split_at_dot[0].contains("Weapon") || split_at_dot[0].contains("AK"))
            && !split_at_dot[0].contains("Player")
            || split_at_dot[0].contains("Knife")
            || split_at_dot[0].contains("CDEagle")
            || split_at_dot[0].contains("C4")
            || split_at_dot[0].contains("Molo")
            || split_at_dot[0].contains("Inc")
            || split_at_dot[0].contains("Infer");

        let is_projectile_prop =
            (split_at_dot[0].contains("Projectile") || split_at_dot[0].contains("Grenade") || split_at_dot[0].contains("Flash"))
                && !split_at_dot[0].contains("Player");
        let is_grenade_or_weapon = is_weapon_prop || is_projectile_prop;

        // Strip first part of name from grenades and weapons.
        // if weapon prop: CAK47.m_iClip1 => m_iClip1
        // if grenade: CSmokeGrenadeProjectile.CBodyComponentBaseAnimGraph.m_cellX => CBodyComponentBaseAnimGraph.m_cellX
        let prop_name = match is_grenade_or_weapon {
            true => split_at_dot[1..].join("."),
            false => full_name.to_string(),
        };
        let mut a = [0, 0, 0, 0, 0, 0, 0];
        for (idx, v) in path.iter().enumerate() {
            a[idx] = *v;
        }
        self.path_to_name.insert(a, prop_name.to_string());

        let prop_already_exists = self.name_to_id.contains_key(&(prop_name).to_string());
        self.set_id(&prop_name, f, is_grenade_or_weapon);
        if !prop_already_exists {
            self.insert_propinfo(&prop_name, f);
        }
        f.should_parse = true;
        if full_name == "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hMyWeapons" {
            f.prop_id = MY_WEAPONS_OFFSET as u32;
        }
        if prop_name.contains("CEconItemAttribute.m_iRawValue32") {
            f.prop_id = WEAPON_SKIN_ID as u32;
        }
        self.id += 1;
    }

    fn set_special_ids(&mut self, name: &str, is_grenade_or_weapon: bool, id: u32) {
        if is_grenade_or_weapon {
            match name {
                "m_hOwnerEntity" => self.special_ids.h_owner_entity = Some(id),
                "m_nOwnerId" => self.special_ids.grenade_owner_id = Some(id),
                "CBodyComponentBaseAnimGraph.m_vecX" => self.special_ids.m_vec_x_grenade = Some(id),
                "CBodyComponentBaseAnimGraph.m_vecY" => self.special_ids.m_vec_y_grenade = Some(id),
                "CBodyComponentBaseAnimGraph.m_vecZ" => self.special_ids.m_vec_z_grenade = Some(id),
                "CBodyComponentBaseAnimGraph.m_cellX" => self.special_ids.m_cell_x_grenade = Some(id),
                "CBodyComponentBaseAnimGraph.m_cellY" => self.special_ids.m_cell_y_grenade = Some(id),
                "CBodyComponentBaseAnimGraph.m_cellZ" => self.special_ids.m_cell_z_grenade = Some(id),
                "m_iItemDefinitionIndex" => self.special_ids.item_def = Some(id),
                "m_OriginalOwnerXuidLow" => self.special_ids.orig_own_low = Some(id),
                "m_OriginalOwnerXuidHigh" => self.special_ids.orig_own_high = Some(id),
                _ => {}
            };
        } else {
            match name {
                "CCSGameRulesProxy.CCSGameRules.m_bFreezePeriod" => self.special_ids.is_freeze_period = Some(id),
                "CCSGameRulesProxy.CCSGameRules.m_fRoundStartTime" => self.special_ids.round_start_time = Some(id),
                "CCSGameRulesProxy.CCSGameRules.m_eRoundWinReason" => self.special_ids.round_win_reason = Some(id),
                "CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed" => self.special_ids.total_rounds_played = Some(id),
                "CCSTeam.m_iTeamNum" => self.special_ids.team_team_num = Some(id),
                "CCSPlayerPawn.m_iTeamNum" => self.special_ids.player_team_pointer = Some(id),
                "CBasePlayerWeapon.m_nOwnerId" => self.special_ids.weapon_owner_pointer = Some(id),
                "CCSPlayerController.m_iTeamNum" => self.special_ids.teamnum = Some(id),
                "CCSPlayerController.m_iszPlayerName" => self.special_ids.player_name = Some(id),
                "CCSPlayerController.m_steamID" => self.special_ids.steamid = Some(id),
                "CCSPlayerController.m_hPlayerPawn" => self.special_ids.player_pawn = Some(id),
                "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellX" => self.special_ids.cell_x_player = Some(id),
                "CCSPlayerPawn.CCSPlayer_MovementServices.m_nButtonDownMaskPrev" => self.special_ids.buttons = Some(id),
                "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecX" => self.special_ids.cell_x_offset_player = Some(id),
                "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellY" => self.special_ids.cell_y_player = Some(id),
                "CCSPlayerPawn.m_angEyeAngles" => self.special_ids.eye_angles = Some(id),
                "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecY" => self.special_ids.cell_y_offset_player = Some(id),
                "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellZ" => self.special_ids.cell_z_player = Some(id),
                "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecZ" => self.special_ids.cell_z_offset_player = Some(id),
                "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon" => self.special_ids.active_weapon = Some(id),
                "CCSPlayerPawn.m_lifeState" => self.special_ids.life_state = Some(id),
                "CCSPlayerController.m_nPawnCharacterDefIndex" => self.special_ids.agent_skin_idx = Some(id),
                _ => {}
            };
        }
    }
    fn traverse_fields(&mut self, fields: &mut Vec<Field>, ser_name: String, path_og: Vec<i32>) {
        for (idx, f) in fields.iter_mut().enumerate() {
            let mut path = path_og.clone();
            path.push(idx as i32);
            match f {
                Field::Value(x) => {
                    let full_name = ser_name.clone() + "." + &x.name;
                    self.handle_prop(&full_name, x, path);
                }
                Field::Serializer(ser) => self.traverse_fields(
                    &mut ser.serializer.fields,
                    ser_name.clone() + "." + &ser.serializer.name,
                    path.clone(),
                ),
                Field::Pointer(ser) => self.traverse_fields(
                    &mut ser.serializer.fields,
                    ser_name.clone() + "." + &ser.serializer.name,
                    path.clone(),
                ),
                Field::Array(ser) => match &mut ser.field_enum.as_mut() {
                    Field::Value(v) => {
                        self.handle_prop(&(ser_name.clone() + "." + &v.name), v, path);
                    }
                    _ => {}
                },
                Field::Vector(_x) => {
                    let vec_path = path.clone();
                    if let Ok(inner) = f.get_inner_mut(0) {
                        match inner {
                            Field::Serializer(s) => {
                                for (inner_idx, f) in &mut s.serializer.fields.iter_mut().enumerate() {
                                    match f {
                                        Field::Value(v) => {
                                            let mut myp = vec_path.clone();
                                            myp.push(inner_idx as i32);
                                            self.handle_prop(&(ser_name.clone() + "." + &v.name), v, myp);
                                        }
                                        _ => {}
                                    }
                                }
                                self.traverse_fields(
                                    &mut s.serializer.fields,
                                    ser_name.clone() + "." + &s.serializer.name,
                                    path_og.clone(),
                                )
                            }
                            Field::Value(x) => {
                                self.handle_prop(&(ser_name.clone() + "." + &x.name), x, path.clone());
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
