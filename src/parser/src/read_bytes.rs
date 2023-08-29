use super::read_bits::DemoParserError;
use crate::parser_settings::Parser;
use crate::parser_thread_settings::ParserThread;

impl ParserThread {
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], DemoParserError> {
        if self.ptr + n as usize >= self.bytes.get_len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        let s = &self.bytes[self.ptr..self.ptr + n as usize];
        self.ptr += n as usize;
        Ok(s)
    }
    #[inline]
    pub fn read_varint(&mut self) -> Result<u32, DemoParserError> {
        let mut result: u32 = 0;
        let mut count: u8 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return Ok(result as u32);
            }

            if self.ptr >= self.bytes.get_len() {
                return Err(DemoParserError::OutOfBytesError);
            }
            b = self.bytes[self.ptr].try_into().unwrap();
            self.ptr += 1;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Ok(result as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::read_bits::Bitreader;

    #[test]
    fn test_variant() {
        let bytes = vec![71, 149, 254, 10, 131, 15, 172, 16, 244, 142, 2, 26];
        let mut bitreader = Bitreader::new(&bytes);
        assert_eq!(bitreader.read_varint().unwrap(), 71);
        assert_eq!(bitreader.read_varint().unwrap(), 179989);
        assert_eq!(bitreader.read_varint().unwrap(), 1923);
    }
}

impl Parser {
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], DemoParserError> {
        if self.ptr + n as usize >= self.bytes.get_len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        let s = &self.bytes[self.ptr..self.ptr + n as usize];
        self.ptr += n as usize;
        Ok(s)
    }
    #[inline]
    pub fn read_varint(&mut self) -> Result<u32, DemoParserError> {
        let mut result: u32 = 0;
        let mut count: u8 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return Ok(result as u32);
            }
            if self.ptr >= self.bytes.get_len() {
                return Err(DemoParserError::OutOfBytesError);
            }
            b = self.bytes[self.ptr].try_into().unwrap();
            self.ptr += 1;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Ok(result as u32)
    }
}
