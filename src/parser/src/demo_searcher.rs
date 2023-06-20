use crate::collect_data::TYPEHM;
use crate::decoder::QfMapper;
use crate::game_events::GameEvent;
use crate::netmessage_types;
use crate::netmessage_types::netmessage_type_from_int;
use crate::parser::demo_cmd_type_from_int;
use crate::parser_settings::create_huffman_lookup_table;
use crate::parser_settings::Parser;
use crate::parser_settings::ParserInputs;
use crate::parser_settings::SpecialIDs;
use crate::sendtables::PropInfo;
use crate::sendtables::Serializer;
use crate::variants::PropColumn;
use crate::variants::VarVec;
use crate::{other_netmessages::Class, parser, read_bits::DemoParserError};
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::RandomState;
use csgoproto::demo::EDemoCommands::*;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use dashmap::DashMap;
use memmap2::Mmap;
use protobuf::Message;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use snap::raw::Decoder as SnapDecoder;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

pub struct DemoSearcher {
    pub fullpacket_offsets: Vec<usize>,
    pub ptr: usize,
    pub bytes: Arc<Mmap>,
    pub tick: i32,
    pub huf: Arc<Vec<(u32, u8)>>,
    pub settings: ParserInputs,
    pub handles: Vec<JoinHandle<()>>,
    pub serializers: AHashMap<String, Serializer>,
    pub cls_by_id: AHashMap<u32, Class>,
    pub start: Instant,
    pub ge_list: Option<AHashMap<i32, Descriptor_t>>,

    pub wanted_player_props: Vec<String>,

    pub wanted_ticks: AHashSet<i32, RandomState>,
    pub wanted_player_props_og_names: Vec<String>,
    // Team and rules props
    pub wanted_other_props: Vec<String>,
    pub wanted_other_props_og_names: Vec<String>,
    pub wanted_event: Option<String>,
    pub parse_entities: bool,
    pub parse_projectiles: bool,

    pub prop_name_to_path: AHashMap<String, [i32; 7]>,
    pub path_to_prop_name: AHashMap<[i32; 7], String>,
    pub wanted_prop_paths: AHashSet<[i32; 7]>,
    pub name_to_id: AHashMap<String, u32>,

    pub qf_mapper: QfMapper,

    pub id: u32,
    pub wanted_prop_ids: Vec<u32>,
    pub controller_ids: SpecialIDs,
    pub player_output_ids: Vec<u8>,
    pub prop_out_id: u8,
    pub id_to_path: AHashMap<u32, [i32; 7]>,
    pub prop_infos: Vec<PropInfo>,

    pub header: AHashMap<String, String>,
}

impl DemoSearcher {
    pub fn front_demo_metadata(&mut self) -> Result<AHashMap<u32, PropColumn>, DemoParserError> {
        self.ptr = 16;
        self.fullpacket_offsets.push(16);

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
            let cmd = demo_cmd_type_from_int(msg_type as i32);

            let bytes = match is_compressed {
                true => SnapDecoder::new()
                    .decompress_vec(self.read_n_bytes(size)?)
                    .unwrap(),
                false => self.read_n_bytes(size)?.to_vec(),
            };
            let ok: Result<(), DemoParserError> =
                match demo_cmd_type_from_int(msg_type as i32).unwrap() {
                    DEM_SendTables => {
                        self.parse_sendtable(Message::parse_from_bytes(&bytes).unwrap())
                            .unwrap();
                        Ok(())
                    }
                    DEM_FileHeader => {
                        self.parse_header(&bytes).unwrap();
                        Ok(())
                    }
                    DEM_ClassInfo => {
                        self.parse_class_info(&bytes).unwrap();
                        Ok(())
                    }
                    DEM_FullPacket => {
                        self.fullpacket_offsets.push(before);
                        Ok(())
                    }
                    DEM_SignonPacket => {
                        self.parse_packet(&bytes).unwrap();
                        Ok(())
                    }
                    DEM_Stop => {
                        break;
                    }
                    _ => Ok(()),
                };
            ok?;
        }
        let mut outputs: Vec<Parser> = self
            .fullpacket_offsets
            .par_iter()
            .map(|offset| {
                let mut parser = Parser::new(self.settings.clone(), &self.cls_by_id).unwrap();
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
                parser.wanted_event = Some("player_death".to_string());
                parser.start().unwrap();
                parser
            })
            .collect();
        let evs: Vec<GameEvent> = outputs.iter().flat_map(|p| p.game_events.clone()).collect();
        //let big = self.combine_dfs(&mut outputs);
        Ok(AHashMap::default())
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

use crate::read_bits::Bitreader;
use bitter::BitReader;
use csgoproto::demo::CDemoClassInfo;
use csgoproto::demo::CDemoFileHeader;
use csgoproto::demo::CDemoPacket;

use csgoproto::demo::*;
use csgoproto::netmessages::*;
use netmessage_types::NetmessageType::*;

impl DemoSearcher {
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

    pub fn parse_class_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
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
        println!("{:2?}", self.start.elapsed());
        Ok(())
    }
}
