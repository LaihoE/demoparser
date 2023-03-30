use super::{
    read_bits::Bitreader,
    sendtables::{Decoder, Field},
    variants::PropData,
};
use crate::parsing::q_float::QuantalizedFloat;

//"bool":    booleanDecoder,
//"char":    stringDecoder,
//"color32": unsignedDecoder,
//"int16":   signedDecoder,
//"int32":   signedDecoder,
//"int64":   signedDecoder,
//"int8":    signedDecoder,
//"uint16":  unsignedDecoder,
//"uint32":  unsignedDecoder,
//"uint8":   unsignedDecoder,

//"GameTime_t": noscaleDecoder,

//"CBodyComponent":       componentDecoder,
//"CGameSceneNodeHandle": unsignedDecoder,
//"Color":                unsignedDecoder,
//"CPhysicsComponent":    componentDecoder,
//"CRenderComponent":     componentDecoder,
//"CUtlString":           stringDecoder,
//"CUtlStringToken":      unsignedDecoder,
//"CUtlSymbolLarge":      stringDecoder,
use crate::parsing::sendtables::Decoder::*;
use crate::parsing::sendtables::FieldModel::*;

impl<'a> Bitreader<'a> {
    pub fn decode(&mut self, decoder: &Decoder, field: &Field) -> PropData {
        match decoder {
            SignedDecoder => PropData::I32(self.read_varint32().unwrap()),
            BooleanDecoder => PropData::Bool(self.read_boolie().unwrap()),
            BaseDecoder => PropData::U32(self.read_varint().unwrap()),
            CentityHandleDecoder => PropData::U32(self.read_varint().unwrap()),
            ChangleDecoder => PropData::U32(self.read_varint().unwrap()),
            ComponentDecoder => PropData::Bool(self.read_boolie().unwrap()),
            //CstrongHandleDecoder => self.read,
            FloatCoordDecoder => PropData::F32(self.read_bit_coord().unwrap()),
            FloatDecoder => PropData::F32(self.decode_float(field).unwrap()),
            FloatRuneTimeDecoder => PropData::F32(self.decode_float(field).unwrap()),
            FloatSimulationTimeDecoder => PropData::F32(self.decode_float(field).unwrap()),
            NoscaleDecoder => PropData::F32(self.read_nbits(32).unwrap() as f32),
            UnsignedDecoder => PropData::U32(self.read_varint().unwrap()),
            StringDecoder => PropData::String(self.read_string().unwrap()),
            Qangle3Decoder => PropData::F32(self.decode_float(field).unwrap()),
            QangleDecoder => PropData::F32(self.decode_float(field).unwrap()),
            QanglePitchYawDecoder => PropData::F32(self.decode_float(field).unwrap()),
            QangleVarDecoder => PropData::F32(self.decode_float(field).unwrap()),
            QuantalizedFloatDecoder => PropData::F32(self.decode_float(field).unwrap()),
            Vector2DDecoder => PropData::FloatVec(self.decode_vector(2, field, decoder)),
            Vector4DDecoder => PropData::FloatVec(self.decode_vector(4, field, decoder)),
            VectorDecoder => PropData::FloatVec(self.decode_vector(3, field, decoder)),
            BaseDecoder => PropData::U32(self.read_varint().unwrap()),
            VectorNormalDecoder => PropData::FloatVec(self.decode_normal_vec()),
            Unsigned64Decoder => PropData::U64(self.decode_fixed64(field).unwrap()),
            CstrongHandleDecoder => PropData::U64(self.decode_fixed64(field).unwrap()),
            Fixed64Decoder => PropData::U64(self.decode_fixed64(field).unwrap()),
        }
    }
    pub fn decode_uint64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.read_n_bytes(8).try_into().unwrap()))
    }
    pub fn decode_fixed64(&mut self, field: &Field) -> Option<u64> {
        match field.encoder.as_str() {
            "fixed64" => self.decode_fixed64(field),
            _ => self.read_varint_u_64(),
        }
    }

    pub fn decode_cursed_vec(&mut self) -> Option<f32> {
        Some(69.0)
    }

    pub fn decode_ranks(&mut self) -> Option<i32> {
        self.read_varint();
        for i in 0..32 {
            //self.read_nbits(1);
            //let res = self.read_varint();
            //println!("RES {:?}", res);
        }
        Some(69)
    }

    pub fn decode_noscale(&mut self) -> Option<f32> {
        Some(self.read_nbits(32).unwrap() as f32)
    }

    #[inline(always)]
    pub fn read_string(&mut self) -> Option<String> {
        // Maybe just some string function for this?
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
    pub fn component_decoder(&mut self) -> bool {
        self.read_boolie().unwrap()
    }
    pub fn decode_vector(&mut self, vec_len: i32, field: &Field, decoder: &Decoder) -> Vec<f64> {
        if vec_len == 3 && field.encoder == "normal" {
            return self.decode_normal_vec();
        };
        let mut v = vec![];
        for _ in 0..vec_len {
            v.push(self.decode(decoder, &field))
        }
        vec![]
    }

    pub fn decode_float(&mut self, field: &Field) -> Option<f32> {
        //println!("{:?}", field);
        match field.var_name.as_str() {
            "m_flAnimTime" => return Some(self.decode_simul_time()),
            "coord" => return Some(self.decode_float_coord()),
            "m_flSimulationTime" => return Some(self.decode_simul_time()),
            "runetime" => return Some(self.decode_run_time()),
            _ => {}
        }
        //let x = self.read_u_bit_var();
        //return Some(x.unwrap() as f32);

        if field.bitcount <= 0 || field.bitcount >= 32 {
            return Some(self.decode_noscale().unwrap());
        }

        // TODO NOSCALERDECODER
        let mut qf = QuantalizedFloat::new(
            field.bitcount.try_into().unwrap(),
            Some(field.encode_flags),
            Some(field.low_value),
            Some(field.high_value),
        );
        Some(qf.decode(self))
    }
    pub fn decode_float_coord(&mut self) -> f32 {
        self.read_bit_coord().unwrap()
    }
    pub fn decode_simul_time(&mut self) -> f32 {
        self.read_varint().unwrap() as f32 * (1.0 / 30.0)
    }
    pub fn decode_run_time(&mut self) -> f32 {
        // SUSSSSSSSSSS
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
}
