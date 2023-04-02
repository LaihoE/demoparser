use super::{
    read_bits::Bitreader,
    sendtables::{Decoder, Field},
    variants::PropData,
};
use crate::parsing::sendtables::Decoder::*;

impl<'a> Bitreader<'a> {
    pub fn decode(&mut self, decoder: &Decoder, field: &Field) -> PropData {
        match decoder {
            SignedDecoder => PropData::I32(self.read_varint32().unwrap()),
            BooleanDecoder => PropData::Bool(self.read_boolie().unwrap()),
            BaseDecoder => PropData::U32(self.read_varint().unwrap()),
            CentityHandleDecoder => PropData::U32(self.read_varint().unwrap()),
            ChangleDecoder => PropData::U32(self.read_varint().unwrap()),
            ComponentDecoder => PropData::Bool(self.read_boolie().unwrap()),
            FloatCoordDecoder => PropData::F32(self.read_bit_coord().unwrap()),
            FloatDecoder => PropData::F32(self.decode_float(field, field.bitcount).unwrap()),
            FloatRuneTimeDecoder => {
                PropData::F32(self.decode_float(field, field.bitcount).unwrap())
            }
            FloatSimulationTimeDecoder => PropData::F32(self.decode_simul_time()),
            NoscaleDecoder => PropData::F32(f32::from_le_bytes(
                self.read_nbits(32).unwrap().to_le_bytes(),
            )),
            UnsignedDecoder => PropData::U32(self.read_varint().unwrap()),
            StringDecoder => PropData::String(self.read_string().unwrap()),
            Qangle3Decoder(bits) => PropData::FloatVec32(self.decode_qangle_all_3(*bits)),
            QangleDecoder => PropData::F32(self.decode_float(field, field.bitcount).unwrap()),
            QanglePitchYawDecoder(bits) => {
                PropData::FloatVec32(self.decode_qangle_pitch_yaw(*bits))
            }
            QangleVarDecoder(bits) => PropData::FloatVec32(self.decode_qangle_variant(*bits)),
            QuantalizedFloatDecoder(qf) => PropData::F32(qf.clone().decode(self)),
            Vector2DDecoder => PropData::FloatVec(self.decode_vector(2, field, decoder)),
            Vector4DDecoder => PropData::FloatVec(self.decode_vector(4, field, decoder)),
            VectorDecoder => PropData::FloatVec(self.decode_vector(3, field, decoder)),
            VectorNormalDecoder => PropData::FloatVec(self.decode_normal_vec()),
            Unsigned64Decoder => PropData::U64(self.decode_fixed64(field).unwrap()),
            CstrongHandleDecoder => PropData::U64(self.decode_fixed64(field).unwrap()),
            Fixed64Decoder => PropData::U64(self.decode_fixed64(field).unwrap()),
            NO => PropData::U32(self.read_varint().unwrap()),
            VectorSpecialDecoder(float_type) => {
                PropData::FloatVec32(self.decode_vector_special(float_type.clone().unwrap()))
            }
        }
    }
    pub fn decode_vector_special(&mut self, float_type: Box<Decoder>) -> Vec<f32> {
        let mut v = vec![];
        let float_type = *float_type;
        match float_type {
            FloatCoordDecoder => {
                for _ in 0..3 {
                    v.push(self.decode_float_coord())
                }
                return v;
            }
            _ => {
                for _ in 0..3 {
                    v.push(self.decode_noscale().unwrap())
                }
                return v;
            }
        }
    }
    pub fn decode_uint64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.read_n_bytes(8).try_into().unwrap()))
    }
    pub fn decode_fixed64(&mut self, field: &Field) -> Option<u64> {
        match field.encoder.as_str() {
            "fixed64" => self.decode_uint64(),
            _ => self.read_varint_u_64(),
        }
    }
    pub fn decode_noscale(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(
            self.read_nbits(32).unwrap().to_le_bytes(),
        ))
    }
    pub fn read_string(&mut self) -> Option<String> {
        let mut s: Vec<u8> = Vec::new();
        loop {
            // Slow
            let c = self.read_n_bytes(1)[0];
            if c == 0 {
                break;
            }
            s.push(c);
        }
        Some(String::from_utf8_lossy(&s).to_string())
    }
    pub fn decode_vector(&mut self, vec_len: i32, field: &Field, _decoder: &Decoder) -> Vec<f64> {
        if field.var_name == "origin" || field.var_name == "dirPrimary" {
            let mut v = vec![];
            for _ in 0..3 {
                v.push(self.decode_float_coord() as f64);
            }
            return v;
        }
        if vec_len == 3 && field.encoder == "normal" {
            return self.decode_normal_vec();
        };
        let mut v = vec![];
        for _ in 0..vec_len {
            v.push(self.decode_noscale().unwrap() as f64);
        }
        v
    }
    pub fn decode_float(&mut self, field: &Field, _bits: i32) -> Option<f32> {
        match field.var_name.as_str() {
            "m_flAnimTime" => return Some(self.decode_simul_time()),
            "coord" => return Some(self.decode_float_coord()),
            "m_flSimulationTime" => return Some(self.decode_simul_time()),
            "runetime" => return Some(self.decode_run_time()),
            _ => {}
        }
        if field.bitcount >= 32 {
            return Some(self.decode_noscale().unwrap());
        }
        Some(69.0)
    }
    pub fn decode_float_coord(&mut self) -> f32 {
        self.read_bit_coord().unwrap()
    }
    pub fn decode_simul_time(&mut self) -> f32 {
        self.read_varint().unwrap() as f32 * (1.0 / 30.0)
    }
    pub fn decode_run_time(&mut self) -> f32 {
        self.read_nbits(4).unwrap() as f32
    }
    pub fn decode_normal(&mut self) -> f64 {
        let is_neg = self.read_boolie().unwrap();
        let len = self.read_nbits(11).unwrap();
        let result = len as f64 * (1.0 / ((1 << 11) as f64) - 1.0);
        match is_neg {
            true => -result,
            false => result,
        }
    }
    pub fn decode_normal_vec(&mut self) -> Vec<f64> {
        let mut v = vec![0.0; 3];
        let has_x = self.read_boolie().unwrap();
        let has_y = self.read_boolie().unwrap();
        if has_x {
            v[0] = self.decode_normal();
        }
        if has_y {
            v[1] = self.decode_normal();
        }
        let neg_z = self.read_boolie().unwrap();
        let prod_sum = v[0] * v[0] + v[1] * v[1];
        if prod_sum < 1.0 {
            v[2] = (1.0 - prod_sum).sqrt();
        } else {
            v[2] = 0.0;
        }
        if neg_z {
            v[2] = -v[2];
        }
        v
    }
    pub fn decode_qangle_pitch_yaw(&mut self, bits: i32) -> Vec<f32> {
        let mut v = vec![];
        v.push(self.read_angle(bits.try_into().unwrap()));
        v.push(self.read_angle(bits.try_into().unwrap()));
        v.push(0.0);
        v
    }
    pub fn decode_qangle_all_3(&mut self, bits: i32) -> Vec<f32> {
        let mut v = vec![];
        v.push(self.read_angle(bits.try_into().unwrap()));
        v.push(self.read_angle(bits.try_into().unwrap()));
        v.push(self.read_angle(bits.try_into().unwrap()));
        v
    }
    pub fn decode_qangle_variant(&mut self, bits: i32) -> Vec<f32> {
        let mut v = vec![];
        let has_x = self.read_boolie().unwrap();
        let has_y = self.read_boolie().unwrap();
        let has_z = self.read_boolie().unwrap();
        if has_x {
            v.push(self.read_bit_coord().unwrap());
        }
        if has_y {
            v.push(self.read_bit_coord().unwrap());
        }
        if has_z {
            v.push(self.read_bit_coord().unwrap());
        }
        v
    }
    #[allow(arithmetic_overflow)]
    pub fn read_angle(&mut self, n: usize) -> f32 {
        let x = f32::from_le_bytes(
            self.read_nbits(n.try_into().unwrap())
                .unwrap()
                .to_le_bytes(),
        );
        x * 360.0 / ((1 << n) as f32)
    }
}
