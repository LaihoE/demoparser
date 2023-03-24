use crate::parsing::read_bits::Bitreader;
use crate::parsing::read_bytes;
use crate::Parser;
use bitter::BitReader;
use bitter::LittleEndianReader;
use csgoproto::demo::CDemoPacket;
use csgoproto::demo::EDemoCommands;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::networkbasetypes::NET_Messages;
use protobuf::Message;
use std::fs;

impl Parser {
    pub fn parse_packet_ents(packet_ents: CSVCMsg_PacketEntities) {
        let n_updates = packet_ents.updated_entries();
        let entity_data = packet_ents.entity_data.unwrap();
        let mut bitreader = Bitreader::new(&entity_data);
        let mut entity_id: i32 = -1;

        for upd in 0..n_updates {
            entity_id += 1 + (bitreader.read_u_bit_var().unwrap() as i32);
            if bitreader.read_boolie().unwrap() {
                bitreader.read_boolie();
            } else if bitreader.read_boolie().unwrap() {
            } else {
            }
        }
    }
}
