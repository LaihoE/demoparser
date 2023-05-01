use super::read_bits::Bitreader;

const QFF_ROUNDDOWN: u32 = 1 << 0;
const QFF_ROUNDUP: u32 = 1 << 1;
const QFF_ENCODE_ZERO: u32 = 1 << 2;
const QFF_ENCODE_INTEGERS: u32 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantalizedFloat {
    low: f32,
    high: f32,
    high_low_mul: f32,
    dec_mul: f32,
    offset: f32,
    bit_count: u32,
    flags: u32,
    no_scale: bool,
}

impl QuantalizedFloat {
    // More or less directly translated from here:
    // https://github.com/dotabuff/manta/blob/09a1d60ef77f68eef84b79e9ca519caf76a1f291/quantizedfloat.go
    fn validate_flags(&mut self) {
        if self.flags == 0 {
            return;
        }
        if (self.low == 0.0 && (self.flags & QFF_ROUNDDOWN) != 0)
            || (self.high == 0.0 && (self.flags & QFF_ROUNDUP) != 0)
        {
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if self.low == 0.0 && (self.flags & QFF_ENCODE_ZERO) != 0 {
            self.flags |= QFF_ROUNDDOWN;
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if self.high == 0.0 && (self.flags & QFF_ENCODE_ZERO) != 0 {
            self.flags |= QFF_ROUNDUP;
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if self.low > 0.0 || self.high < 0.0 {
            self.flags &= !QFF_ENCODE_ZERO;
        }
        if (self.flags & QFF_ENCODE_INTEGERS) != 0 {
            self.flags &= !(QFF_ROUNDUP | QFF_ROUNDDOWN | QFF_ENCODE_ZERO);
        }
        if self.flags & (QFF_ROUNDDOWN | QFF_ROUNDUP) == (QFF_ROUNDDOWN | QFF_ROUNDUP) {
            panic!("Roundup / Rounddown are mutually exclusive")
        }
    }
    fn assign_multipliers(&mut self, steps: u32) {
        self.high_low_mul = 0.0;
        let range = self.high - self.low;

        let high: u32;
        if self.bit_count == 32 {
            high = 0xFFFFFFFE;
        } else {
            high = (1 << self.bit_count) - 1;
        }

        let mut high_mul: f32;
        // Xd?
        if range.abs() <= 0.0 {
            high_mul = high as f32;
        } else {
            high_mul = (high as f32) / range;
        }

        if (high_mul * range > (high as f32))
            || (((high_mul * range) as f64) > ((high as f32) as f64))
        {
            let multipliers = vec![0.9999, 0.99, 0.9, 0.8, 0.7];
            for multiplier in multipliers {
                high_mul = (high as f32) / range * multiplier;
                if (high_mul * range > (high as f32))
                    || (((high_mul * range) as f64) > (high as f32) as f64)
                {
                    continue;
                }
                break;
            }
        }
        self.high_low_mul = high_mul;
        self.dec_mul = 1.0 / (steps - 1) as f32;

        if self.high_low_mul == 0.0 {
            panic!("FAILED TO DECODE HIGH LOW MULTP");
        }
    }
    pub fn quantize(&mut self, val: f32) -> f32 {
        if val < self.low {
            return self.low;
        } else if val > self.high {
            return self.high;
        }
        let i = ((val - self.low) * self.high_low_mul) as u32;
        self.low + (self.high - self.low) * ((i as f32) * self.dec_mul)
    }
    pub fn decode(&mut self, bitreader: &mut Bitreader) -> f32 {
        if self.flags & QFF_ROUNDDOWN != 0 && bitreader.read_boolean().unwrap() {
            return self.low;
        }
        if self.flags & QFF_ROUNDUP != 0 && bitreader.read_boolean().unwrap() {
            return self.high;
        }
        if self.flags & QFF_ENCODE_ZERO != 0 && bitreader.read_boolean().unwrap() {
            return 0.0;
        }
        let bits = bitreader.read_nbits(self.bit_count).unwrap();
        self.low + (self.high - self.low) * bits as f32 * self.dec_mul
    }
    pub fn new(
        bitcount: u32,
        flags: Option<i32>,
        low_value: Option<f32>,
        high_value: Option<f32>,
    ) -> Self {
        let mut qf = QuantalizedFloat {
            no_scale: false,
            bit_count: 0,
            dec_mul: 0.0,
            low: 0.0,
            high: 0.0,
            high_low_mul: 0.0,
            offset: 0.0,
            flags: 0,
        };

        if bitcount == 0 || bitcount >= 32 {
            qf.no_scale = true;
            qf.bit_count = 32;
            return qf;
        } else {
            qf.no_scale = false;
            qf.bit_count = bitcount;
            qf.offset = 0.0;

            if low_value.is_some() {
                qf.low = low_value.unwrap() as f32;
            } else {
                qf.low = 0.0;
            }

            if high_value.is_some() {
                qf.high = high_value.unwrap();
            } else {
                qf.high = 1.0;
            }
        }
        if flags.is_some() {
            qf.flags = flags.unwrap() as u32;
        } else {
            qf.flags = 0;
        }

        qf.validate_flags();
        let mut steps = 1 << qf.bit_count;

        if (qf.flags & QFF_ROUNDDOWN) != 0 {
            let range = qf.high - qf.low;
            qf.offset = range / (steps as f32);
            qf.high -= qf.offset;
        } else if (qf.flags & QFF_ROUNDUP) != 0 {
            let range = qf.high - qf.low;
            qf.offset = range / (steps as f32);
            qf.low += qf.offset;
        }
        if (qf.flags & QFF_ENCODE_INTEGERS) != 0 {
            let mut delta = qf.high - qf.low;
            if delta < 1.0 {
                delta = 1.0;
            }
            let delta_log2 = delta.log2().ceil();
            let range_2: u32 = 1 << delta_log2 as u32;
            let mut bit_count = qf.bit_count;
            loop {
                if (1 << bit_count) > range_2 {
                    break;
                } else {
                    bit_count += 1;
                }
            }
            if bit_count > qf.bit_count {
                qf.bit_count = bit_count;
                steps = 1 << qf.bit_count;
            }
            qf.offset = range_2 as f32 / steps as f32;
            qf.high = qf.low + ((range_2 as f32 - qf.offset) as f32);
        }

        qf.assign_multipliers(steps);

        if (qf.flags & QFF_ROUNDDOWN) != 0 {
            if qf.quantize(qf.low) == qf.low {
                qf.flags &= !QFF_ROUNDDOWN;
            }
        }
        if (qf.flags & QFF_ROUNDUP) != 0 {
            if qf.quantize(qf.high) == qf.high {
                qf.flags &= !QFF_ROUNDUP
            }
        }
        if (qf.flags & QFF_ENCODE_ZERO) != 0 {
            if qf.quantize(0.0) == 0.0 {
                qf.flags &= !QFF_ENCODE_ZERO;
            }
        }

        qf
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::q_float::*;

    #[test]
    fn test_qfloat_new() {
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.96875000000000000000000000000000,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.03125000000000000000000000000000,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.96875000000000000000000000000000,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.03125000000000000000000000000000,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.96875000000000000000000000000000,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.03125000000000000000000000000000,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(5), Some(-4.000000), Some(12.000000));
        let correct = QuantalizedFloat {
            low: -4.00000000000000000000000000000000,
            high: 11.93750000000000000000000000000000,
            high_low_mul: 16.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.06250000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1.00000000000000000000000000000000,
            high_low_mul: 255.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(-8), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.99996948242187500000000000000000,
            high_low_mul: 32768.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.00003051757812500000000000000000,
            bit_count: 15,
            flags: 4294967288,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(-8), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.99996948242187500000000000000000,
            high_low_mul: 32768.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.00003051757812500000000000000000,
            bit_count: 15,
            flags: 4294967288,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.00000000000000000000000000000000,
            high_low_mul: 1.00000000000000000000000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 1.00000000000000000000000000000000,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(1), None, Some(256.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 255.75000000000000000000000000000000,
            high_low_mul: 4.00000000000000000000000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.25000000000000000000000000000000,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 31.99987792968750000000000000000000,
            dec_mul: 0.00000381471181754022836685180664,
            offset: 0.00000000000000000000000000000000,
            bit_count: 18,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 31.99987792968750000000000000000000,
            dec_mul: 0.00000381471181754022836685180664,
            offset: 0.00000000000000000000000000000000,
            bit_count: 18,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 31.99987792968750000000000000000000,
            dec_mul: 0.00000381471181754022836685180664,
            offset: 0.00000000000000000000000000000000,
            bit_count: 18,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.96875000000000000000000000000000,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.03125000000000000000000000000000,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.96875000000000000000000000000000,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.03125000000000000000000000000000,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(15, Some(1), None, Some(1024.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1023.96875000000000000000000000000000,
            high_low_mul: 32.00000000000000000000000000000000,
            dec_mul: 0.00003051850944757461547851562500,
            offset: 0.03125000000000000000000000000000,
            bit_count: 15,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(4.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 3.98437500000000000000000000000000,
            high_low_mul: 64.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.01562500000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(32, None, None, None);
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.00000000000000000000000000000000,
            high_low_mul: 0.00000000000000000000000000000000,
            dec_mul: 0.00000000000000000000000000000000,
            offset: 0.00000000000000000000000000000000,
            bit_count: 32,
            flags: 0,
            no_scale: true,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(32, None, None, None);
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.00000000000000000000000000000000,
            high_low_mul: 0.00000000000000000000000000000000,
            dec_mul: 0.00000000000000000000000000000000,
            offset: 0.00000000000000000000000000000000,
            bit_count: 32,
            flags: 0,
            no_scale: true,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(32, None, None, None);
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.00000000000000000000000000000000,
            high_low_mul: 0.00000000000000000000000000000000,
            dec_mul: 0.00000000000000000000000000000000,
            offset: 0.00000000000000000000000000000000,
            bit_count: 32,
            flags: 0,
            no_scale: true,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(4), Some(-64.000000), Some(64.000000));
        let correct = QuantalizedFloat {
            low: -64.00000000000000000000000000000000,
            high: 64.00000000000000000000000000000000,
            high_low_mul: 7.99218750000000000000000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.00000000000000000000000000000000,
            bit_count: 10,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(4), Some(-64.000000), Some(64.000000));
        let correct = QuantalizedFloat {
            low: -64.00000000000000000000000000000000,
            high: 64.00000000000000000000000000000000,
            high_low_mul: 7.99218750000000000000000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.00000000000000000000000000000000,
            bit_count: 10,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(20, Some(4), None, Some(128.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 127.99987792968750000000000000000000,
            high_low_mul: 8192.00000000000000000000000000000000,
            dec_mul: 0.00000095367522590095177292823792,
            offset: 0.00012207031250000000000000000000,
            bit_count: 20,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(-8), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.99609375000000000000000000000000,
            high_low_mul: 256.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.00390625000000000000000000000000,
            bit_count: 8,
            flags: 4294967288,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(20, Some(1), None, Some(256.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 255.99975585937500000000000000000000,
            high_low_mul: 4096.00000000000000000000000000000000,
            dec_mul: 0.00000095367522590095177292823792,
            offset: 0.00024414062500000000000000000000,
            bit_count: 20,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(2), Some(-25.000000), Some(25.000000));
        let correct = QuantalizedFloat {
            low: -24.95117187500000000000000000000000,
            high: 25.00000000000000000000000000000000,
            high_low_mul: 20.47999954223632812500000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.04882812500000000000000000000000,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(2), None, Some(102.300003));
        let correct = QuantalizedFloat {
            low: 0.09990234673023223876953125000000,
            high: 102.30000305175781250000000000000000,
            high_low_mul: 10.00977420806884765625000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.09990234673023223876953125000000,
            bit_count: 10,
            flags: 2,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, Some(2), None, Some(102.300003));
        let correct = QuantalizedFloat {
            low: 0.09990234673023223876953125000000,
            high: 102.30000305175781250000000000000000,
            high_low_mul: 10.00977420806884765625000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.09990234673023223876953125000000,
            bit_count: 10,
            flags: 2,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(64.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 63.75000000000000000000000000000000,
            high_low_mul: 4.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.25000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(256.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 255.00000000000000000000000000000000,
            high_low_mul: 1.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 1.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(100.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 100.00000000000000000000000000000000,
            high_low_mul: 2.54999995231628417968750000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(12, Some(1), None, Some(2048.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 2047.50000000000000000000000000000000,
            high_low_mul: 2.00000000000000000000000000000000,
            dec_mul: 0.00024420025874860584735870361328,
            offset: 0.50000000000000000000000000000000,
            bit_count: 12,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(17, Some(4), Some(-4096.000000), Some(4096.000000));
        let correct = QuantalizedFloat {
            low: -4096.00000000000000000000000000000000,
            high: 4096.00000000000000000000000000000000,
            high_low_mul: 15.99987792968750000000000000000000,
            dec_mul: 0.00000762945273891091346740722656,
            offset: 0.00000000000000000000000000000000,
            bit_count: 17,
            flags: 4,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(360.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 360.00000000000000000000000000000000,
            high_low_mul: 0.70833331346511840820312500000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, None, None, Some(360.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 360.00000000000000000000000000000000,
            high_low_mul: 0.70833331346511840820312500000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.00000000000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(16, Some(1), None, Some(500.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 499.99237060546875000000000000000000,
            high_low_mul: 131.05889892578125000000000000000000,
            dec_mul: 0.00001525902189314365386962890625,
            offset: 0.00762939453125000000000000000000,
            bit_count: 16,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(18, Some(1), None, Some(1500.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 1499.99426269531250000000000000000000,
            high_low_mul: 174.76266479492187500000000000000000,
            dec_mul: 0.00000381471181754022836685180664,
            offset: 0.00572204589843750000000000000000,
            bit_count: 18,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(11, None, Some(-1.000000), Some(63.000000));
        let correct = QuantalizedFloat {
            low: -1.00000000000000000000000000000000,
            high: 63.00000000000000000000000000000000,
            high_low_mul: 31.98437500000000000000000000000000,
            dec_mul: 0.00048851978499442338943481445312,
            offset: 0.00000000000000000000000000000000,
            bit_count: 11,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(7, Some(1), None, Some(360.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 357.18750000000000000000000000000000,
            high_low_mul: 0.35555556416511535644531250000000,
            dec_mul: 0.00787401571869850158691406250000,
            offset: 2.81250000000000000000000000000000,
            bit_count: 7,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(6, Some(2), None, Some(64.000000));
        let correct = QuantalizedFloat {
            low: 1.00000000000000000000000000000000,
            high: 64.00000000000000000000000000000000,
            high_low_mul: 1.00000000000000000000000000000000,
            dec_mul: 0.01587301678955554962158203125000,
            offset: 1.00000000000000000000000000000000,
            bit_count: 6,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(1), None, Some(1.000000));
        let correct = QuantalizedFloat {
            low: 0.00000000000000000000000000000000,
            high: 0.99609375000000000000000000000000,
            high_low_mul: 256.00000000000000000000000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.00390625000000000000000000000000,
            bit_count: 8,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(10, None, Some(0.100000), Some(10.000000));
        let correct = QuantalizedFloat {
            low: 0.10000000149011611938476562500000,
            high: 10.00000000000000000000000000000000,
            high_low_mul: 103.33333587646484375000000000000000,
            dec_mul: 0.00097751710563898086547851562500,
            offset: 0.00000000000000000000000000000000,
            bit_count: 10,
            flags: 0,
            no_scale: false,
        };
        assert_eq!(qf, correct);
        let qf = QuantalizedFloat::new(8, Some(2), None, Some(60.000000));
        let correct = QuantalizedFloat {
            low: 0.23437500000000000000000000000000,
            high: 60.00000000000000000000000000000000,
            high_low_mul: 4.26624011993408203125000000000000,
            dec_mul: 0.00392156885936856269836425781250,
            offset: 0.23437500000000000000000000000000,
            bit_count: 8,
            flags: 2,
            no_scale: false,
        };
        assert_eq!(qf, correct);
    }
}
