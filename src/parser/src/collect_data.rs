use super::entities::PlayerMetaData;
use super::variants::Variant;
use crate::maps::BUTTONMAP;
use crate::maps::WEAPINDICIES;
use crate::parser_thread_settings::ParserThread;
use crate::prop_controller::PropInfo;
use crate::variants::PropColumn;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropType {
    Team,
    Rules,
    Custom,
    Controller,
    Player,
    PlayerVec,
    Weapon,
    Button,
    Name,
    Steamid,
    Tick,
}
#[derive(Debug, PartialEq)]
// While this is an error, its very common and doesn't exactly signal
// that anything went "wrong", just that prop was not found.
// We can do this cause rusts errors like these are very cheap :) (not ok with exceptions).
// Serves mostly as help for debugging.
// The point is to be able to track why a prop was not found, without
// having to go and add a bunch of prints everywhere
pub enum PropCollectionError {
    PlayerSpecialIDCellXMissing,
    PlayerSpecialIDCellYMissing,
    PlayerSpecialIDCellZMissing,
    PlayerSpecialIDOffsetXMissing,
    PlayerSpecialIDOffsetYMissing,
    PlayerSpecialIDOffsetZMissing,
    GrenadeSpecialIDCellXMissing,
    GrenadeSpecialIDCellYMissing,
    GrenadeSpecialIDCellZMissing,
    GrenadeSpecialIDOffsetXMissing,
    GrenadeSpecialIDOffsetYMissing,
    GrenadeSpecialIDOffsetZMissing,
    CoordinateOffsetNone,
    CoordinateCellNone,
    CoordinateIncorrectTypes,
    CoordinateBothNone,
    GrenadeOffsetVariantNone,
    PlayerMetaDataNameNone,
    ButtonsSpecialIDNone,
    ButtonsMapNoEntryFound,
    GetPropFromEntEntityNotFound,
    GetPropFromEntPropNotFound,
    ButtonMaskNotU64Variant,
    RulesEntityIdNotSet,
    ControllerEntityIdNotSet,
    SpecialidsEyeAnglesNotSet,
    SpecialidsItemDefNotSet,
    EyeAnglesWrongVariant,
    WeaponIdxMappingNotFound,
    WeaponDefVariantWrongType,
    SpecialidsPlayerTeamPointerNotSet,
    TeamNumIncorrectVariant,
    IllegalTeamValue,
    TeamEntityIdNotSet,
    GrenadeOwnerIdNotSet,
    GrenadeOwnerIdPropIncorrectVariant,
    PlayerNotFound,
    SpecialidsActiveWeaponNotSet,
    WeaponHandleIncorrectVariant,
    UnknownCustomPropName,
}
// DONT KNOW IF THESE ARE CORRECT. SEEMS TO GIVE CORRECT VALUES
const CELL_BITS: i32 = 9;
const MAX_COORD: f32 = (1 << 14) as f32;

impl std::error::Error for PropCollectionError {}
impl fmt::Display for PropCollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectileRecord {
    pub steamid: Option<u64>,
    pub name: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub tick: Option<i32>,
    pub grenade_type: Option<String>,
}

// This file collects the data that is converted into a dataframe in the end in parser.parse_ticks()

impl ParserThread {
    pub fn collect_entities(&mut self) {
        if !self.wanted_ticks.contains(&self.tick) && self.wanted_ticks.len() != 0 || self.wanted_event.is_some() {
            return;
        }
        if self.parse_projectiles {
            self.collect_projectiles();
        }
        // iterate every player and every wanted prop name
        // if either one is missing then push None to output
        for (entity_id, player) in &self.players {
            for prop_info in &self.prop_controller.prop_infos {
                // All values come trough here. None if cant be found.
                match self.find_prop(prop_info, entity_id, player) {
                    Ok(prop) => {
                        println!("{:?}", prop);
                        self.output
                            .entry(prop_info.id)
                            .or_insert_with(|| PropColumn::new())
                            .push(Some(prop));
                    }
                    Err(_e) => {
                        // Ultimate debugger is to print this error
                        println!("{:?}", _e);
                        self.output
                            .entry(prop_info.id)
                            .or_insert_with(|| PropColumn::new())
                            .push(None);
                    }
                }
            }
        }
    }
    pub fn find_prop(
        &self,
        prop_info: &PropInfo,
        entity_id: &i32,
        player: &PlayerMetaData,
    ) -> Result<Variant, PropCollectionError> {
        match prop_info.prop_type {
            Some(PropType::Tick) => return self.create_tick(),
            Some(PropType::Name) => return self.create_name(player),
            Some(PropType::Steamid) => return self.create_steamid(player),
            Some(PropType::Player) => return self.get_prop_from_ent(&prop_info.id, &entity_id),
            Some(PropType::Team) => return self.find_team_prop(&prop_info.id, &entity_id),
            Some(PropType::Custom) => self.create_custom_prop(prop_info.prop_name.as_str(), entity_id),
            Some(PropType::Weapon) => return self.find_weapon_prop(&prop_info.id, &entity_id),
            Some(PropType::Button) => return self.get_button_prop(&prop_info, &entity_id),
            Some(PropType::Controller) => return self.get_controller_prop(prop_info, player),
            Some(PropType::Rules) => return self.get_rules_prop(prop_info),
            _ => panic!("no type for: {:?}", prop_info),
        }
    }
    pub fn get_prop_from_ent(&self, prop_id: &u32, entity_id: &i32) -> Result<Variant, PropCollectionError> {
        match self.entities.get(entity_id) {
            None => return Err(PropCollectionError::GetPropFromEntEntityNotFound),
            Some(e) => match e.props.get(&prop_id) {
                None => return Err(PropCollectionError::GetPropFromEntPropNotFound),
                Some(prop) => return Ok(prop.clone()),
            },
        }
    }
    fn create_tick(&self) -> Result<Variant, PropCollectionError> {
        // This can't actually fail
        return Ok(Variant::I32(self.tick));
    }
    fn create_steamid(&self, player: &PlayerMetaData) -> Result<Variant, PropCollectionError> {
        match player.steamid {
            Some(steamid) => return Ok(Variant::U64(steamid)),
            // Revisit this as it was related to pandas null support with u64's
            _ => return Ok(Variant::U64(0)),
        }
    }
    fn create_name(&self, player: &PlayerMetaData) -> Result<Variant, PropCollectionError> {
        match &player.name {
            Some(name) => return Ok(Variant::String(name.to_string())),
            _ => return Err(PropCollectionError::PlayerMetaDataNameNone),
        }
    }
    pub fn get_button_prop(&self, prop_info: &PropInfo, entity_id: &i32) -> Result<Variant, PropCollectionError> {
        match self.prop_controller.special_ids.buttons {
            None => Err(PropCollectionError::ButtonsSpecialIDNone),
            Some(button_id) => match self.get_prop_from_ent(&button_id, &entity_id) {
                Ok(Variant::U64(button_mask)) => match BUTTONMAP.get(&prop_info.prop_name) {
                    Some(button_flag) => Ok(Variant::Bool(button_mask & button_flag != 0)),
                    None => return Err(PropCollectionError::ButtonsMapNoEntryFound),
                },
                Ok(_) => return Err(PropCollectionError::ButtonMaskNotU64Variant),
                Err(e) => Err(e),
            },
        }
    }
    pub fn get_rules_prop(&self, prop_info: &PropInfo) -> Result<Variant, PropCollectionError> {
        match self.rules_entity_id {
            Some(entid) => return self.get_prop_from_ent(&prop_info.id, &entid),
            None => return Err(PropCollectionError::RulesEntityIdNotSet),
        }
    }
    pub fn get_controller_prop(&self, prop_info: &PropInfo, player: &PlayerMetaData) -> Result<Variant, PropCollectionError> {
        match player.controller_entid {
            Some(entid) => return self.get_prop_from_ent(&prop_info.id, &entid),
            None => return Err(PropCollectionError::ControllerEntityIdNotSet),
        }
    }
    pub fn find_thrower_steamid(&self, entity_id: &i32) -> Result<u64, PropCollectionError> {
        let owner_id = match self.prop_controller.special_ids.grenade_owner_id {
            Some(owner_id) => owner_id,
            None => return Err(PropCollectionError::GrenadeOwnerIdNotSet),
        };
        let owner_entid = match self.get_prop_from_ent(&owner_id, entity_id) {
            Ok(Variant::U32(prop)) => prop & 0x7FF,
            Ok(_) => return Err(PropCollectionError::GrenadeOwnerIdPropIncorrectVariant),
            Err(e) => return Err(e),
        };
        match self.players.get(&(owner_entid as i32)) {
            Some(metadata) => match metadata.steamid {
                Some(s) => Ok(s),
                // Watch out
                None => Ok(0),
            },
            None => Err(PropCollectionError::PlayerNotFound),
        }
    }
    pub fn find_thrower_name(&self, entity_id: &i32) -> Result<String, PropCollectionError> {
        let owner_id = match self.prop_controller.special_ids.grenade_owner_id {
            Some(owner_id) => owner_id,
            None => return Err(PropCollectionError::GrenadeOwnerIdNotSet),
        };
        let owner_entid = match self.get_prop_from_ent(&owner_id, entity_id) {
            Ok(Variant::U32(prop)) => prop & 0x7FF,
            Ok(_) => return Err(PropCollectionError::GrenadeOwnerIdPropIncorrectVariant),
            Err(e) => return Err(e),
        };
        match self.players.get(&(owner_entid as i32)) {
            Some(metadata) => match &metadata.name {
                Some(s) => Ok(s.to_owned()),
                // Watch out
                None => Err(PropCollectionError::PlayerMetaDataNameNone),
            },
            None => Err(PropCollectionError::PlayerNotFound),
        }
    }

    fn find_grenade_type(&self, entity_id: &i32) -> Option<String> {
        if let Some(ent) = self.entities.get(&entity_id) {
            if let Some(cls) = self.cls_by_id.get(&ent.cls_id).as_ref() {
                if !cls.name.contains("Grenade") {
                    return None;
                }
                // remove extra from name: CSmokeGrenadeProjectile --> SmokeGrenade
                // Todo maybe make names like this: smoke_grenade or just "smoke"
                let mut clean_name = cls.name[1..].split_at(cls.name.len() - 11).0;
                // Seems like the only exception
                if clean_name == "BaseCSGrenade" {
                    clean_name = "HeGrenade"
                }
                return Some(clean_name.to_owned());
            }
        }
        None
    }

    pub fn collect_projectiles(&mut self) {
        for projectile_entid in &self.projectiles {
            let grenade_type = self.find_grenade_type(projectile_entid);
            let steamid = self.find_thrower_steamid(projectile_entid);
            let name = self.find_thrower_name(projectile_entid);
            let x = self.collect_cell_coordinate_grenade("X", projectile_entid);
            let y = self.collect_cell_coordinate_grenade("Y", projectile_entid);
            let z = self.collect_cell_coordinate_grenade("Z", projectile_entid);

            // Watch out with these
            let float_x = match x {
                Ok(Variant::F32(p)) => Some(p),
                Ok(_) => None,
                Err(_) => None,
            };
            let float_y = match y {
                Ok(Variant::F32(p)) => Some(p),
                Ok(_) => None,
                Err(_) => None,
            };
            let float_z = match z {
                Ok(Variant::F32(p)) => Some(p),
                Ok(_) => None,
                Err(_) => None,
            };
            let steamid = match steamid {
                Ok(p) => Some(p),
                Err(_) => None,
            };
            let name = match name {
                Ok(p) => Some(p),
                Err(_) => None,
            };

            self.projectile_records.push(ProjectileRecord {
                steamid: steamid,
                name: name,
                x: float_x,
                y: float_y,
                z: float_z,
                tick: Some(self.tick),
                grenade_type: grenade_type,
            });
        }
    }

    fn find_weapon_name(&self, entity_id: &i32) -> Result<Variant, PropCollectionError> {
        let item_def_id = match self.prop_controller.special_ids.item_def {
            Some(x) => x,
            None => return Err(PropCollectionError::SpecialidsItemDefNotSet),
        };
        match self.find_weapon_prop(&item_def_id, entity_id) {
            Ok(Variant::U32(def_idx)) => match WEAPINDICIES.get(&def_idx) {
                Some(v) => return Ok(Variant::String(v.to_string())),
                None => return Err(PropCollectionError::WeaponIdxMappingNotFound),
            },
            Ok(_) => return Err(PropCollectionError::WeaponDefVariantWrongType),
            Err(e) => Err(e),
        }
    }
    pub fn collect_cell_coordinate_player(&self, axis: &str, entity_id: &i32) -> Result<Variant, PropCollectionError> {
        let coordinate = match axis {
            "X" => {
                let x_prop_id = match self.prop_controller.special_ids.cell_x_player {
                    Some(x) => x,
                    None => return Err(PropCollectionError::PlayerSpecialIDCellXMissing),
                };
                let x_offset_id = match self.prop_controller.special_ids.cell_x_offset_player {
                    Some(x) => x,
                    None => return Err(PropCollectionError::PlayerSpecialIDOffsetXMissing),
                };
                let offset = self.get_prop_from_ent(&x_offset_id, entity_id);
                let cell = self.get_prop_from_ent(&x_prop_id, entity_id);
                coord_from_cell(cell, offset)
            }
            "Y" => {
                let y_prop_id = match self.prop_controller.special_ids.cell_y_player {
                    Some(y) => y,
                    None => return Err(PropCollectionError::PlayerSpecialIDCellYMissing),
                };
                let y_offset_id = match self.prop_controller.special_ids.cell_y_offset_player {
                    Some(y) => y,
                    None => return Err(PropCollectionError::PlayerSpecialIDOffsetYMissing),
                };

                let offset = self.get_prop_from_ent(&y_offset_id, entity_id);
                let cell = self.get_prop_from_ent(&y_prop_id, entity_id);
                coord_from_cell(cell, offset)
            }
            "Z" => {
                let z_prop_id = match self.prop_controller.special_ids.cell_z_player {
                    Some(z) => z,
                    None => return Err(PropCollectionError::PlayerSpecialIDCellZMissing),
                };
                let z_offset_id = match self.prop_controller.special_ids.cell_z_offset_player {
                    Some(z) => z,
                    None => return Err(PropCollectionError::PlayerSpecialIDOffsetZMissing),
                };
                let offset = self.get_prop_from_ent(&z_offset_id, entity_id);
                let cell = self.get_prop_from_ent(&z_prop_id, entity_id);
                coord_from_cell(cell, offset)
            }
            _ => panic!("Unknown axis: {}", axis),
        };
        Ok(Variant::F32(coordinate?))
    }
    pub fn collect_cell_coordinate_grenade(&self, axis: &str, entity_id: &i32) -> Result<Variant, PropCollectionError> {
        // Todo rename to be consistent with player special ids
        let coordinate = match axis {
            "X" => {
                let x_prop_id = match self.prop_controller.special_ids.m_cell_x_grenade {
                    Some(x) => x,
                    None => return Err(PropCollectionError::GrenadeSpecialIDCellXMissing),
                };
                let x_offset_id = match self.prop_controller.special_ids.m_vec_x_grenade {
                    Some(x) => x,
                    None => return Err(PropCollectionError::GrenadeSpecialIDOffsetXMissing),
                };
                let offset = self.get_prop_from_ent(&x_offset_id, entity_id);
                let cell = self.get_prop_from_ent(&x_prop_id, entity_id);
                coord_from_cell(cell, offset)
            }
            "Y" => {
                let y_prop_id = match self.prop_controller.special_ids.m_cell_y_grenade {
                    Some(y) => y,
                    None => return Err(PropCollectionError::GrenadeSpecialIDCellYMissing),
                };
                let y_offset_id = match self.prop_controller.special_ids.m_vec_y_grenade {
                    Some(y) => y,
                    None => return Err(PropCollectionError::GrenadeSpecialIDOffsetYMissing),
                };

                let offset = self.get_prop_from_ent(&y_offset_id, entity_id);
                let cell = self.get_prop_from_ent(&y_prop_id, entity_id);
                coord_from_cell(cell, offset)
            }
            "Z" => {
                let z_prop_id = match self.prop_controller.special_ids.m_cell_z_grenade {
                    Some(z) => z,
                    None => return Err(PropCollectionError::GrenadeSpecialIDCellZMissing),
                };
                let z_offset_id = match self.prop_controller.special_ids.m_vec_z_grenade {
                    Some(z) => z,
                    None => return Err(PropCollectionError::GrenadeSpecialIDOffsetZMissing),
                };
                let offset = self.get_prop_from_ent(&z_offset_id, entity_id);
                let cell = self.get_prop_from_ent(&z_prop_id, entity_id);
                coord_from_cell(cell, offset)
            }
            _ => panic!("Unknown axis: {}", axis),
        };
        Ok(Variant::F32(coordinate?))
    }

    fn find_pitch_or_yaw(&self, entity_id: &i32, idx: usize) -> Result<Variant, PropCollectionError> {
        match self.prop_controller.special_ids.eye_angles {
            Some(prop_id) => match self.get_prop_from_ent(&prop_id, entity_id) {
                Ok(Variant::VecXYZ(v)) => return Ok(Variant::F32(v[idx])),
                Ok(_) => return Err(PropCollectionError::EyeAnglesWrongVariant),
                Err(e) => return Err(e),
            },
            None => Err(PropCollectionError::SpecialidsEyeAnglesNotSet),
        }
    }
    pub fn create_custom_prop(&self, prop_name: &str, entity_id: &i32) -> Result<Variant, PropCollectionError> {
        match prop_name {
            "X" => self.collect_cell_coordinate_player("X", entity_id),
            "Y" => self.collect_cell_coordinate_player("Y", entity_id),
            "Z" => self.collect_cell_coordinate_player("Z", entity_id),
            "pitch" => self.find_pitch_or_yaw(entity_id, 0),
            "yaw" => self.find_pitch_or_yaw(entity_id, 1),
            "weapon_name" => self.find_weapon_name(entity_id),
            _ => Err(PropCollectionError::UnknownCustomPropName),
        }
    }
    pub fn find_weapon_prop(&self, prop: &u32, player_entid: &i32) -> Result<Variant, PropCollectionError> {
        let p = match self.prop_controller.special_ids.active_weapon {
            Some(p) => p,
            None => return Err(PropCollectionError::SpecialidsActiveWeaponNotSet),
        };
        match self.get_prop_from_ent(&p, player_entid) {
            Ok(Variant::U32(weap_handle)) => {
                // Could be more specific
                let weapon_entity_id = (weap_handle & 0x7FF) as i32;
                self.get_prop_from_ent(&prop, &weapon_entity_id)
            }
            Ok(_) => Err(PropCollectionError::WeaponHandleIncorrectVariant),
            Err(e) => Err(e),
        }
    }
    pub fn find_team_prop(&self, prop: &u32, player_entid: &i32) -> Result<Variant, PropCollectionError> {
        match self.prop_controller.special_ids.player_team_pointer {
            None => return Err(PropCollectionError::SpecialidsPlayerTeamPointerNotSet),
            Some(p) => {
                match self.get_prop_from_ent(&p, player_entid) {
                    Ok(Variant::U32(team_num)) => {
                        let team_entid = match team_num {
                            // 1 should be spectator
                            1 => self.teams.team1_entid,
                            2 => self.teams.team2_entid,
                            3 => self.teams.team3_entid,
                            _ => return Err(PropCollectionError::IllegalTeamValue),
                        };
                        // Get prop from team entity
                        match team_entid {
                            Some(eid) => return self.get_prop_from_ent(prop, &eid),
                            None => return Err(PropCollectionError::TeamEntityIdNotSet),
                        }
                    }
                    Ok(_) => Err(PropCollectionError::TeamNumIncorrectVariant),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

fn coord_from_cell(
    cell: Result<Variant, PropCollectionError>,
    offset: Result<Variant, PropCollectionError>,
) -> Result<f32, PropCollectionError> {
    // Both cell and offset are needed for calculation
    match (offset, cell) {
        (Ok(Variant::F32(offset)), Ok(Variant::U32(cell))) => {
            let cell_coord = ((cell as f32 * (1 << CELL_BITS) as f32) - MAX_COORD) as f32;
            Ok(cell_coord + offset)
        }
        (Err(_), Err(_)) => Err(PropCollectionError::CoordinateBothNone),
        (Ok(Variant::F32(_offset)), Err(_)) => Err(PropCollectionError::CoordinateCellNone),
        (Err(_), Ok(Variant::U32(_cell))) => Err(PropCollectionError::CoordinateOffsetNone),
        (_, _) => Err(PropCollectionError::CoordinateIncorrectTypes),
    }
}

#[cfg(test)]
mod tests {
    use crate::collect_data::PropCollectionError;
    use crate::entities::Entity;
    use crate::entities::EntityType;
    use crate::entities::PlayerMetaData;
    use crate::maps::BUTTONMAP;
    use crate::parser_settings::Parser;
    use crate::prop_controller::PropInfo;
    use crate::variants::*;
    use crate::{parser_settings::ParserInputs, parser_thread_settings::ParserThread, prop_controller::PropController};
    use ahash::AHashMap;
    use memmap2::MmapOptions;
    use std::fs::File;
    use std::sync::Arc;

    use super::PropType;
    const PLAYER_ENTITY_ID: i32 = 1;
    const WANTED_PROP_ID: u32 = 2;
    const THIS_PLAYERS_CONTROLLER_ID: i32 = 3;
    const RULES_ENTITY_ID: i32 = 4;
    const BUTTONS_SPECIAL_ID: u32 = 5;
    const PLAYER_TEAM_POINTER_SPECIAL_ID: u32 = 6;
    const TEAM_ENTITY_ID: i32 = 7;
    const PLAYER_STEAMID: u64 = 76511234567899874;
    const PLAYER_NAME: &str = "PLAYER_NAME";
    const CELL_PROP_ID: u32 = 8;
    const OFFSET_PROP_ID: u32 = 9;
    const CELL_X_PLAYER_ID: u32 = 10;
    const CELL_Y_PLAYER_ID: u32 = 11;
    const CELL_Z_PLAYER_ID: u32 = 12;
    const OFFSET_X_PLAYER_ID: u32 = 13;
    const OFFSET_Y_PLAYER_ID: u32 = 14;
    const OFFSET_Z_PLAYER_ID: u32 = 15;

    fn default_setup() -> (ParserThread, PlayerMetaData) {
        let file = File::open("src/collect_data.rs").unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            bytes: Arc::new(mmap),
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: None,
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: Arc::new(vec![]),
        };
        let player_md = PlayerMetaData {
            controller_entid: Some(THIS_PLAYERS_CONTROLLER_ID),
            name: Some(PLAYER_NAME.to_string()),
            player_entity_id: Some(PLAYER_ENTITY_ID),
            steamid: Some(PLAYER_STEAMID),
            team_num: Some(3),
        };
        let parser = Parser::new(settings);
        let input = parser.create_parser_thread_input(0, false);
        let parser_thread = ParserThread::new(input).unwrap();
        (parser_thread, player_md)
    }
    #[test]
    fn test_player_x() {
        let (mut parser_thread, player_md) = default_setup();
        let mut player_props = AHashMap::default();

        let mut prop_controller_new = PropController::new(vec![], vec![], AHashMap::default());
        // prop_controller_new.special_ids.cell_x_player = Some(CELL_X_PLAYER_ID);
        prop_controller_new.special_ids.cell_x_offset_player = Some(OFFSET_X_PLAYER_ID);
        parser_thread.prop_controller = Arc::new(prop_controller_new);

        let cell = Variant::U32(10);
        let offset = Variant::F32(59.0);

        player_props.insert(CELL_PROP_ID, cell);
        player_props.insert(OFFSET_PROP_ID, offset);

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };

        parser_thread.entities.insert(player.entity_id, player.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Custom),
            prop_name: "X".to_string(),
            prop_friendly_name: "X".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::GetPropFromEntPropNotFound), prop);
    }
    #[test]
    fn test_create_tick() {
        let (mut parser_thread, player_md) = default_setup();
        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Tick),
            prop_name: "tick".to_string(),
            prop_friendly_name: "tick".to_string(),
        };
        parser_thread.tick = 5555555;
        let prop = parser_thread.find_prop(&prop_info, &69, &player_md);
        assert_eq!(Ok(Variant::I32(5555555)), prop);
    }
    #[test]
    fn test_create_steamid() {
        let (mut parser_thread, player_md) = default_setup();
        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Steamid),
            prop_name: "steamid".to_string(),
            prop_friendly_name: "steamid".to_string(),
        };
        parser_thread
            .players
            .insert(player_md.player_entity_id.unwrap(), player_md.clone());
        let prop = parser_thread.find_prop(&prop_info, &69, &player_md);
        assert_eq!(Ok(Variant::U64(PLAYER_STEAMID)), prop);
    }
    #[test]
    fn test_create_name() {
        let (mut parser_thread, player_md) = default_setup();
        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Name),
            prop_name: "name".to_string(),
            prop_friendly_name: "name".to_string(),
        };
        parser_thread
            .players
            .insert(player_md.player_entity_id.unwrap(), player_md.clone());
        let prop = parser_thread.find_prop(&prop_info, &69, &player_md);
        assert_eq!(Ok(Variant::String(PLAYER_NAME.to_string())), prop);
    }
    #[test]
    fn test_get_prop_from_ent_no_entity_found() {
        let (parser_thread, player_md) = default_setup();
        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Controller),
            prop_name: "WINS".to_string(),
            prop_friendly_name: "WINS".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &999999, &player_md);
        assert_eq!(Err(PropCollectionError::GetPropFromEntEntityNotFound), prop);
    }
    #[test]
    fn test_get_prop_from_ent_no_prop_found() {
        let (mut parser_thread, player_md) = default_setup();
        let player_props = AHashMap::default();
        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Player),
            prop_name: "WINS".to_string(),
            prop_friendly_name: "WINS".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::GetPropFromEntPropNotFound), prop);
    }

    #[test]
    fn test_controller_prop_found() {
        let (mut parser_thread, player_md) = default_setup();

        let mut controller_props = AHashMap::default();
        controller_props.insert(WANTED_PROP_ID, Variant::I32(555));
        let player_props = AHashMap::default();
        // CCSPlayerController.m_iCompetitiveWins
        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let controller = Entity {
            cls_id: 0,
            entity_id: THIS_PLAYERS_CONTROLLER_ID,
            props: controller_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(controller.entity_id, controller.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Controller),
            prop_name: "WINS".to_string(),
            prop_friendly_name: "WINS".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Ok(Variant::I32(555)), prop);
    }
    #[test]
    fn test_controller_prop_not_found() {
        let (mut parser_thread, player_md) = default_setup();
        let controller_props = AHashMap::default();
        let player_props = AHashMap::default();

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let controller = Entity {
            cls_id: 0,
            entity_id: THIS_PLAYERS_CONTROLLER_ID,
            props: controller_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(controller.entity_id, controller.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Controller),
            prop_name: "WINS".to_string(),
            prop_friendly_name: "WINS".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::GetPropFromEntPropNotFound), prop);
    }
    #[test]
    fn test_rules_prop_found() {
        let (mut parser_thread, player_md) = default_setup();
        let mut rules_props = AHashMap::default();
        let player_props = AHashMap::default();

        rules_props.insert(WANTED_PROP_ID, Variant::U32(33333));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let rules = Entity {
            cls_id: 0,
            entity_id: RULES_ENTITY_ID,
            props: rules_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(rules.entity_id, rules.clone());

        // EHH odd place to store this
        parser_thread.rules_entity_id = Some(RULES_ENTITY_ID);

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Rules),
            prop_name: "WINS".to_string(),
            prop_friendly_name: "WINS".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Ok(Variant::U32(33333)), prop);
    }
    #[test]
    fn test_button_prop_found() {
        let (mut parser_thread, player_md) = default_setup();

        let mut prop_controller_new = PropController::new(vec![], vec![], AHashMap::default());
        prop_controller_new.special_ids.buttons = Some(BUTTONS_SPECIAL_ID);
        parser_thread.prop_controller = Arc::new(prop_controller_new);

        let mut player_props = AHashMap::default();
        let flag = *BUTTONMAP.get("A").unwrap();
        player_props.insert(BUTTONS_SPECIAL_ID, Variant::U64(flag));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());

        let prop_info = PropInfo {
            id: BUTTONS_SPECIAL_ID,
            prop_type: Some(PropType::Button),
            prop_name: "A".to_string(),
            prop_friendly_name: "A".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Ok(Variant::Bool(true)), prop);
    }
    #[test]
    fn test_team_prop_found() {
        let (mut parser_thread, player_md) = default_setup();

        let mut prop_controller_new = PropController::new(vec![], vec![], AHashMap::default());
        prop_controller_new.special_ids.player_team_pointer = Some(PLAYER_TEAM_POINTER_SPECIAL_ID);
        parser_thread.prop_controller = Arc::new(prop_controller_new);

        let mut player_props = AHashMap::default();
        let mut team_props = AHashMap::default();

        player_props.insert(PLAYER_TEAM_POINTER_SPECIAL_ID, Variant::U32(3));
        team_props.insert(WANTED_PROP_ID, Variant::F32(55.6484211));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let team = Entity {
            cls_id: 0,
            entity_id: TEAM_ENTITY_ID,
            props: team_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(team.entity_id, team.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Team),
            prop_name: "someprop".to_string(),
            prop_friendly_name: "someprop".to_string(),
        };
        parser_thread.teams.team3_entid = Some(TEAM_ENTITY_ID);
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Ok(Variant::F32(55.6484211)), prop);
    }
    #[test]
    fn test_team_prop_not_found_team_incorrect_variant() {
        let (mut parser_thread, player_md) = default_setup();

        let mut prop_controller_new = PropController::new(vec![], vec![], AHashMap::default());
        prop_controller_new.special_ids.player_team_pointer = Some(PLAYER_TEAM_POINTER_SPECIAL_ID);
        parser_thread.prop_controller = Arc::new(prop_controller_new);

        let mut player_props = AHashMap::default();
        let mut team_props = AHashMap::default();

        player_props.insert(PLAYER_TEAM_POINTER_SPECIAL_ID, Variant::Bool(false));
        team_props.insert(WANTED_PROP_ID, Variant::F32(55.6484211));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let team = Entity {
            cls_id: 0,
            entity_id: TEAM_ENTITY_ID,
            props: team_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(team.entity_id, team.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Team),
            prop_name: "someprop".to_string(),
            prop_friendly_name: "someprop".to_string(),
        };
        parser_thread.teams.team3_entid = Some(TEAM_ENTITY_ID);
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::TeamNumIncorrectVariant), prop);
    }
    #[test]
    fn test_team_prop_not_found_player_pointer_not_set() {
        let (mut parser_thread, player_md) = default_setup();

        let mut player_props = AHashMap::default();
        let mut team_props = AHashMap::default();

        player_props.insert(PLAYER_TEAM_POINTER_SPECIAL_ID, Variant::U32(3));
        team_props.insert(WANTED_PROP_ID, Variant::F32(55.6484211));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let team = Entity {
            cls_id: 0,
            entity_id: TEAM_ENTITY_ID,
            props: team_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(team.entity_id, team.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Team),
            prop_name: "someprop".to_string(),
            prop_friendly_name: "someprop".to_string(),
        };
        parser_thread.teams.team3_entid = Some(TEAM_ENTITY_ID);
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::SpecialidsPlayerTeamPointerNotSet), prop);
    }

    #[test]
    fn test_team_prop_not_found_team_entity_id_not_set() {
        let (mut parser_thread, player_md) = default_setup();

        let mut player_props = AHashMap::default();
        let mut team_props = AHashMap::default();

        let mut prop_controller_new = PropController::new(vec![], vec![], AHashMap::default());
        prop_controller_new.special_ids.player_team_pointer = Some(PLAYER_TEAM_POINTER_SPECIAL_ID);
        parser_thread.prop_controller = Arc::new(prop_controller_new);

        player_props.insert(PLAYER_TEAM_POINTER_SPECIAL_ID, Variant::U32(3));
        team_props.insert(WANTED_PROP_ID, Variant::F32(55.6484211));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let team = Entity {
            cls_id: 0,
            entity_id: TEAM_ENTITY_ID,
            props: team_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(team.entity_id, team.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Team),
            prop_name: "someprop".to_string(),
            prop_friendly_name: "someprop".to_string(),
        };
        // parser_thread.teams.team3_entid = Some(TEAM_ENTITY_ID);
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::TeamEntityIdNotSet), prop);
    }
    #[test]
    fn test_team_prop_not_found_illegal_team_num() {
        let (mut parser_thread, player_md) = default_setup();

        let mut player_props = AHashMap::default();
        let mut team_props = AHashMap::default();

        let mut prop_controller_new = PropController::new(vec![], vec![], AHashMap::default());
        prop_controller_new.special_ids.player_team_pointer = Some(PLAYER_TEAM_POINTER_SPECIAL_ID);
        parser_thread.prop_controller = Arc::new(prop_controller_new);

        player_props.insert(PLAYER_TEAM_POINTER_SPECIAL_ID, Variant::U32(4));
        team_props.insert(WANTED_PROP_ID, Variant::F32(55.6484211));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        let team = Entity {
            cls_id: 0,
            entity_id: TEAM_ENTITY_ID,
            props: team_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());
        parser_thread.entities.insert(team.entity_id, team.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Team),
            prop_name: "someprop".to_string(),
            prop_friendly_name: "someprop".to_string(),
        };
        // parser_thread.teams.team3_entid = Some(TEAM_ENTITY_ID);
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Err(PropCollectionError::IllegalTeamValue), prop);
    }

    #[test]
    fn test_player_prop_found() {
        let (mut parser_thread, player_md) = default_setup();
        let mut player_props = AHashMap::default();
        player_props.insert(WANTED_PROP_ID, Variant::U8(47));

        let player = Entity {
            cls_id: 0,
            entity_id: PLAYER_ENTITY_ID,
            props: player_props,
            entity_type: EntityType::Normal,
        };
        parser_thread.entities.insert(player.entity_id, player.clone());

        let prop_info = PropInfo {
            id: WANTED_PROP_ID,
            prop_type: Some(PropType::Player),
            prop_name: "player_prop".to_string(),
            prop_friendly_name: "player_prop".to_string(),
        };
        let prop = parser_thread.find_prop(&prop_info, &player.entity_id, &player_md);
        assert_eq!(Ok(Variant::U8(47)), prop);
    }
}
