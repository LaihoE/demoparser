use super::class::Class;
use super::entities::HuffmanNode;
use super::game_events::GameEvent;
use super::sendtables::Serializer;
use super::variants::PropColumn;
use super::variants::VarVec;
use crate::parsing::entities::generate_huffman_tree;
use crate::parsing::entities::Entity;
use crate::parsing::entities::PlayerMetaData;
use crate::parsing::read_bits::Bitreader;
use ahash::HashMap;
use ahash::HashSet;
use bitter::BitReader;
use csgoproto::demo::CDemoFileHeader;
use csgoproto::demo::CDemoPacket;
use csgoproto::demo::CDemoSendTables;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use csgoproto::networkbasetypes::*;
use protobuf::Message;
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
    pub tick: i32,
    pub huffman_tree: HuffmanNode,

    pub wanted_ticks: HashSet<i32>,
    pub wanted_props: Vec<String>,
    pub wanted_event: Option<String>,
    pub players: HashMap<i32, PlayerMetaData>,

    pub output: HashMap<String, PropColumn>,
    pub game_events: Vec<GameEvent>,
    pub parse_entities: bool,
}
pub struct PacketMsg {
    msg_type: i32,
    data: Vec<u8>,
}

impl Parser {
    pub fn new(
        path: &str,
        wanted_props: Vec<String>,
        wanted_ticks: Vec<i32>,
        wanted_event: Option<String>,
        parse_ents: bool,
    ) -> Self {
        let bytes = fs::read(path).unwrap();
        let tree = generate_huffman_tree().unwrap();
        Parser {
            ptr: 0,
            bytes: bytes,
            ge_list: None,
            serializers: HashMap::default(),
            cls_by_id: HashMap::default(),
            cls_by_name: HashMap::default(),
            entities: HashMap::default(),
            cls_bits: 0,
            tick: -99999,
            huffman_tree: tree,
            wanted_props: wanted_props,
            players: HashMap::default(),
            output: HashMap::default(),
            wanted_ticks: HashSet::from_iter(wanted_ticks),
            game_events: vec![],
            wanted_event: wanted_event,
            parse_entities: parse_ents,
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
            self.tick = tick as i32;
            if tick > 20000 && tick < 494967295 {
                //break;
            }
            //println!("{:?}", self.tick);
            let msg_type = if cmd > 64 { cmd as u32 ^ 64 } else { cmd };
            let is_compressed = (cmd & 64) == 64;
            use snap::raw::Decoder;

            let bytes = match is_compressed {
                true => Decoder::new()
                    .decompress_vec(self.read_n_bytes(size))
                    .unwrap(),
                false => self.read_n_bytes(size).to_vec(),
            };

            match msg_type {
                // 0 = End of demo
                0 => break,
                1 => self.parse_header(&bytes),
                4 => self.parse_classes(&bytes),
                5 => self.parse_class_info(&bytes),
                7 => self.parse_packet(&bytes),
                8 => self.parse_packet(&bytes),
                _ => {}
            }
            self.collect();
        }
        // Collects wanted data from entities
    }
    pub fn parse_classes(&mut self, bytes: &[u8]) {
        if self.parse_entities {
            let tables: CDemoSendTables = Message::parse_from_bytes(bytes).unwrap();
            self.parse_sendtable(tables);
        }
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
                44 => {
                    let st: CSVCMsg_CreateStringTable = Message::parse_from_bytes(&bytes).unwrap();
                    self.parse_create_stringtable(st);
                }
                55 => {
                    if self.parse_entities {
                        let packet_ents: CSVCMsg_PacketEntities =
                            Message::parse_from_bytes(&bytes).unwrap();
                        self.parse_packet_ents(packet_ents);
                    }
                }
                40 => {
                    let server_info: CSVCMsg_ServerInfo =
                        Message::parse_from_bytes(&bytes).unwrap();
                    self.parse_server_info(server_info);
                }
                207 => {
                    if self.wanted_event.is_some() {
                        let ge: CSVCMsg_GameEvent = Message::parse_from_bytes(&bytes).unwrap();
                        self.parse_event(ge);
                    }
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
