use super::{read_bits::Bitreader, sendtables::Field};
use crate::parsing::q_float::QuantalizedFloat;

impl<'a> Bitreader<'a> {
    pub fn component_decoder(&mut self) -> bool {
        self.read_boolie().unwrap()
    }
    pub fn decode_vector(&mut self, vec_len: i32, field: Field) -> Vec<f64> {
        if vec_len == 3 && field.encoder == "normal" {
            return self.decode_normal_vec();
        };
        vec![]
    }
    pub fn decode_float(&mut self, field: Field, bitreader: &mut Bitreader) -> f32 {
        match field.encoder.as_str() {
            "coord" => return self.decode_float_coord(),
            "simtime" => return self.decode_simul_time(),
            "runetime" => return self.decode_run_time(),
            _ => {}
        }
        // TODO NOSCALERDECODER
        let mut qf = QuantalizedFloat::new(
            field.bitcount.try_into().unwrap(),
            Some(field.encode_flags),
            Some(field.low_value),
            Some(field.high_value),
        );
        qf.decode(bitreader)
    }
    pub fn decode_float_coord(&mut self) -> f32 {
        0.0
    }
    pub fn decode_simul_time(&mut self) -> f32 {
        0.0
    }
    pub fn decode_run_time(&mut self) -> f32 {
        0.0
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
        let mut v = vec![];

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
