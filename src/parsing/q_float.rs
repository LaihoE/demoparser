use super::read_bits::Bitreader;

const qff_rounddown: u32 = (1 << 0);
const qff_roundup: u32 = (1 << 1);
const qff_encode_zero: u32 = (1 << 2);
const qff_encode_integers: u32 = (1 << 3);

#[derive(Debug, Clone)]
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
        if (self.low == 0.0 && (self.flags & qff_rounddown) != 0)
            || (self.high == 0.0 && (self.flags & qff_roundup) != 0)
        {
            self.flags = !qff_encode_zero;
        }
        if self.low == 0.0 && (self.flags & qff_encode_zero) != 0 {
            self.flags |= qff_rounddown;
            self.flags = !qff_encode_zero;
        }
        if self.high == 0.0 && (self.flags & qff_encode_zero) != 0 {
            self.flags |= qff_roundup;
            self.flags = !qff_encode_zero;
        }
        if self.low > 0.0 || self.high < 0.0 {
            self.flags = !qff_encode_zero;
        }
        if (self.flags & qff_encode_integers) != 0 {
            self.flags = !(qff_roundup | qff_rounddown | qff_encode_zero);
        }
        if self.flags & (qff_rounddown | qff_roundup) == (qff_rounddown | qff_roundup) {
            panic!("Roundup / Rounddown are mutually exclusive")
        }
    }
    fn assign_multipliers(&mut self, steps: u32) {
        self.high_low_mul = 0.0;
        let range = self.high - self.low;

        let mut high: u32 = 0;
        if self.bit_count == 32 {
            high = 0xFFFFFFFE;
        } else {
            high = (1 << self.bit_count) - 1;
        }

        let mut high_mul = 0.0;
        // Xd?
        if range.abs() <= 0.0 {
            high_mul = high as f32;
        } else {
            high_mul = high as f32 / range;
        }

        if high_mul * range > high as f32 || high_mul * range > high as f32 {
            let multipliers = vec![0.9999, 0.99, 0.9, 0.8, 0.7];
            for multiplier in multipliers {
                let high_mul = high as f32 / range * multiplier;
                if high_mul * range > high as f32 || high_mul * range > high as f32 {
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
        let i = (val - self.low) * self.high_low_mul;
        self.low + (self.high - self.low) * (i * self.dec_mul)
    }
    pub fn decode(&mut self, bitreader: &mut Bitreader) -> f32 {
        if self.flags & qff_rounddown != 0 && bitreader.read_boolie().unwrap() {
            return self.low;
        }
        if self.flags & qff_roundup != 0 && bitreader.read_boolie().unwrap() {
            return self.high;
        }
        if self.flags & qff_encode_zero != 0 && bitreader.read_boolie().unwrap() {
            return 0.0;
        }
        println!("PASSED {:?}", self.bit_count);
        if self.bit_count == 11 {
            self.bit_count = 10;
        }
        self.low
            + (self.high - self.low)
                * bitreader.read_nbits(self.bit_count).unwrap() as f32
                * self.dec_mul
    }
    pub fn new(
        mut bitcount: u32,
        flags: Option<i32>,
        low_value: Option<f32>,
        high_value: Option<f32>,
    ) -> Self {
        println!("qBITCOUNT: {}", bitcount);

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
            println!("early");
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
        let mut steps = (1 << qf.bit_count);

        let mut range = 0.0;
        if (qf.flags & qff_rounddown) != 0 {
            range = qf.high - qf.low;
            qf.offset = range / steps as f32;
            qf.high -= qf.offset;
        } else if (qf.flags & qff_roundup) != 0 {
            range = qf.high - low_value.unwrap();
            qf.offset = range / steps as f32;
            qf.low += qf.offset;
        }
        if (qf.flags & qff_encode_integers) != 0 {
            let mut delta = qf.high - qf.low;
            if delta < 1.0 {
                delta = 1.0;
            }
            let delta_log2 = delta.log2().ceil();
            let range_2: u32 = 1 << delta_log2 as u32;
            let mut bit_count = qf.bit_count;
            let pre_b = bit_count;
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
            /*
            println!("{}", qf.bit_count);

            println!(
                "{} {} {} {} {} {}",
                qf.high, qf.low, delta_log2, range_2, pre_b, qf.bit_count
            );
            */
            qf.offset = range_2 as f32 / steps as f32;
            qf.high = qf.low + (range_2 as f32 - qf.offset) as f32;
        }
        if (qf.flags & qff_rounddown) != 0 {
            if qf.quantize(qf.low) == qf.low {
                qf.flags = !qff_rounddown;
            }
        }
        if (qf.flags & qff_encode_zero) != 0 {
            if qf.quantize(0.0) == 0.0 {
                qf.flags = !qff_encode_zero;
            }
        }

        qf
    }
}
