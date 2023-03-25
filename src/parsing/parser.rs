use super::class::Class;
use super::sendtables::Serializer;
use crate::parsing::entities::Entity;
use crate::parsing::read_bits::Bitreader;
use ahash::HashMap;
use bitter::BitReader;
use csgoproto::demo::CDemoFileHeader;
use csgoproto::demo::CDemoPacket;
use csgoproto::demo::CDemoSendTables;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use csgoproto::networkbasetypes::*;
use protobuf::Message;
use snappy;
use std::fs;

pub struct Parser {
    pub ptr: usize,
    pub bytes: Vec<u8>,
    pub ge_list: Option<HashMap<i32, Descriptor_t>>,
    pub serializers: HashMap<String, Serializer>,
    pub cls_by_id: HashMap<i32, Class>,
    pub cls_by_name: HashMap<String, Class>,
    pub cls_bits: u32,
    pub entities: HashMap<i32, Entity>,
}

impl Parser {
    pub fn new(path: &str) -> Self {
        let bytes = fs::read(path).unwrap();
        Parser {
            ptr: 0,
            bytes: bytes,
            ge_list: None,
            serializers: HashMap::default(),
            cls_by_id: HashMap::default(),
            cls_by_name: HashMap::default(),
            entities: HashMap::default(),
            cls_bits: 0,
        }
    }
    pub fn start(&mut self) {
        // Header
        self.skip_n_bytes(16);
        // Outer loop
        loop {
            let cmd = self.read_varint();
            let tick = self.read_varint();
            let size = self.read_varint();
            println!("Tick: {}", tick);
            // Think my demo is shit
            if tick == 1000 {
                break;
            }

            let msg_type = if cmd > 64 { cmd as u32 ^ 64 } else { cmd };
            let is_compressed = (cmd & 64) == 64;

            let bytes = match is_compressed {
                true => {
                    let bytes = self.read_n_bytes(size);
                    snappy::uncompress(bytes).unwrap()
                }
                false => self.read_n_bytes(size).to_vec(),
            };

            match msg_type {
                1 => self.parse_header(&bytes),
                4 => self.parse_classes(&bytes),
                5 => self.parse_class_info(&bytes),
                7 => self.parse_packet(&bytes),
                8 => self.parse_packet(&bytes),
                _ => {}
            }
        }
    }
    pub fn parse_classes(&mut self, bytes: &[u8]) {
        let tables: CDemoSendTables = Message::parse_from_bytes(bytes).unwrap();
        self.parse_sendtable(tables);
    }
    pub fn parse_header(&self, bytes: &[u8]) {
        let header: CDemoFileHeader = Message::parse_from_bytes(bytes).unwrap();
    }
    pub fn parse_packet(&mut self, bytes: &[u8]) {
        let packet: CDemoPacket = Message::parse_from_bytes(bytes).unwrap();
        let packet_data = packet.data.unwrap();
        let mut bitreader = Bitreader::new(&packet_data);

        // Inner loop
        while bitreader.reader.bits_remaining().unwrap() > 8 {
            let msg_type = bitreader.read_u_bit_var().unwrap();
            let size = bitreader.read_varint().unwrap();
            let bytes = bitreader.read_n_bytes(size as usize);

            match msg_type {
                55 => {
                    let packet_ents: CSVCMsg_PacketEntities =
                        Message::parse_from_bytes(&bytes).unwrap();
                    self.parse_packet_ents(packet_ents);
                }
                40 => {
                    let server_info: CSVCMsg_ServerInfo =
                        Message::parse_from_bytes(&bytes).unwrap();
                    self.parse_server_info(server_info);
                }

                207 => {
                    let ge: CSVCMsg_GameEvent = Message::parse_from_bytes(&bytes).unwrap();
                    //self.parse_event(ge);
                }
                205 => {
                    let ge_list_msg: CSVCMsg_GameEventList =
                        Message::parse_from_bytes(&bytes).unwrap();
                    self.ge_list = Some(Parser::parse_game_event_map(ge_list_msg));
                }
                _ => {
                    //println!("MSGTYPE: {}", msg_type);
                }
            }
        }
    }
    pub fn parse_server_info(&mut self, server_info: CSVCMsg_ServerInfo) {
        let class_count = server_info.max_classes();
        self.cls_bits = (class_count as f32 + 1.).log2().ceil() as u32;
    }

    pub fn parse_game_event_map(event_list: CSVCMsg_GameEventList) -> HashMap<i32, Descriptor_t> {
        let mut hm: HashMap<i32, Descriptor_t> = HashMap::default();
        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        hm
    }
}
