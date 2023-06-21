use crate::game_events::GameEvent;
use crate::netmessage_types;
use crate::netmessage_types::netmessage_type_from_int;
use crate::parser_settings::Parser;
use crate::parser_thread_settings::ChatMessageRecord;
use crate::parser_thread_settings::EconItem;
use crate::parser_thread_settings::ParserThread;
use crate::parser_thread_settings::PlayerEndMetaData;
use crate::parser_threads::demo_cmd_type_from_int;
use crate::read_bits::Bitreader;
use crate::variants::PropColumn;
use crate::variants::VarVec;
use crate::{other_netmessages::Class, read_bits::DemoParserError};
use ahash::AHashMap;
use bitter::BitReader;
use csgoproto::demo::CDemoClassInfo;
use csgoproto::demo::CDemoFileHeader;
use csgoproto::demo::CDemoPacket;
use csgoproto::demo::CDemoSendTables;
use csgoproto::demo::EDemoCommands::*;
use netmessage_types::NetmessageType::*;
use protobuf::Message;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use snap::raw::Decoder as SnapDecoder;
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub struct DemoOutput {
    pub df: AHashMap<u32, PropColumn>,
    pub game_events: Vec<GameEvent>,
    pub skins: Vec<EconItem>,
    pub item_drops: Vec<EconItem>,
    pub chat_messages: Vec<ChatMessageRecord>,
    pub convars: AHashMap<String, String>,
    pub header: AHashMap<String, String>,
    pub player_md: Vec<PlayerEndMetaData>,
    pub game_events_counter: AHashMap<String, i32>,
}

impl Parser {
    pub fn parse_demo(&mut self) -> Result<DemoOutput, DemoParserError> {
        self.ptr = 16;
        self.fullpacket_offsets.push(16);

        let mut sendtable: Option<CDemoSendTables> = None;

        loop {
            let before = self.ptr;

            let cmd = self.read_varint()?;
            let tick = self.read_varint()?;
            let size = self.read_varint()?;

            self.tick = tick as i32;
            if self.tick > 180000 {
                break;
            }
            let msg_type = cmd & !64;
            let is_compressed = (cmd & 64) == 64;

            let bytes = match is_compressed {
                true => SnapDecoder::new()
                    .decompress_vec(self.read_n_bytes(size)?)
                    .unwrap(),
                false => self.read_n_bytes(size)?.to_vec(),
            };

            let ok: Result<(), DemoParserError> = match demo_cmd_type_from_int(msg_type as i32)
                .unwrap()
            {
                DEM_SendTables => {
                    let before = Instant::now();
                    sendtable = Some(Message::parse_from_bytes(&bytes).unwrap());
                    // self.parse_sendtable(&bytes)
                    Ok(())
                }
                DEM_FileHeader => self.parse_header(&bytes),
                DEM_ClassInfo => {
                    let my_s = sendtable.clone();
                    let my_b = bytes.clone();
                    let handle = thread::spawn(move || self.parse_class_info(&my_b, my_s.unwrap()));
                    Ok(())
                }
                DEM_SignonPacket => self.parse_packet(&bytes),
                DEM_Stop => break,
                DEM_FullPacket => {
                    self.fullpacket_offsets.push(before);
                    Ok(())
                }
                _ => Ok(()),
            };
            ok?;
        }
        let outputs: Vec<ParserThread> = self
            .fullpacket_offsets
            .par_iter()
            .map(|offset| {
                let mut parser = ParserThread::new(self.settings.clone(), &self.cls_by_id).unwrap();
                if offset == &16 {
                    parser.fullpackets_parsed = 1;
                }
                parser.ptr = *offset;
                parser.cls_by_id = &self.cls_by_id;
                parser.prop_name_to_path = self.prop_name_to_path.clone();
                parser.prop_infos = self.prop_infos.clone();
                parser.controller_ids = self.controller_ids.clone();
                parser.parse_entities = true;
                parser.qf_map = self.qf_mapper.clone();
                parser.ge_list = self.ge_list.clone();
                parser.wanted_event = self.wanted_event.clone();
                parser.baselines = self.baselines.clone();
                parser.start().unwrap();
                parser
            })
            .collect();
        let mut p = outputs.iter().map(|x| x.output.clone()).collect();
        let evs: Vec<GameEvent> = outputs.iter().flat_map(|p| p.game_events.clone()).collect();
        let df = self.combine_dfs(&mut p);
        let chat_msgs: Vec<ChatMessageRecord> = outputs
            .iter()
            .flat_map(|p| p.chat_messages.clone())
            .collect();
        let item_drops: Vec<EconItem> = outputs.iter().flat_map(|p| p.item_drops.clone()).collect();
        let skins: Vec<EconItem> = outputs.iter().flat_map(|p| p.skins.clone()).collect();

        let out = DemoOutput {
            df: df,
            game_events: evs,
            chat_messages: chat_msgs,
            item_drops: item_drops,
            skins: skins,
            convars: self.convars.clone(),
            header: self.header.clone(),
            player_md: self.player_md.clone(),
            // fix
            game_events_counter: AHashMap::default(),
        };
        Ok(out)
    }

    fn combine_dfs(&self, v: &mut Vec<AHashMap<u32, PropColumn>>) -> AHashMap<u32, PropColumn> {
        let mut big: AHashMap<u32, PropColumn> = AHashMap::default();
        for part_df in v {
            for (k, v) in part_df {
                big.entry(*k).or_insert(v.clone()).extend_from(v)
            }
        }
        big
    }
    fn insert_type(&self, v: &mut Vec<AHashMap<u32, PropColumn>>, prop_id: &u32, typ: Option<u32>) {
        for part in v {
            for (prop_id_inner, col) in part.iter_mut() {
                if prop_id == prop_id_inner {
                    col.resolve_vec_type(typ);
                }
                //insert_df(&col.data, *name, &mut big);
            }
        }
    }
    fn resolve_type(&self, v: &mut Vec<AHashMap<u32, PropColumn>>, prop_id: &u32) -> Option<u32> {
        let mut cor_type = None;
        for part in v {
            for (prop_id_inner, col) in part.iter_mut() {
                if prop_id == prop_id_inner {
                    let this_type = PropColumn::get_type(&col.data);

                    if cor_type != None && this_type != None && this_type != cor_type {
                        panic!("ILLEGAL PROP TYPES")
                    }
                    cor_type = this_type;
                }
                //insert_df(&col.data, *name, &mut big);
            }
        }
        cor_type
        /*
        for part in v {
            for (name, col) in part.iter_mut() {
                col.resolve_vec_type(cor_type);
            }
        }
        */
    }
}

fn insert_df(v: &Option<VarVec>, prop_id: u32, map: &mut AHashMap<u32, PropColumn>) {
    match v {
        Some(VarVec::I32(i)) => match map.get_mut(&prop_id) {
            Some(p) => {
                if let Some(VarVec::I32(ii)) = &mut p.data {
                    ii.extend(i);
                } else {
                    panic!("INSERT {:?}", v);
                }
            }
            _ => {
                panic!("INSERT {:?}", v);
            }
        },
        Some(VarVec::U64(i)) => match map.get_mut(&prop_id) {
            Some(p) => {
                if let Some(VarVec::U64(ii)) = &mut p.data {
                    ii.extend(i);
                } else {
                    panic!("INSERT {:?}", v);
                }
            }
            _ => {
                panic!("INSERT {:?}", v);
            }
        },
        Some(VarVec::String(i)) => match map.get_mut(&prop_id) {
            Some(p) => {
                if let Some(VarVec::String(ii)) = &mut p.data {
                    ii.extend_from_slice(i);
                } else {
                    panic!("INSERT {:?}", v);
                }
            }
            _ => {
                panic!("INSERT {:?}", v);
            }
        },
        Some(VarVec::U32(i)) => match map.get_mut(&prop_id) {
            Some(p) => {
                if let Some(VarVec::U32(ii)) = &mut p.data {
                    ii.extend(i);
                } else {
                    panic!("INSERT {:?}", v);
                }
            }
            _ => {
                panic!("INSERT {:?}", v);
            }
        },
        _ => {}
    }
}

impl Parser {
    pub fn parse_packet(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let packet: CDemoPacket = Message::parse_from_bytes(bytes).unwrap();
        let packet_data = packet.data.unwrap();
        let mut bitreader = Bitreader::new(&packet_data);
        // Inner loop
        while bitreader.reader.has_bits_remaining(8) {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            let msg_bytes = bitreader.read_n_bytes(size as usize)?;

            let ok = match netmessage_type_from_int(msg_type as i32) {
                GE_Source1LegacyGameEventList => self.parse_game_event_list(&msg_bytes),
                //GE_Source1LegacyGameEvent => self.parse_event(&msg_bytes),
                svc_CreateStringTable => self.parse_create_stringtable(&msg_bytes),
                svc_UpdateStringTable => self.update_string_table(&msg_bytes),
                _ => Ok(()),
            };
            ok?
        }
        Ok(())
    }

    pub fn parse_header(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let header: CDemoFileHeader = Message::parse_from_bytes(bytes).unwrap();
        self.header.insert(
            "demo_file_stamp".to_string(),
            header.demo_file_stamp().to_string(),
        );
        self.header.insert(
            "demo_version_guid".to_string(),
            header.demo_version_guid().to_string(),
        );
        self.header.insert(
            "network_protocol".to_string(),
            header.network_protocol().to_string(),
        );
        self.header
            .insert("server_name".to_string(), header.server_name().to_string());
        self.header
            .insert("client_name".to_string(), header.client_name().to_string());
        self.header
            .insert("map_name".to_string(), header.map_name().to_string());
        self.header.insert(
            "game_directory".to_string(),
            header.game_directory().to_string(),
        );
        self.header.insert(
            "fullpackets_version".to_string(),
            header.fullpackets_version().to_string(),
        );
        self.header.insert(
            "allow_clientside_entities".to_string(),
            header.allow_clientside_entities().to_string(),
        );
        self.header.insert(
            "allow_clientside_particles".to_string(),
            header.allow_clientside_particles().to_string(),
        );
        self.header.insert(
            "allow_clientside_particles".to_string(),
            header.allow_clientside_particles().to_string(),
        );
        self.header
            .insert("addons".to_string(), header.addons().to_string());
        self.header.insert(
            "demo_version_name".to_string(),
            header.demo_version_name().to_string(),
        );
        self.header
            .insert("addons".to_string(), header.addons().to_string());
        Ok(())
    }

    pub fn parse_class_info(
        &mut self,
        bytes: &[u8],
        sendtables: CDemoSendTables,
    ) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
        self.parse_sendtable(sendtables);
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();
        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();

            self.cls_by_id.insert(
                cls_id as u32,
                Class {
                    class_id: cls_id,
                    name: network_name.to_string(),
                    serializer: self.serializers[network_name].clone(),
                },
            );
        }
        Ok(())
    }
}
