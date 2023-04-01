#[derive(Debug, Clone)]
pub enum PropData {
    Bool(bool),
    U32(u32),
    I32(i32),
    F32(f32),
    I64(i64),
    U64(u64),
    String(String),
    VecXY([f32; 2]),
    VecXYZ([f32; 3]),
    Vec(Vec<i32>),
    FloatVec(Vec<f64>),
    FloatVec32(Vec<f32>),
}
