use std::fmt;

/// LSB-first bit reader over a byte slice.
///
/// `bits` caches up to 64 bits starting at bit position `pos`; `bits_left` says how many of them
/// are valid. The position is a plain bit index, so skipping is O(1) (`skip`).
pub struct Bitreader<'a> {
    data: &'a [u8],
    /// Bit position (in `data`) of the lowest bit in `bits`.
    pos: usize,
    pub bits_left: u32,
    pub bits: u64,
    pub total_bits_left: u32,
}
pub fn read_varint(bytes: &[u8], ptr: &mut usize) -> Result<u32, DemoParserError> {
    let mut result: u32 = 0;
    let mut count: u8 = 0;
    loop {
        if count >= 5 {
            return Ok(result as u32);
        }
        let b = match bytes.get(*ptr) {
            Some(b) => *b as u32,
            None => return Err(DemoParserError::OutOfBytesError),
        };
        *ptr += 1;
        result |= (b & 127) << (7 * count);
        count += 1;
        if b & 0x80 == 0 {
            break;
        }
    }
    Ok(result as u32)
}

impl<'a> Bitreader<'a> {
    pub fn new(bytes: &'a [u8]) -> Bitreader<'a> {
        Bitreader {
            data: bytes,
            pos: 0,
            bits: 0,
            bits_left: 0,
            total_bits_left: 0,
        }
    }
    #[inline(always)]
    pub fn consume(&mut self, n: u32) {
        self.bits_left = self.bits_left.wrapping_sub(n);
        self.bits >>= n;
        self.pos += n as usize;
    }
    #[inline(always)]
    pub fn peek(&mut self, n: u32) -> u64 {
        self.bits & ((1 << n) - 1)
    }
    /// Reload the cache so it starts at `pos`. Leaves at least 57 valid bits unless the end of
    /// the data is near; past the end the cache is zero-padded.
    #[inline(always)]
    pub fn refill(&mut self) {
        let byte = self.pos >> 3;
        let shift = (self.pos & 7) as u32;
        if byte + 8 <= self.data.len() {
            // SAFETY: bounds checked above; read_unaligned has no alignment requirement.
            let raw = unsafe { (self.data.as_ptr().add(byte) as *const u64).read_unaligned() };
            self.bits = u64::from_le(raw) >> shift;
            self.bits_left = 64 - shift;
        } else {
            self.refill_tail(byte, shift);
        }
    }
    #[cold]
    #[inline(never)]
    fn refill_tail(&mut self, byte: usize, shift: u32) {
        let mut buf = [0u8; 8];
        if byte < self.data.len() {
            let rest = &self.data[byte..];
            buf[..rest.len()].copy_from_slice(rest);
        }
        self.bits = u64::from_le_bytes(buf) >> shift;
        let remaining = (self.data.len() * 8).saturating_sub(self.pos);
        self.bits_left = remaining.min((64 - shift) as usize) as u32;
    }
    /// Skip `n` bits in O(1).
    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        if n <= self.bits_left {
            self.consume(n);
        } else {
            self.pos += n as usize;
            self.refill();
        }
    }
    /// Current bit position in the data.
    #[inline(always)]
    pub fn bit_pos(&self) -> usize {
        self.pos
    }
    #[inline(always)]
    pub fn bits_remaining(&mut self) -> Option<usize> {
        Some((self.data.len() * 8).saturating_sub(self.pos))
    }
    /// Skip a varint (max 5 bytes, like `read_varint`) with one peek: the length is given by the
    /// first byte whose continuation bit (bit 7) is clear.
    #[inline(always)]
    pub fn skip_varint(&mut self) {
        if self.bits_left < 40 {
            self.refill();
        }
        let stops = !self.bits & 0x80_8080_8080;
        let len = if stops == 0 { 40 } else { stops.trailing_zeros() + 1 };
        self.consume(len);
    }
    #[inline(always)]
    pub fn read_nbits(&mut self, n: u32) -> Result<u32, DemoParserError> {
        if self.bits_left < n {
            self.refill();
        }
        let b = self.peek(n);
        self.consume(n);
        return Ok(b as u32);
    }
    #[inline(always)]
    pub fn read_u_bit_var(&mut self) -> Result<u32, DemoParserError> {
        let bits = self.read_nbits(6)?;
        match bits & 0b110000 {
            0b10000 => return Ok((bits & 0b1111) | (self.read_nbits(4)? << 4)),
            0b100000 => return Ok((bits & 0b1111) | (self.read_nbits(8)? << 4)),
            0b110000 => return Ok((bits & 0b1111) | (self.read_nbits(28)? << 4)),
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
                    return Err(DemoParserError::MalformedMessage);
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
        Ok(self.read_nbits(1)? != 0)
    }
    pub fn read_n_bytes(&mut self, n: usize) -> Result<Vec<u8>, DemoParserError> {
        let mut bytes = vec![0_u8; n];
        self.read_n_bytes_mut(n, &mut bytes)?;
        Ok(bytes)
    }
    pub fn read_n_bytes_mut(&mut self, n: usize, buf: &mut [u8]) -> Result<(), DemoParserError> {
        if buf.len() < n {
            return Err(DemoParserError::MalformedMessage);
        }
        let remaining = self.bits_remaining().unwrap_or(0);
        if n.checked_mul(8).map_or(true, |bits| bits > remaining) {
            return Err(DemoParserError::FailedByteRead(format!(
                "Failed to read message/command. bytes left in stream: {}, requested bytes: {}",
                remaining / 8,
                n,
            )));
        }
        let byte = self.pos >> 3;
        let shift = (self.pos & 7) as u32;
        if shift == 0 {
            buf[..n].copy_from_slice(&self.data[byte..byte + n]);
        } else {
            // Unaligned: n*8 bits remain past `pos`, so data[byte + n] exists.
            let src = &self.data[byte..byte + n + 1];
            for (i, dst) in buf[..n].iter_mut().enumerate() {
                *dst = (src[i] >> shift) | (src[i + 1] << (8 - shift));
            }
        }
        self.pos += n * 8;
        self.refill();
        Ok(())
    }
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

#[derive(Debug, PartialEq, Clone)]
pub enum DemoParserError {
    ClassMapperNotFoundFirstPass,
    FieldNoDecoder,
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
    VoiceDataWriteError(String),
    UnknownDemoCmd(i32),
    IllegalPathOp,
    VectorResizeFailure,
    ImpossibleCmd,
    UnkVoiceFormat,
    MalformedVoicePacket,
}

impl std::error::Error for DemoParserError {}

impl fmt::Display for DemoParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
