use super::variants::PropData;
use crate::parsing::parser::Parser;
use crate::parsing::parser::ProjectileRecord;
use crate::parsing::variants::PropColumn;
use crate::parsing::variants::VarVec;

#[inline(always)]
pub fn create_default(col_type: i32, mut playback_frames: usize) -> PropColumn {
    if playback_frames == 0 {
        playback_frames = 10000;
    }
    let v = match col_type {
        0 => VarVec::I32(Vec::with_capacity(playback_frames)),
        1 => VarVec::F32(Vec::with_capacity(playback_frames)),
        2 => VarVec::F32(Vec::with_capacity(playback_frames)),
        4 => VarVec::String(Vec::with_capacity(playback_frames)),
        5 => VarVec::U64(Vec::with_capacity(playback_frames)),
        10 => VarVec::I32(Vec::with_capacity(playback_frames)),
        _ => panic!("INCORRECT COL TYPE"),
    };
    PropColumn { data: v }
}
fn coord_from_cell(cell: Option<&PropData>, offset: Option<&PropData>) -> Option<f32> {
    // Coordinates of non-player entities seem to be mapped into a grid and
    // into offset inside that grid
    let cell_bits = 9;
    let max_coord = (1 << 14) as f32;
    // Both are needed for calculation
    if let Some(PropData::U32(cell)) = cell {
        if let Some(PropData::F32(offset)) = offset {
            let cell_coord = ((*cell as f32 * (1 << cell_bits) as f32) - max_coord) as f32;
            return Some(cell_coord + offset);
        }
    }
    // println!("{:?} {:?}", cell, offset);
    None
}

impl Parser {
    pub fn collect_cell_coordinate(&self, axis: &str, entity_id: &i32) -> Option<f32> {
        let offset = self.get_prop_for_ent(&("m_vec".to_owned() + axis), entity_id);
        let cell = self.get_prop_for_ent(&("m_cell".to_owned() + axis), entity_id);
        coord_from_cell(cell, offset)
    }

    pub fn find_thrower_steamid(&self, entity_id: &i32) -> Option<u64> {
        let owner_entid = match self.get_prop_for_ent("m_nOwnerId", entity_id) {
            Some(PropData::U32(prop)) => Some(*prop & 0x7FF),
            _ => None,
        };
        let steamid = match owner_entid {
            Some(entid) => match self.players.get(&(entid as i32)) {
                Some(metadata) => Some(metadata.steamid as u64),
                None => None,
            },
            None => None,
        };
        steamid
    }
    fn find_grenade_type(&self, entity_id: &i32) -> Option<String> {
        if let Some(ent) = self.entities.get(&entity_id) {
            if let Some(cls) = self.cls_by_id.get(&ent.cls_id) {
                // remove extra from name: CSmokeGrenadeProjectile --> SmokeGrenade
                // Todo maybe make names like this: smoke_grenade or just smoke
                let mut clean_name = cls.name[1..].split_at(cls.name.len() - 11).0;
                // Seems like only exception
                if clean_name == "BaseCSGrenade" {
                    clean_name = "HeGrenade"
                }
                return Some(clean_name.to_owned());
            }
        } else {
            println!("NO GRENADE FOUND?! ");
        }
        None
    }

    pub fn collect_projectiles(&mut self) {
        for projectile_entid in &self.projectiles {
            let grenade_type = self.find_grenade_type(projectile_entid);
            let steamid = self.find_thrower_steamid(projectile_entid);
            let x = self.collect_cell_coordinate("X", projectile_entid);
            let y = self.collect_cell_coordinate("Y", projectile_entid);
            let z = self.collect_cell_coordinate("Z", projectile_entid);

            self.projectile_records.push(ProjectileRecord {
                steamid: steamid,
                x: x,
                y: y,
                z: z,
                tick: Some(self.tick),
                grenade_type: grenade_type,
            });
        }
    }

    pub fn calculate_coordinates(&mut self) {
        // We create "real coordinates here by comining cell and offset"
        // Values are inserted into entities as if they were parsed normally
        // so the collect function can work the same way
        for (ent_id, _) in &self.players {
            let x = self.collect_cell_coordinate("X", ent_id);
            let y = self.collect_cell_coordinate("Y", ent_id);
            let z = self.collect_cell_coordinate("Z", ent_id);
            match self.entities.get_mut(ent_id) {
                Some(e) => {
                    if let Some(p) = x {
                        e.props.insert("X".to_owned(), PropData::F32(p));
                    }
                    if let Some(p) = y {
                        e.props.insert("Y".to_owned(), PropData::F32(p));
                    }
                    if let Some(p) = z {
                        e.props.insert("Z".to_owned(), PropData::F32(p));
                    }
                }
                None => {}
            };
        }
    }

    pub fn collect(&mut self) {
        if !self.wanted_ticks.contains(&self.tick) && self.wanted_ticks.len() != 0 {
            return;
        }
        self.collect_projectiles();
        self.calculate_coordinates();

        for (ent_id, player) in &self.players {
            for prop_name in &self.wanted_props {
                match self.entities.get(&ent_id) {
                    Some(ent) => {
                        match ent.props.get(prop_name) {
                            Some(p) => self
                                .output
                                .entry(prop_name.to_string())
                                .or_insert_with(|| PropColumn::new(&p))
                                .push(Some(p.clone())),
                            None => self
                                .output
                                .entry(prop_name.to_string())
                                .or_insert_with(|| PropColumn {
                                    data: VarVec::F32(vec![]),
                                })
                                .push(None),
                        };
                    }
                    None => self
                        .output
                        .entry(prop_name.to_string())
                        .or_insert_with(|| PropColumn {
                            data: VarVec::F32(vec![]),
                        })
                        .push(None),
                }
            }
            self.output
                .entry("tick".to_string())
                .or_insert_with(|| create_default(0, self.wanted_ticks.len()))
                .data
                .push_i32(self.tick);
            self.output
                .entry("name".to_string())
                .or_insert_with(|| create_default(4, self.wanted_ticks.len()))
                .data
                .push_string(player.name.to_string());
            self.output
                .entry("steamid".to_string())
                .or_insert_with(|| create_default(5, self.wanted_ticks.len()))
                .data
                .push_u64(player.steamid);
        }
    }
    pub fn find_val_for_entity(&self, entity_id: i32, val: &String) -> Option<PropData> {
        // used for game events
        match self.entities.get(&entity_id) {
            Some(entity) => match entity.props.get(val) {
                Some(prop) => Some(prop.clone()),
                None => None,
            },
            None => None,
        }
    }
}
