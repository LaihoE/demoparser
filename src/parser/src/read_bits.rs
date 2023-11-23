use bitter::BitReader;
use bitter::LittleEndianReader;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DemoParserError {
    OutOfBitsError,
    OutOfBytesError,
    FailedByteRead(String),
    UnknownPathOP,
    EntityNotFound,
    ClassNotFound,
    MalformedMessage,
    StringTableNotFound,
    Source1DemoError,
    DemoEndsEarly(String),
    UnknownFile,
    IncorrectMetaDataProp,
    UnknownPropName(String),
    GameEventListNotSet,
    PropTypeNotFound(String),
    GameEventUnknownId(String),
    UnknownPawnPrefix(String),
    UnknownEntityHandle(String),
    ClsIdOutOfBounds,
    UnknownGameEventVariant(String),
    FileNotFound(String),
    NoEvents,
    DecompressionFailure(String),
    NoSendTableMessage,
    UserIdNotFound,
    EventListFallbackNotFound(String),
}

impl std::error::Error for DemoParserError {}

impl fmt::Display for DemoParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
    pub fn read_nbits(&mut self, n: u32) -> Result<u32, DemoParserError> {
        match self.reader.read_bits(n) {
            Some(bits) => Ok(bits as u32 & MASKS[n as usize]),
            None => Err(DemoParserError::OutOfBitsError),
        }
    }
    #[inline(always)]
    pub fn read_u_bit_var(&mut self) -> Result<u32, DemoParserError> {
        let bits = self.read_nbits(6)?;
        match bits & 48 {
            16 => return Ok((bits & 15) | (self.read_nbits(4)? << 4)),
            32 => return Ok((bits & 15) | (self.read_nbits(8)? << 4)),
            48 => return Ok((bits & 15) | (self.read_nbits(28)? << 4)),
            _ => return Ok(bits),
        }
    }
    #[inline(always)]
    pub fn read_varint32(&mut self) -> Result<i32, DemoParserError> {
        let x = self.read_varint()? as i32;
        let mut y = x >> 1;
        if x & 1 != 0 {
            y = !y;
        }
        Ok(y as i32)
    }
    #[inline(always)]
    pub fn read_varint(&mut self) -> Result<u32, DemoParserError> {
        let mut result: u32 = 0;
        let mut count: i32 = 0;
        let mut b: u32;
        loop {
            if count >= 5 {
                return Ok(result);
            }
            b = self.read_nbits(8)?;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Ok(result)
    }
    #[inline(always)]
    pub fn read_varint_u_64(&mut self) -> Result<u64, DemoParserError> {
        let mut result: u64 = 0;
        let mut count: i32 = 0;
        let mut b: u32;
        let mut s = 0;
        loop {
            b = self.read_nbits(8)?;
            if b < 0x80 {
                if count > 9 || count == 9 && b > 1 {
                    panic!("overflow!");
                }
                return Ok(result | (b as u64) << s);
            }
            result |= ((b as u64) & 127) << s;
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
            s += 7;
        }
        Ok(result)
    }
    #[inline(always)]
    pub fn read_boolean(&mut self) -> Result<bool, DemoParserError> {
        match self.reader.read_bit() {
            Some(b) => Ok(b),
            None => Err(DemoParserError::OutOfBitsError),
        }
    }
    pub fn read_n_bytes(&mut self, n: usize) -> Result<Vec<u8>, DemoParserError> {
        let mut bytes = vec![0; n];
        match self.reader.read_bytes(&mut bytes) {
            true => Ok(bytes),
            false => Err(DemoParserError::FailedByteRead(
                format!(
                    "Failed to read message/command. bytes left in stream: {}, requested bytes: {}",
                    self.reader.bits_remaining().unwrap() / 8,
                    n,
                )
                .to_string(),
            )),
        }
    }
    pub fn read_n_bytes_mut(&mut self, n: usize, buf: &mut [u8]) -> Result<(), DemoParserError> {
        match self.reader.read_bytes(&mut buf[..n]) {
            true => Ok(()),
            false => Err(DemoParserError::FailedByteRead(
                format!(
                    "Failed to read message/command. bytes left in stream: {}, requested bytes: {}",
                    self.reader.bits_remaining().unwrap() / 8,
                    n,
                )
                .to_string(),
            )),
        }

        /*
        match self.reader.read_bytes(buf) {
            true => Ok(()),
            false => Err(DemoParserError::FailedByteRead(
                format!(
                    "Failed to read message/command. bytes left in stream: {}, requested bytes: {}",
                    self.reader.bits_remaining().unwrap() / 8,
                    n,
                )
                .to_string(),
            )),
        }
        */
    }
    #[inline(always)]
    pub fn read_ubit_var_fp(&mut self) -> Result<u32, DemoParserError> {
        if self.read_boolean()? {
            return Ok(self.read_nbits(2)?);
        }
        if self.read_boolean()? {
            return Ok(self.read_nbits(4)?);
        }
        if self.read_boolean()? {
            return Ok(self.read_nbits(10)?);
        }
        if self.read_boolean()? {
            return Ok(self.read_nbits(17)?);
        }
        return Ok(self.read_nbits(31)?);
    }
    #[inline(always)]
    pub fn read_bit_coord(&mut self) -> Result<f32, DemoParserError> {
        let mut int_val = 0;
        let mut frac_val = 0;
        let i2 = self.read_boolean()?;
        let f2 = self.read_boolean()?;
        if !i2 && !f2 {
            return Ok(0.0);
        }
        let sign = self.read_boolean()?;
        if i2 {
            int_val = self.read_nbits(14)? + 1;
        }
        if f2 {
            frac_val = self.read_nbits(5)?;
        }
        let resol: f64 = 1.0 / (1 << 5) as f64;
        let result: f32 = (int_val as f64 + (frac_val as f64 * resol) as f64) as f32;
        if sign {
            Ok(-result)
        } else {
            Ok(result)
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
