use crate::first_pass::prop_controller::PropInfo;
use crate::second_pass::collect_data::ProjectileRecord;
use crate::second_pass::parser_settings::{EconItem, PlayerEndMetaData};
use ahash::HashMap;
use itertools::Itertools;
use memmap2::Mmap;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    Bool(bool),
    U32(u32),
    I32(i32),
    F32(f32),
    U64(u64),
    String(String),
    VecXY([f32; 2]),
    VecXYZ([f32; 3]),
    // Todo change to Vec<T>
    StringVec(Vec<String>),
    U32Vec(Vec<u32>),
    U64Vec(Vec<u64>),
    Stickers(Vec<Sticker>),
    InputHistory(Vec<InputHistory>),
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sticker {
    pub name: String,
    pub wear: f32,
    pub id: u32,
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InputHistory {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub render_tick_count: i32,
    pub render_tick_fraction: f32,
    pub player_tick_count: i32,
    pub player_tick_fraction: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarVec {
    U32(Vec<Option<u32>>),
    Bool(Vec<Option<bool>>),
    U64(Vec<Option<u64>>),
    F32(Vec<Option<f32>>),
    I32(Vec<Option<i32>>),
    String(Vec<Option<String>>),
    StringVec(Vec<Vec<String>>),
    U64Vec(Vec<Vec<u64>>),
    U32Vec(Vec<Vec<u32>>),
    XYVec(Vec<Option<[f32; 2]>>),
    XYZVec(Vec<Option<[f32; 3]>>),
    Stickers(Vec<Vec<Sticker>>),
    InputHistory(Vec<Vec<InputHistory>>),
}

impl VarVec {
    pub fn new(item: &Variant) -> Self {
        match item {
            Variant::Bool(_) => VarVec::Bool(vec![]),
            Variant::I32(_) => VarVec::I32(vec![]),
            Variant::F32(_) => VarVec::F32(vec![]),
            Variant::String(_) => VarVec::String(vec![]),
            Variant::U64(_) => VarVec::U64(vec![]),
            Variant::U32(_) => VarVec::U32(vec![]),
            Variant::StringVec(_) => VarVec::StringVec(vec![]),
            Variant::U64Vec(_) => VarVec::U64Vec(vec![]),
            Variant::U32Vec(_) => VarVec::U32Vec(vec![]),
            Variant::VecXY(_) => VarVec::XYVec(vec![]),
            Variant::VecXYZ(_) => VarVec::XYZVec(vec![]),
            Variant::Stickers(_) => VarVec::Stickers(vec![]),
            Variant::InputHistory(_) => VarVec::InputHistory(vec![]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropColumn {
    pub data: Option<VarVec>,
    pub num_nones: usize,
}

impl PropColumn {
    pub fn new() -> Self {
        PropColumn { data: None, num_nones: 0 }
    }
    pub fn slice_to_new(&self, indicies: &[usize]) -> Option<PropColumn> {
        let data = match &self.data {
            Some(VarVec::Bool(b)) => VarVec::Bool(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::I32(b)) => VarVec::I32(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::F32(b)) => VarVec::F32(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::String(b)) => VarVec::String(indicies.iter().map(|x| b[*x].to_owned()).collect_vec()),
            Some(VarVec::U32(b)) => VarVec::U32(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::U64(b)) => VarVec::U64(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::StringVec(b)) => VarVec::StringVec(indicies.iter().map(|x| b[*x].to_owned()).collect_vec()),
            Some(VarVec::U64Vec(b)) => VarVec::U64Vec(indicies.iter().map(|x| b[*x].to_owned()).collect_vec()),
            Some(VarVec::U32Vec(b)) => VarVec::U32Vec(indicies.iter().map(|x| b[*x].to_owned()).collect_vec()),
            Some(VarVec::XYVec(b)) => VarVec::XYVec(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::XYZVec(b)) => VarVec::XYZVec(indicies.iter().map(|x| b[*x]).collect_vec()),
            Some(VarVec::Stickers(b)) => VarVec::Stickers(indicies.iter().map(|x| b[*x].to_owned()).collect_vec()),
            Some(VarVec::InputHistory(b)) => VarVec::InputHistory(indicies.iter().map(|x| b[*x].to_owned()).collect_vec()),
            None => {
                return Some(PropColumn {
                    data: None,
                    num_nones: indicies.len(),
                })
            }
        };
        Some(PropColumn {
            data: Some(data),
            num_nones: 0,
        })
    }
    pub fn len(&self) -> usize {
        match &self.data {
            Some(VarVec::Bool(b)) => b.len(),
            Some(VarVec::I32(b)) => b.len(),
            Some(VarVec::F32(b)) => b.len(),
            Some(VarVec::String(b)) => b.len(),
            Some(VarVec::U32(b)) => b.len(),
            Some(VarVec::U64(b)) => b.len(),
            Some(VarVec::StringVec(b)) => b.len(),
            Some(VarVec::U64Vec(b)) => b.len(),
            Some(VarVec::U32Vec(b)) => b.len(),
            Some(VarVec::XYVec(b)) => b.len(),
            Some(VarVec::XYZVec(b)) => b.len(),
            Some(VarVec::Stickers(b)) => b.len(),
            Some(VarVec::InputHistory(b)) => b.len(),
            None => self.num_nones,
        }
    }
    pub fn extend_from(&mut self, other: &mut PropColumn) {
        match &mut self.data {
            Some(VarVec::Bool(v)) => match &other.data {
                Some(VarVec::Bool(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::I32(v)) => match &other.data {
                Some(VarVec::I32(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::F32(v)) => match &other.data {
                Some(VarVec::F32(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::String(v)) => match &other.data {
                Some(VarVec::String(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::U32(v)) => match &other.data {
                Some(VarVec::U32(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::U64(v)) => match &other.data {
                Some(VarVec::U64(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::StringVec(v)) => match &other.data {
                Some(VarVec::StringVec(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(vec![]);
                    }
                }
                _ => {}
            },
            Some(VarVec::U64Vec(v)) => match &other.data {
                Some(VarVec::U64Vec(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(vec![]);
                    }
                }
                _ => {}
            },
            Some(VarVec::XYVec(v)) => match &other.data {
                Some(VarVec::XYVec(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::XYZVec(v)) => match &other.data {
                Some(VarVec::XYZVec(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {}
            },
            Some(VarVec::Stickers(v)) => match &other.data {
                Some(VarVec::Stickers(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(vec![]);
                    }
                }
                _ => {}
            },
            Some(VarVec::InputHistory(v)) => match &other.data {
                Some(VarVec::InputHistory(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(vec![]);
                    }
                }
                _ => {}
            },
            Some(VarVec::U32Vec(v)) => match &other.data {
                Some(VarVec::U32Vec(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for _ in 0..other.num_nones {
                        v.push(vec![]);
                    }
                }
                _ => {}
            },
            None => match &other.data {
                Some(VarVec::Bool(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::I32(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::U32(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::U64(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::String(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::F32(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::StringVec(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::U64Vec(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::XYVec(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::XYZVec(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::Stickers(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::U32Vec(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::InputHistory(_inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                None => {
                    self.num_nones += other.num_nones;
                }
            },
        }
    }

    pub fn get_type(v: &Option<VarVec>) -> Option<u32> {
        match v {
            Some(VarVec::Bool(_)) => Some(0),
            Some(VarVec::F32(_)) => Some(1),
            Some(VarVec::I32(_)) => Some(2),
            Some(VarVec::String(_)) => Some(3),
            Some(VarVec::U32(_)) => Some(4),
            Some(VarVec::U64(_)) => Some(5),
            Some(VarVec::StringVec(_)) => Some(6),
            Some(VarVec::U64Vec(_)) => Some(7),
            Some(VarVec::XYVec(_)) => Some(8),
            Some(VarVec::XYZVec(_)) => Some(9),
            Some(VarVec::Stickers(_)) => Some(10),
            Some(VarVec::U32Vec(_)) => Some(11),
            Some(VarVec::InputHistory(_)) => Some(12),

            None => None,
        }
    }
    pub fn resolve_vec_type(&mut self, v_type: Option<u32>) {
        if self.data.is_some() {
            return;
        }
        match v_type {
            Some(0) => self.data = Some(VarVec::Bool(vec![])),
            Some(1) => self.data = Some(VarVec::F32(vec![])),
            Some(2) => self.data = Some(VarVec::I32(vec![])),
            Some(3) => self.data = Some(VarVec::String(vec![])),
            Some(4) => self.data = Some(VarVec::U32(vec![])),
            Some(5) => self.data = Some(VarVec::U64(vec![])),
            Some(6) => self.data = Some(VarVec::StringVec(vec![])),
            Some(7) => self.data = Some(VarVec::U64Vec(vec![])),
            Some(8) => self.data = Some(VarVec::XYVec(vec![])),
            Some(9) => self.data = Some(VarVec::XYZVec(vec![])),
            Some(10) => self.data = Some(VarVec::Stickers(vec![])),
            Some(11) => self.data = Some(VarVec::U32Vec(vec![])),
            Some(12) => self.data = Some(VarVec::InputHistory(vec![])),
            _ => {}
        }
        for _ in 0..self.num_nones {
            self.push(None);
        }
    }
    #[inline(always)]
    pub fn push(&mut self, item: Option<Variant>) {
        match &self.data {
            Some(_) => {}
            None => match &item {
                None => self.num_nones += 1,
                Some(p) => {
                    let mut var_vec = VarVec::new(&p);
                    for _ in 0..self.num_nones {
                        var_vec.push_variant(None);
                    }
                    self.num_nones = 0;
                    self.data = Some(var_vec);
                }
            },
        };
        if let Some(v) = &mut self.data {
            v.push_variant(item.clone());
        }
    }
}

impl VarVec {
    #[inline(always)]
    pub fn push_variant(&mut self, item: Option<Variant>) {
        match item {
            Some(Variant::F32(p)) => match self {
                VarVec::F32(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::I32(p)) => match self {
                VarVec::I32(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::String(p)) => match self {
                VarVec::String(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::U32(p)) => match self {
                VarVec::U32(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::U64(p)) => match self {
                VarVec::U64(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::Bool(p)) => match self {
                VarVec::Bool(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::StringVec(p)) => match self {
                VarVec::StringVec(f) => f.push(p),
                _ => {}
            },
            Some(Variant::U64Vec(p)) => match self {
                VarVec::U64Vec(f) => f.push(p),
                _ => {}
            },
            Some(Variant::U32Vec(p)) => match self {
                VarVec::U32Vec(f) => f.push(p),
                _ => {}
            },
            Some(Variant::VecXY(p)) => match self {
                VarVec::XYVec(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::VecXYZ(p)) => match self {
                VarVec::XYZVec(f) => f.push(Some(p)),
                _ => {}
            },
            Some(Variant::Stickers(p)) => match self {
                VarVec::Stickers(f) => f.push(p),
                _ => {}
            },
            Some(Variant::InputHistory(p)) => match self {
                VarVec::InputHistory(f) => f.push(p),
                _ => {}
            },
            None => self.push_none(),
            _ => {}
        }
    }
    pub fn push_none(&mut self) {
        match self {
            VarVec::I32(f) => f.push(None),
            VarVec::F32(f) => f.push(None),
            VarVec::String(f) => f.push(None),
            VarVec::U32(f) => f.push(None),
            VarVec::U64(f) => f.push(None),
            VarVec::Bool(f) => f.push(None),
            VarVec::StringVec(f) => f.push(vec![]),
            VarVec::U64Vec(f) => f.push(vec![]),
            VarVec::XYVec(f) => f.push(None),
            VarVec::XYZVec(f) => f.push(None),
            VarVec::U32Vec(f) => f.push(vec![]),
            VarVec::Stickers(f) => f.push(vec![]),
            VarVec::InputHistory(f) => f.push(vec![]),
        }
    }
}
#[allow(dead_code)]
pub fn filter_to_vec<Wanted>(v: impl IntoIterator<Item = impl TryInto<Wanted>>) -> Vec<Wanted> {
    v.into_iter().filter_map(|x| x.try_into().ok()).collect()
}

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Variant::Bool(b) => serializer.serialize_bool(*b),
            Variant::F32(f) => serializer.serialize_f32(*f),
            Variant::I32(i) => serializer.serialize_i32(*i),
            Variant::String(s) => serializer.serialize_str(s),
            Variant::U32(u) => serializer.serialize_u32(*u),
            Variant::U64(u) => serializer.serialize_str(&u.to_string()),
            Variant::StringVec(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(item)?;
                }
                s.end()
            }
            Variant::VecXY(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(item)?;
                }
                s.end()
            }
            Variant::VecXYZ(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(item)?;
                }
                s.end()
            }
            Variant::U32Vec(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(item)?;
                }
                s.end()
            }
            Variant::U64Vec(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(&item.to_string())?;
                }
                s.end()
            }
            Variant::Stickers(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(&item)?;
                }
                s.end()
            }
            Variant::InputHistory(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    s.serialize_element(&item)?;
                }
                s.end()
            }
        }
    }
}

impl Serialize for PlayerEndMetaData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PlayerEndMetaData", 3)?;
        state.serialize_field("name", &self.name)?;
        let steamid = match self.steamid {
            Some(u) => Some(u.to_string()),
            None => None,
        };
        state.serialize_field("steamid", &steamid)?;
        state.serialize_field("team_number", &self.team_number)?;
        state.end()
    }
}
impl Serialize for EconItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("EconItem", 14)?;
        let steamid = match self.steamid {
            Some(u) => Some(u.to_string()),
            None => None,
        };
        state.serialize_field("steamid", &steamid)?;
        state.serialize_field("account_id", &self.account_id)?;
        state.serialize_field("custom_name", &self.custom_name)?;
        state.serialize_field("def_index", &self.def_index)?;
        state.serialize_field("dropreason", &self.dropreason)?;
        state.serialize_field("item_id", &self.item_id)?;
        state.serialize_field("inventory", &self.inventory)?;
        state.serialize_field("item_id", &self.item_id)?;
        state.serialize_field("paint_index", &self.paint_index)?;
        state.serialize_field("paint_seed", &self.paint_seed)?;
        state.serialize_field("paint_wear", &self.paint_wear)?;
        state.serialize_field("quality", &self.quality)?;
        state.serialize_field("quest_id", &self.quest_id)?;
        state.serialize_field("rarity", &self.rarity)?;
        state.serialize_field("item_name", &self.item_name)?;
        state.serialize_field("skin_name", &self.skin_name)?;
        state.end()
    }
}
impl Serialize for ProjectileRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ProjectileRecord", 7)?;
        let steamid = match self.steamid {
            Some(u) => Some(u.to_string()),
            None => None,
        };
        state.serialize_field("steamid", &steamid)?;
        state.serialize_field("grenade_type", &self.grenade_type)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("tick", &self.tick)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("z", &self.z)?;
        state.serialize_field("entity_id", &self.entity_id)?;
        state.end()
    }
}
#[derive(Debug)]
pub enum BytesVariant {
    Mmap(Mmap),
    Vec(Vec<u8>),
}

impl<Idx> std::ops::Index<Idx> for BytesVariant
where
    Idx: std::slice::SliceIndex<[u8]>,
{
    type Output = Idx::Output;
    #[inline(always)]
    fn index(&self, i: Idx) -> &Self::Output {
        match self {
            Self::Mmap(m) => &m[i],
            Self::Vec(v) => &v[i],
        }
    }
}
impl BytesVariant {
    pub fn get_len(&self) -> usize {
        match self {
            Self::Mmap(m) => m.len(),
            Self::Vec(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputSerdeHelperStruct {
    pub prop_infos: Vec<PropInfo>,
    pub inner: HashMap<u32, PropColumn>,
}
pub fn soa_to_aos(soa: OutputSerdeHelperStruct) -> Vec<std::collections::HashMap<String, Option<Variant>>> {
    let mut total_rows = 0;
    for (_, v) in &soa.inner {
        total_rows = v.len();
    }
    let mut v = Vec::with_capacity(total_rows);
    for idx in 0..total_rows {
        let mut hm: std::collections::HashMap<String, Option<Variant>> = std::collections::HashMap::with_capacity(soa.prop_infos.len());
        for prop_info in &soa.prop_infos {
            if soa.inner.contains_key(&prop_info.id) {
                match &soa.inner[&prop_info.id].data {
                    None => continue,
                    Some(VarVec::F32(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::F32(*f))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::I32(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::I32(*f))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::String(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::String(f.to_string()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::U64(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::String(f.to_string()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::Bool(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::Bool(*f))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::U32(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::U32(*f))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::StringVec(val)) => match val.get(idx) {
                        Some(f) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::StringVec(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::U64Vec(val)) => match val.get(idx) {
                        Some(f) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::U64Vec(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::U32Vec(val)) => match val.get(idx) {
                        Some(f) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::U32Vec(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::XYVec(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::VecXY(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::XYZVec(val)) => match val.get(idx) {
                        Some(Some(f)) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::VecXYZ(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::Stickers(val)) => match val.get(idx) {
                        Some(f) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::Stickers(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                    Some(VarVec::InputHistory(val)) => match val.get(idx) {
                        Some(f) => hm.insert(prop_info.prop_friendly_name.clone(), Some(Variant::InputHistory(f.clone()))),
                        _ => hm.insert(prop_info.prop_friendly_name.clone(), None),
                    },
                };
            }
        }
        v.push(hm);
    }
    v
}

impl Serialize for OutputSerdeHelperStruct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.prop_infos.len()))?;

        for prop_info in &self.prop_infos {
            if self.inner.contains_key(&prop_info.id) {
                match &self.inner[&prop_info.id].data {
                    None => {}
                    Some(VarVec::F32(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::I32(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::String(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::U64(val)) => {
                        let as_str: Vec<Option<String>> = val
                            .iter()
                            .map(|x| match x {
                                Some(u) => Some(u.to_string()),
                                None => None,
                            })
                            .collect_vec();
                        map.serialize_entry(&prop_info.prop_friendly_name, &as_str)?;
                    }
                    Some(VarVec::Bool(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::U32(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::StringVec(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::U32Vec(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::XYVec(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::XYZVec(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::Stickers(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::InputHistory(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)?;
                    }
                    Some(VarVec::U64Vec(val)) => {
                        let string_sid = val
                            .iter()
                            .map(|v| {
                                let as_sid = v.iter().map(|s| s.to_string()).collect_vec();
                                as_sid
                            })
                            .collect_vec();
                        map.serialize_entry(&prop_info.prop_friendly_name, &string_sid)?;
                    }
                }
            }
        }
        map.end()
    }
}
