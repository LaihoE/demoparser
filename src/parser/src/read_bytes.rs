use super::read_bits::DemoParserError;
use crate::parser_settings::Parser;
use crate::parser_thread_settings::ParserThread;
/*
impl<'a> ParserThread<'a> {
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
 */
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
/*
impl<'a> Parser<'a> {
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
*/
#[derive(Debug)]
pub struct Reader<'a> {
    pub bytes: &'a [u8],
    pub ptr: usize,
    pub max_entries: i32,
    pub updated_entries: i32,
    pub is_delta: bool,
    pub update_baseline: bool,
    pub baseline: i32,
    pub delta_from: i32,
    pub entity_data: Vec<u8>,
    pub pending_full_frame: bool,
    pub active_spawngroup_handle: u32,
    pub max_spawngroup_creationsequence: u32,
    pub last_cmd_number: u32,
    pub server_tick: u32,
    pub serialized_entities: Vec<u8>,
    pub data_start: usize,
    pub data_end: usize,
    //pub command_queue_info: ::protobuf::MessageField<csvcmsg_packet_entities::Command_queue_info_t>,
    //pub alternate_baselines: ::std::vec::Vec<csvcmsg_packet_entities::Alternate_baseline_t>,
    //pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> Reader<'a> {
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
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], DemoParserError> {
        if self.ptr + n as usize >= self.bytes.len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        let s = &self.bytes[self.ptr..self.ptr + n as usize];
        self.ptr += n as usize;
        Ok(s)
    }
    pub fn read(&mut self, varint: u32) -> bool {
        match varint {
            8 => {
                self.max_entries = self.read_varint().unwrap() as i32;
                // self.max_entries = ::std::option::Option::Some(is.read_int32()?);
            }
            16 => {
                self.updated_entries = self.read_varint().unwrap() as i32;
                // println!("updated_entires: {:?}", self.read_varint());
                // self.updated_entries = ::std::option::Option::Some(is.read_int32()?);
            }
            24 => {
                self.is_delta = self.read_varint().unwrap() != 0;
            }
            32 => {
                self.update_baseline = self.read_varint().unwrap() != 0;
            }
            40 => {
                self.baseline = self.read_varint().unwrap() as i32;
            }
            48 => {
                self.delta_from = self.read_varint().unwrap() as i32;
            }
            58 => {
                let buf_len = self.read_varint().unwrap() as usize;
                self.data_start = self.ptr;
                self.data_end = self.ptr + buf_len;
                self.ptr += buf_len;
                //self.bytes = self.read_n_bytes(buf_len as u32).unwrap();
                //let b: &[u8] = self.read_n_bytes(buf_len as u32).unwrap();
            }
            64 => {
                self.pending_full_frame = self.read_varint().unwrap() != 0;
            }
            72 => {
                self.active_spawngroup_handle = self.read_varint().unwrap() as u32;
            }
            80 => {
                self.max_spawngroup_creationsequence = self.read_varint().unwrap() as u32;
            }
            88 => {
                self.last_cmd_number = self.read_varint().unwrap() as u32;
            }
            96 => {
                self.server_tick = self.read_varint().unwrap() as u32;
            }
            106 => {
                return true;
                // let buf_len = self.read_varint().unwrap() as usize;
                // let b = self.read_n_bytes(buf_len as u32).unwrap();
                //self.serialized_entities = ::std::option::Option::Some(is.read_bytes()?);
            }
            114 => {
                // ::protobuf::rt::read_singular_message_into_field(is, &mut self.command_queue_info)?;
            }
            122 => {
                // self.alternate_baselines.push(is.read_message()?);
            }

            _ => panic!("UNK {}", varint),
        }
        false
    }
}
pub fn read_varint(bytes: &[u8], ptr: &mut usize) -> Result<u32, DemoParserError> {
    let mut result: u32 = 0;
    let mut count: u8 = 0;
    let mut b: u32;
    loop {
        if count >= 5 {
            return Ok(result as u32);
        }
        if *ptr >= bytes.len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        b = bytes[*ptr].try_into().unwrap();
        *ptr += 1;
        result |= (b & 127) << (7 * count);
        count += 1;
        if b & 0x80 == 0 {
            break;
        }
    }
    Ok(result as u32)
}

pub fn read_varint2(bytes: &[u8], mut ptr: usize) -> Result<(u32, usize), DemoParserError> {
    let mut result: u32 = 0;
    let mut count: u8 = 0;
    let mut b: u32;

    loop {
        if count >= 5 {
            return Ok((result as u32, ptr));
        }

        if ptr >= bytes.len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        b = bytes[ptr].try_into().unwrap();
        ptr += 1;
        result |= (b & 127) << (7 * count);
        count += 1;
        if b & 0x80 == 0 {
            break;
        }
    }
    println!("{} {}", result, ptr);
    Ok((result as u32, ptr + 1))
}
pub fn read_two_varint(bytes: &[u8], mut ptr: usize) -> Result<(u32, usize), DemoParserError> {
    let mut result: u32 = 0;
    let mut count: u8 = 0;
    let mut b: u32;

    loop {
        if count >= 5 {
            return Ok((result as u32, ptr));
        }

        if ptr >= bytes.len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        b = bytes[ptr].try_into().unwrap();
        ptr += 1;
        result |= (b & 127) << (7 * count);
        count += 1;
        if b & 0x80 == 0 {
            break;
        }
    }
    println!("{} {}", result, ptr);
    Ok((result as u32, ptr + 1))
}

pub fn read_proto_packet(bytes: &[u8]) -> Result<(usize, usize), DemoParserError> {
    let (msg_type, ptr) = read_varint2(bytes, 0)?;
    println!("MSG TYPE: {}", msg_type);
    let (buf_len, _ptr) = read_varint2(bytes, ptr)?;

    println!("RANGE {:?}", &bytes[ptr..ptr + buf_len as usize]);
    Ok((ptr, (ptr + buf_len as usize)))
}

pub struct ProtoPacketParser<'a> {
    pub bytes: &'a [u8],
    pub start: usize,
    pub end: usize,
    pub ptr: usize,
}

impl<'a> ProtoPacketParser<'a> {
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
    #[inline]
    pub fn read_n_bytes(&mut self, n: u32) -> Result<&[u8], DemoParserError> {
        if self.ptr + n as usize >= self.bytes.len() {
            return Err(DemoParserError::OutOfBytesError);
        }
        let s = &self.bytes[self.ptr..self.ptr + n as usize];
        self.ptr += n as usize;
        Ok(s)
    }
    pub fn read_packet(&mut self, varint: u32) -> Result<(), DemoParserError> {
        match varint {
            26 => {
                let buf_len = self.read_varint().unwrap() as usize;
                self.start = self.ptr;
                self.end = self.ptr + buf_len;
                return Ok(());
            }
            _ => panic!("UNK {}", varint),
        }
    }
    pub fn read_proto_packet(&mut self) -> Result<(), DemoParserError> {
        let msg_type = self.read_varint()? as usize;
        self.read_packet(msg_type as u32)?;
        Ok(())
    }
}
