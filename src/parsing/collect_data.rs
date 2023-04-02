use crate::parsing::parser::Parser;
use crate::parsing::variants::PropColumn;
use crate::parsing::variants::VarVec;

use super::variants::PropData;

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

impl Parser {
    pub fn collect(&mut self) {
        if !self.wanted_ticks.contains(&self.tick) && self.wanted_ticks.len() != 0 {
            return;
        }
        'outer: for (ent_id, player) in &self.players {
            for prop in &self.wanted_props {
                match self.entities.get(&ent_id) {
                    Some(ent) => {
                        for x in &ent.props {
                            /*
                            if x.0.contains("Damage") {
                                println!("{:?}", x);
                            }
                            */
                        }

                        match ent.props.get(prop) {
                            Some(p) => self
                                .output
                                .entry(prop.to_string())
                                .or_insert_with(|| PropColumn::new(&p))
                                .push(Some(p.clone())),
                            None => match self.output.get_mut(prop) {
                                Some(vec) => vec.push(None),
                                None => {
                                    // Value has not been created yet, dont insert metadata
                                    continue 'outer;
                                }
                            },
                        };
                    }
                    None => {}
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
        match self.entities.get(&entity_id) {
            Some(entity) => match entity.props.get(val) {
                Some(prop) => Some(prop.clone()),
                None => None,
            },
            None => None,
        }
    }
}
