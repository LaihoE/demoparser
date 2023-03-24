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

const NSERIALBITS: u32 = 17;

pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
}

impl Parser {
    pub fn parse_packet_ents(&mut self, packet_ents: CSVCMsg_PacketEntities) {
        let n_updates = packet_ents.updated_entries();
        let entity_data = packet_ents.entity_data.unwrap();
        let mut bitreader = Bitreader::new(&entity_data);
        let mut entity_id: i32 = -1;

        for upd in 0..n_updates {
            entity_id += 1 + (bitreader.read_u_bit_var().unwrap() as i32);
            println!("ENTITY ID {:?}", entity_id);
            if bitreader.read_boolie().unwrap() {
                bitreader.read_boolie();
            } else if bitreader.read_boolie().unwrap() {
                let cls_id = bitreader.read_nbits(self.cls_bits).unwrap();
                let serial = bitreader.read_nbits(NSERIALBITS).unwrap();

                let entity = Entity {
                    entity_id: entity_id,
                    cls_id: cls_id,
                };
                self.entities.insert(entity_id, entity);

                println!("{:?} {:?}", cls_id, serial);
                return;
            } else {
            }
        }
    }
    pub fn parse_props() {}
}
