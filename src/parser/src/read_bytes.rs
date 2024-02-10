use crate::first_pass::read_bits::DemoParserError;

#[derive(Debug)]
pub struct PacketEntitiesParser<'a> {
    pub bytes: &'a [u8],
    pub ptr: usize,
    pub max_entries: i32,
    pub updated_entries: i32,
    pub is_delta: bool,
    pub update_baseline: bool,
    pub baseline: i32,
    pub delta_from: i32,
    pub pending_full_frame: bool,
    pub active_spawngroup_handle: u32,
    pub max_spawngroup_creationsequence: u32,
    pub last_cmd_number: u32,
    pub server_tick: u32,
    pub data_start: usize,
    pub data_end: usize,
}

impl<'a> PacketEntitiesParser<'a> {
    pub fn new(bytes: &[u8]) -> PacketEntitiesParser {
        PacketEntitiesParser {
            bytes: &bytes,
            ptr: 0,
            active_spawngroup_handle: 0,
            max_entries: 0,
            update_baseline: false,
            updated_entries: 0,
            is_delta: false,
            baseline: 0,
            delta_from: 0,
            pending_full_frame: false,
            max_spawngroup_creationsequence: 0,
            last_cmd_number: 0,
            server_tick: 0,
            data_end: 0,
            data_start: 0,
        }
    }
    pub fn parse_message(&mut self) -> Result<(), DemoParserError> {
        for _ in 0..1000 {
            let varint = self.read_varint().unwrap();
            let is_done = self.read(varint);
            if is_done {
                return Ok(());
            }
        }
        Err(DemoParserError::MalformedMessage)
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
            }
            16 => {
                self.updated_entries = self.read_varint().unwrap() as i32;
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
            }
            114 => {}
            122 => {}
            _ => return true,
        }
        false
    }
}
#[derive(Debug)]
pub struct FullPacketParser<'a> {
    pub bytes: &'a [u8],
    pub ptr: usize,

    pub st_data_end: usize,
    pub st_data_start: usize,
    pub packet_data_end: usize,
    pub packet_data_start: usize,
}

impl<'a> FullPacketParser<'a> {
    pub fn new(bytes: &[u8]) -> FullPacketParser {
        FullPacketParser {
            bytes: &bytes,
            ptr: 0,

            st_data_end: 0,
            st_data_start: 0,
            packet_data_end: 0,
            packet_data_start: 0,
        }
    }
    pub fn parse_message(&mut self) -> Result<(), DemoParserError> {
        for _ in 0..1000 {
            let varint = self.read_varint().unwrap();
            let is_done = self.read(varint);
            if is_done {
                return Ok(());
            }
        }
        Err(DemoParserError::MalformedMessage)
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
            10 => {
                let buf_len = self.read_varint().unwrap() as usize;
                self.st_data_start = self.ptr;
                self.st_data_end = self.ptr + buf_len;
                self.ptr += buf_len;
            }
            18 => {
                let buf_len = self.read_varint().unwrap() as usize;
                self.packet_data_start = self.ptr + 4;
                self.packet_data_end = self.ptr + buf_len;
                self.ptr += buf_len;
                return true;
            }
            _ => panic!("unkown protobuf type in message {}", varint),
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
    Ok((result as u32, ptr + 1))
}

pub struct ProtoPacketParser<'a> {
    pub bytes: &'a [u8],
    pub start: usize,
    pub end: usize,
    pub ptr: usize,
}
impl<'a> ProtoPacketParser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        ProtoPacketParser {
            bytes: bytes,
            start: 0,
            end: 0,
            ptr: 0,
        }
    }
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
            _ => return Err(DemoParserError::MalformedMessage),
        }
    }
    pub fn read_proto_packet(&mut self) -> Result<(), DemoParserError> {
        let msg_type = self.read_varint()? as usize;
        self.read_packet(msg_type as u32)?;
        Ok(())
    }
}
