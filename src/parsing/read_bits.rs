use bitter::BitReader;
use bitter::LittleEndianReader;

pub struct Bitreader<'a> {
    pub reader: LittleEndianReader<'a>,
}

impl<'a> Bitreader<'a> {
    pub fn new(bytes: &'a [u8]) -> Bitreader<'a> {
        let b = Bitreader {
            reader: LittleEndianReader::new(bytes),
        };
        b
    }
    #[inline(always)]
    pub fn read_nbits(&mut self, n: u32) -> Option<u32> {
        let bits = self.reader.read_bits(n)?;
        Some(bits as u32 & MASKS[n as usize])
    }
    #[inline(always)]
    pub fn read_u_bit_var(&mut self) -> Option<u32> {
        let mut ret = self.read_nbits(6)?;
        if ret & 48 == 16 {
            ret = (ret & 15) | (self.read_nbits(4)? << 4);
        } else if ret & 48 == 32 {
            ret = (ret & 15) | (self.read_nbits(8)? << 4);
        } else if ret & 48 == 48 {
            ret = (ret & 15) | (self.read_nbits(28)? << 4);
        }
        Some(ret)
    }
    pub fn read_varint32(&mut self) -> Option<i32> {
        // IDK BOUT THIS FUNC
        let x = self.read_varint().unwrap() as i32;

        let mut y = x >> 1;
        if x & 1 != 0 {
            y = !y;
        }
        Some(y as i32)
    }
    #[inline(always)]
    pub fn read_varint(&mut self) -> Option<u32> {
        let mut result: u32 = 0;
        let mut count: i32 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return result.try_into().unwrap();
            }
            b = self.read_nbits(8)?;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Some(result)
    }
    #[inline(always)]
    pub fn read_varint_u_64(&mut self) -> Option<u64> {
        let mut result: u64 = 0;
        let mut count: i32 = 0;
        let mut b: u64;
        let mut s = 0;
        loop {
            b = self.read_nbits(8)? as u64;

            if b < 0x80 {
                if count > 9 || count == 9 && b > 1 {
                    panic!("overflow!");
                }
                return Some(result | b << s);
            }
            result |= (b & 127) << s;
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
            s += 7;
        }
        Some(result)
    }
    #[inline(always)]
    pub fn read_boolie(&mut self) -> Option<bool> {
        self.reader.read_bit()
    }
    pub fn read_n_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut bytes = vec![0; n];
        self.reader.read_bytes(&mut bytes);
        bytes
    }
    pub fn read_ubit_var_fp(&mut self) -> u32 {
        if self.read_boolie().unwrap() {
            return self.read_nbits(2).unwrap();
        }
        if self.read_boolie().unwrap() {
            return self.read_nbits(4).unwrap();
        }
        if self.read_boolie().unwrap() {
            return self.read_nbits(10).unwrap();
        }
        if self.read_boolie().unwrap() {
            return self.read_nbits(17).unwrap();
        }
        return self.read_nbits(31).unwrap();
    }
    #[inline(always)]
    pub fn read_bit_coord(&mut self) -> Option<f32> {
        let mut int_val = 0;
        let mut frac_val = 0;

        let i2 = self.reader.read_bit().unwrap();
        let f2 = self.reader.read_bit().unwrap();

        if !i2 && !f2 {
            return Some(0.0);
        }
        let sign = self.reader.read_bit().unwrap();
        if i2 {
            int_val = self.read_nbits(14)? + 1;
        }
        if f2 {
            frac_val = self.read_nbits(5)?;
        }
        let resol: f64 = 1.0 / (1 << 5) as f64;
        let result: f32 = (int_val as f64 + (frac_val as f64 * resol) as f64) as f32;
        if sign {
            Some(-result)
        } else {
            Some(result)
        }
    }
}

static MASKS: [u32; 32 + 1] = [
    0,
    u32::MAX >> 31,
    u32::MAX >> 30,
    u32::MAX >> 29,
    u32::MAX >> 28,
    u32::MAX >> 27,
    u32::MAX >> 26,
    u32::MAX >> 25,
    u32::MAX >> 24,
    u32::MAX >> 23,
    u32::MAX >> 22,
    u32::MAX >> 21,
    u32::MAX >> 20,
    u32::MAX >> 19,
    u32::MAX >> 18,
    u32::MAX >> 17,
    u32::MAX >> 16,
    u32::MAX >> 15,
    u32::MAX >> 14,
    u32::MAX >> 13,
    u32::MAX >> 12,
    u32::MAX >> 11,
    u32::MAX >> 10,
    u32::MAX >> 9,
    u32::MAX >> 8,
    u32::MAX >> 7,
    u32::MAX >> 6,
    u32::MAX >> 5,
    u32::MAX >> 4,
    u32::MAX >> 3,
    u32::MAX >> 2,
    u32::MAX >> 1,
    u32::MAX,
];
