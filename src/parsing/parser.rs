use crate::parsing::parser_settings::Parser;
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
use snap::raw::Decoder as SnapDecoder;

// The parser struct is defined in parser_settings.rs
impl Parser {
    pub fn start(&mut self) {
        // Header
        self.skip_n_bytes(16);
        // Outer loop
        loop {
            let cmd = self.read_varint();
            let tick = self.read_varint();
            let size = self.read_varint();
            self.tick = tick as i32;

            let msg_type = cmd & !64; //if cmd > 64 { cmd as u32 ^ 64 } else { cmd };
            let is_compressed = (cmd & 64) == 64;

            let bytes = match is_compressed {
                true => SnapDecoder::new()
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
                    if self.parse_entities {
                        let packet_ents: CSVCMsg_PacketEntities =
                            Message::parse_from_bytes(&bytes).unwrap();
                        self.parse_packet_ents(packet_ents);
                    }
                }
                44 => {
                    let st: CSVCMsg_CreateStringTable = Message::parse_from_bytes(&bytes).unwrap();
                    self.parse_create_stringtable(st);
                }
                45 => {
                    let st: CSVCMsg_UpdateStringTable = Message::parse_from_bytes(&bytes).unwrap();
                    self.update_string_table(st);
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
    pub fn parse_header(&self, bytes: &[u8]) {
        let _header: CDemoFileHeader = Message::parse_from_bytes(bytes).unwrap();
    }
    pub fn parse_classes(&mut self, bytes: &[u8]) {
        if self.parse_entities {
            let tables: CDemoSendTables = Message::parse_from_bytes(bytes).unwrap();
            self.parse_sendtable(tables);
        }
    }
    pub fn parse_game_event_map(event_list: CSVCMsg_GameEventList) -> HashMap<i32, Descriptor_t> {
        let mut hm: HashMap<i32, Descriptor_t> = HashMap::default();
        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        hm
    }
}
