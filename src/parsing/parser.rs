use crate::parsing::read_bits::Bitreader;
use crate::parsing::read_bytes;
use ahash::HashMap;
use bitter::BitReader;
use bitter::LittleEndianReader;
use csgoproto::demo::CDemoFileHeader;
use csgoproto::demo::CDemoPacket;
use csgoproto::demo::EDemoCommands;
use csgoproto::demo::EDemoCommands::DEM_IsCompressed;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_ClassInfo;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use csgoproto::networkbasetypes::CSVCMsg_GameEvent;
use csgoproto::networkbasetypes::NET_Messages;
use protobuf::Message;
use snap::read::FrameDecoder;
use snappy;
use std::fs;
use std::io;
use std::io::Write;

pub struct Parser {
    pub ptr: usize,
    pub bytes: Vec<u8>,
    pub ge_list: Option<HashMap<i32, Descriptor_t>>,
}

impl Parser {
    pub fn new() -> Self {
        let bytes = fs::read("/home/laiho/Documents/demos/cs2/dem.dem").unwrap();
        let mut parser = Parser {
            ptr: 0,
            bytes: bytes,
            ge_list: None,
        };
        parser
    }
    pub fn start(&mut self) {
        //self.skip_n_bytes((self.bytes.len() - 847471) as u32);

        self.skip_n_bytes(16);
        'outer: loop {
            if self.ptr > 1000000 {
                break 'outer;
            }
            let cmd = self.read_varint();
            let tick = self.read_varint();
            let size = self.read_varint();
            if tick == 1000 {
                break;
            }

            let msg_type = if cmd > 64 { cmd as u32 ^ 64 } else { cmd };

            if (size as usize + self.ptr) > self.bytes.len() {
                break;
            };

            //println!("{}")

            let is_compressed = (cmd & 64) == 64;
            if msg_type != 7 {
                //println!("{} {} {} {}", cmd, msg_type, is_compressed, size);
            }

            //println!("{}", self.bytes.len());

            let bytes = match is_compressed {
                true => {
                    let mut bytes = self.read_n_bytes(size);
                    let bl = bytes.len();
                    let b = snappy::uncompress(bytes).unwrap();
                    b
                }
                false => self.read_n_bytes(size).to_vec(),
            };

            match msg_type {
                1 => Parser::parse_header(&bytes),
                7 => self.parse_packet(&bytes),
                4 => Parser::parse_classes(&bytes),
                8 => {
                    //println!("OUTER CMD: {} {:?} {:?} {:?}", cmd, msg_type, tick, size,);
                    self.parse_packet(&bytes)
                }
                _ => {}
            }
        }
    }
    pub fn parse_classes(bytes: &[u8]) {
        let cls: CSVCMsg_SendTable = Message::parse_from_bytes(bytes).unwrap();
        //let packet: CSVCMsg_ClassInfo = Message::parse_from_bytes(bytes).unwrap();
        //println!("{:?}", cls);
    }
    pub fn parse_header(bytes: &[u8]) {
        let packet: CDemoFileHeader = Message::parse_from_bytes(bytes).unwrap();
        //println!("{:?}", packet);
    }
    pub fn parse_signon_packet(bytes: &[u8]) {
        let packet: CDemoPacket = Message::parse_from_bytes(bytes).unwrap();
        println!("{:?}", packet);
    }
    pub fn parse_packet(&mut self, bytes: &[u8]) {
        let packet: CDemoPacket = Message::parse_from_bytes(bytes).unwrap();
        let packet_data = packet.data.unwrap();
        let mut bitreader = Bitreader::new(&packet_data);

        while bitreader.reader.bits_remaining().unwrap() > 8 {
            let msg_type = bitreader.read_u_bit_var().unwrap();
            let size = bitreader.read_varint().unwrap();
            let bytes = bitreader.read_n_bytes(size as usize);

            match msg_type {
                55 => {
                    let packet_ents: CSVCMsg_PacketEntities =
                        Message::parse_from_bytes(&bytes).unwrap();
                }
                4 => {}
                207 => {
                    let ge: CSVCMsg_GameEvent = Message::parse_from_bytes(&bytes).unwrap();
                    self.parse_event(ge);
                }
                205 => {
                    let ge_list_msg: CSVCMsg_GameEventList =
                        Message::parse_from_bytes(&bytes).unwrap();
                    self.ge_list = Some(Parser::parse_game_event_map(ge_list_msg));
                }
                _ => {
                    println!("MSGTYPE: {}", msg_type);
                }
            }
            //
        }
    }
    //pub fn parse_packet_ents()
    pub fn parse_game_event_map(event_list: CSVCMsg_GameEventList) -> HashMap<i32, Descriptor_t> {
        let mut hm: HashMap<i32, Descriptor_t> = HashMap::default();

        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        hm
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
