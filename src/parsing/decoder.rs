use super::{
    read_bits::{Bitreader, DemoParserError},
    sendtables::Decoder,
    variants::PropData,
};
use crate::parsing::sendtables::Decoder::*;

impl<'a> Bitreader<'a> {
    #[inline(always)]
    pub fn decode(&mut self, decoder: &Decoder) -> Result<PropData, DemoParserError> {
        match decoder {
            SignedDecoder => Ok(PropData::I32(self.read_varint32()?)),
            BooleanDecoder => Ok(PropData::Bool(self.read_boolie()?)),
            BaseDecoder => Ok(PropData::U32(self.read_varint()?)),
            CentityHandleDecoder => Ok(PropData::U32(self.read_varint()?)),
            ChangleDecoder => Ok(PropData::U32(self.read_varint()?)),
            ComponentDecoder => Ok(PropData::Bool(self.read_boolie()?)),
            FloatCoordDecoder => Ok(PropData::F32(self.read_bit_coord()?)),
            FloatSimulationTimeDecoder => Ok(PropData::F32(self.decode_simul_time()?)),
            NoscaleDecoder => Ok(PropData::F32(f32::from_bits(self.read_nbits(32)?))),
            UnsignedDecoder => Ok(PropData::U32(self.read_varint()?)),
            StringDecoder => Ok(PropData::String(self.read_string()?)),
            Qangle3Decoder => Ok(PropData::VecXYZ(self.decode_qangle_all_3()?)),
            QanglePitchYawDecoder => Ok(PropData::VecXYZ(self.decode_qangle_pitch_yaw()?)),
            QangleVarDecoder => Ok(PropData::VecXYZ(self.decode_qangle_variant()?)),
            QuantalizedFloatDecoder(qf) => Ok(PropData::F32(qf.clone().decode(self))),
            VectorNormalDecoder => Ok(PropData::FloatVec32(self.decode_normal_vec()?)),
            Unsigned64Decoder => Ok(PropData::U64(self.read_varint_u_64()?)),
            Fixed64Decoder => Ok(PropData::U64(self.decode_uint64()?)),
            VectorFloatCoordDecoder => Ok(PropData::FloatVec32(self.decode_vector_float_coord()?)),
            VectorNoscaleDecoder => Ok(PropData::FloatVec32(self.decode_vector_noscale()?)),
            NO => Ok(PropData::U32(self.read_varint()?)),
            X => Ok(PropData::U32(self.read_nbits(1)?)),
            _ => panic!("huh {:?}", decoder),
        }
    }
    pub fn decode_vector_float_coord(&mut self) -> Result<Vec<f32>, DemoParserError> {
        let mut v = vec![];
        for _ in 0..3 {
            v.push(self.decode_float_coord()?)
        }
        Ok(v)
    }
    pub fn decode_vector_noscale(&mut self) -> Result<Vec<f32>, DemoParserError> {
        let mut v = vec![];
        for _ in 0..3 {
            v.push(self.decode_noscale()?)
        }
        Ok(v)
    }
    pub fn decode_uint64(&mut self) -> Result<u64, DemoParserError> {
        let bytes = self.read_n_bytes(8)?;
        let val = u64::from_ne_bytes(bytes.try_into().unwrap());
        Ok(val)
    }
    pub fn decode_noscale(&mut self) -> Result<f32, DemoParserError> {
        Ok(f32::from_le_bytes(self.read_nbits(32)?.to_le_bytes()))
    }
    pub fn read_string(&mut self) -> Result<String, DemoParserError> {
        let mut s: Vec<u8> = vec![];
        loop {
            // Slow
            let c = self.read_n_bytes(1)?[0];
            if c == 0 {
                break;
            }
            s.push(c);
        }
        Ok(String::from_utf8_lossy(&s).to_string())
    }
    pub fn decode_float_coord(&mut self) -> Result<f32, DemoParserError> {
        Ok(self.read_bit_coord())?
    }
    pub fn decode_simul_time(&mut self) -> Result<f32, DemoParserError> {
        Ok(self.read_varint()? as f32 * (1.0 / 30.0))
    }
    pub fn decode_normal(&mut self) -> Result<f32, DemoParserError> {
        let is_neg = self.read_boolie()?;
        let len = self.read_nbits(11)?;
        let result = (len as f64 * (1.0 / ((1 << 11) as f64) - 1.0)) as f32;
        match is_neg {
            true => Ok(-result),
            false => Ok(result),
        }
    }
    pub fn decode_normal_vec(&mut self) -> Result<Vec<f32>, DemoParserError> {
        let mut v = vec![0.0; 3];
        let has_x = self.read_boolie()?;
        let has_y = self.read_boolie()?;
        if has_x {
            v[0] = self.decode_normal()?;
        }
        if has_y {
            v[1] = self.decode_normal()?;
        }
        let neg_z = self.read_boolie()?;
        let prod_sum = v[0] * v[0] + v[1] * v[1];
        if prod_sum < 1.0 {
            v[2] = (1.0 - prod_sum).sqrt() as f32;
        } else {
            v[2] = 0.0;
        }
        if neg_z {
            v[2] = -v[2];
        }
        Ok(v)
    }
    pub fn decode_qangle_pitch_yaw(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        v[0] = self.read_angle(32)?;
        v[1] = self.read_angle(32)?;
        Ok(v)
    }
    pub fn decode_qangle_all_3(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        v[0] = self.read_angle(32)?;
        v[1] = self.read_angle(32)?;
        v[2] = self.read_angle(32)?;
        Ok(v)
    }
    pub fn decode_qangle_variant(&mut self) -> Result<[f32; 3], DemoParserError> {
        let mut v = [0.0; 3];
        let has_x = self.read_boolie()?;
        let has_y = self.read_boolie()?;
        let has_z = self.read_boolie()?;
        if has_x {
            v[0] = self.read_bit_coord()?;
        }
        if has_y {
            v[1] = self.read_bit_coord()?;
        }
        if has_z {
            v[2] = self.read_bit_coord()?;
        }
        Ok(v)
    }
    pub fn read_angle(&mut self, n: usize) -> Result<f32, DemoParserError> {
        let bits = self.read_nbits(n as u32)?;
        let x = f32::from_ne_bytes(bits.to_ne_bytes());
        Ok(x * 360.0 / ((1 << n) as f32))
    }
}
