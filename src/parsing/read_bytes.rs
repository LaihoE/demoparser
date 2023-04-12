use crate::parsing::parser_settings::Parser;

impl Parser {
    #[inline]
    pub fn skip_n_bytes(&mut self, n: u32) {
        self.ptr += n as usize;
    }
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> &[u8] {
        let s = &self.bytes[self.ptr..self.ptr + n as usize];
        self.ptr += n as usize;
        s
    }
    #[inline]
    pub fn read_varint(&mut self) -> u32 {
        let mut result: u32 = 0;
        let mut count: u8 = 0;
        let mut b: u32;

        loop {
            if count >= 5 {
                return result as u32;
            }
            b = self.bytes[self.ptr].try_into().unwrap();
            self.ptr += 1;
            result |= (b & 127) << (7 * count);
            count += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        result as u32
    }
}
