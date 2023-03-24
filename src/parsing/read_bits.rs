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
    pub fn read_boolie(&mut self) -> Option<bool> {
        self.reader.read_bit()
    }
    pub fn read_n_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut bytes = vec![0; n];
        self.reader.read_bytes(&mut bytes);
        bytes
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
