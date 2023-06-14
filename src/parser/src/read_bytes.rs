use super::read_bits::DemoParserError;
use crate::demo_searcher::DemoSearcher;
use crate::parser_settings::Parser;

impl Parser {
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], DemoParserError> {
        if self.ptr + n as usize >= self.bytes.len() {
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

            if self.ptr >= self.bytes.len() {
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

impl DemoSearcher {
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], DemoParserError> {
        if self.ptr + n as usize >= self.bytes.len() {
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
            if self.ptr >= self.bytes.len() {
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
