use crate::sendtables::PropInfo;
use ahash::HashMap;
use serde::ser::SerializeMap;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum Variant {
    Bool(bool),
    U32(u32),
    I32(i32),
    I16(i16),
    F32(f32),
    U64(u64),
    U8(u8),
    String(String),
    VecXY([f32; 2]),
    VecXYZ([f32; 3]),
    Vec(Vec<i32>),
    FloatVec32(Vec<f32>),
}

#[derive(Debug, Clone)]
pub enum VarVec {
    U32(Vec<Option<u32>>),
    Bool(Vec<Option<bool>>),
    U64(Vec<Option<u64>>),
    F32(Vec<Option<f32>>),
    I32(Vec<Option<i32>>),
    String(Vec<Option<String>>),
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
            _ => panic!("Tried to create propcolumns from: {:?}", item),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropColumn {
    pub data: Option<VarVec>,
    num_nones: usize,
}
impl PropColumn {
    pub fn new() -> Self {
        PropColumn {
            data: None,
            num_nones: 0,
        }
    }
    pub fn len(&self) -> usize {
        match &self.data {
            Some(VarVec::Bool(b)) => b.len(),
            Some(VarVec::I32(b)) => b.len(),
            Some(VarVec::F32(b)) => b.len(),
            Some(VarVec::String(b)) => b.len(),
            Some(VarVec::U32(b)) => b.len(),
            Some(VarVec::U64(b)) => b.len(),
            None => self.num_nones,
        }
    }
    pub fn extend_from(&mut self, other: &mut PropColumn) {
        match &mut self.data {
            Some(VarVec::Bool(v)) => match &other.data {
                Some(VarVec::Bool(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                _ => {
                    panic!("illegal 1");
                }
            },
            Some(VarVec::I32(v)) => match &other.data {
                Some(VarVec::I32(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for i in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {
                    panic!("illegal 2 {:?}", other);
                }
            },
            Some(VarVec::F32(v)) => match &other.data {
                Some(VarVec::F32(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for i in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {
                    panic!("illegal 3");
                }
            },
            Some(VarVec::String(v)) => match &other.data {
                Some(VarVec::String(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for i in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {
                    panic!("illegal 4");
                }
            },
            Some(VarVec::U32(v)) => match &other.data {
                Some(VarVec::U32(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for i in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {
                    panic!("illegal 5");
                }
            },
            Some(VarVec::U64(v)) => match &other.data {
                Some(VarVec::U64(v_other)) => {
                    v.extend_from_slice(&v_other);
                }
                None => {
                    for i in 0..other.num_nones {
                        v.push(None);
                    }
                }
                _ => {
                    panic!("illegal 6");
                }
            },
            None => match &other.data {
                Some(VarVec::Bool(inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::I32(inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::U32(inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::U64(inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::String(inner)) => {
                    self.resolve_vec_type(PropColumn::get_type(&other.data));
                    self.extend_from(other);
                }
                Some(VarVec::F32(inner)) => {
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
            _ => panic!("NONE OR > 5 TYPE FOR VEC RESOLUTION : {:?}", v_type),
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
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            Some(Variant::I32(p)) => match self {
                VarVec::I32(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            Some(Variant::String(p)) => match self {
                VarVec::String(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a ? into a {:?} column", self);
                }
            },
            Some(Variant::U32(p)) => match self {
                VarVec::U32(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            Some(Variant::U64(p)) => match self {
                VarVec::U64(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            Some(Variant::Bool(p)) => match self {
                VarVec::Bool(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            None => self.push_none(),
            _ => panic!("bad type for prop: {:?}", item),
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
        }
    }
}
#[allow(dead_code)]
pub fn filter_to_vec<Wanted>(v: impl IntoIterator<Item = impl TryInto<Wanted>>) -> Vec<Wanted> {
    v.into_iter().filter_map(|x| x.try_into().ok()).collect()
}

pub fn eventdata_type_from_variant(value: &Option<Variant>) -> i32 {
    match value {
        Some(Variant::String(_)) => 1,
        Some(Variant::F32(_)) => 2,
        Some(Variant::U32(_)) => 7,
        Some(Variant::I32(_)) => 4,
        Some(Variant::Bool(_)) => 6,
        None => 99,
        _ => panic!("Could not convert: {:?} into type", value),
    }
}

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Variant::Bool(b) => serializer.serialize_bool(*b),
            Variant::F32(f) => serializer.serialize_f32(*f),
            Variant::I16(i) => serializer.serialize_i16(*i),
            Variant::I32(i) => serializer.serialize_i32(*i),
            Variant::String(s) => serializer.serialize_str(s),
            Variant::U32(u) => serializer.serialize_u32(*u),
            Variant::U64(u) => serializer.serialize_u64(*u),
            Variant::U8(u) => serializer.serialize_u8(*u),
            _ => panic!("cant ser: {:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputSerdeHelperStruct {
    pub prop_infos: Vec<PropInfo>,
    pub inner: HashMap<u32, PropColumn>,
}

impl Serialize for OutputSerdeHelperStruct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        for prop_info in &self.prop_infos {
            if self.inner.contains_key(&prop_info.id) {
                match &self.inner[&prop_info.id].data {
                    Some(VarVec::F32(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)
                            .unwrap();
                    }
                    Some(VarVec::I32(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)
                            .unwrap();
                    }
                    Some(VarVec::String(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)
                            .unwrap();
                    }
                    Some(VarVec::U64(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)
                            .unwrap();
                    }
                    Some(VarVec::Bool(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)
                            .unwrap();
                    }
                    Some(VarVec::U32(val)) => {
                        map.serialize_entry(&prop_info.prop_friendly_name, val)
                            .unwrap();
                    }
                    None => {}
                }
            }
        }
        map.end()
    }
}
