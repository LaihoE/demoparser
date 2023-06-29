use crate::collect_data::ProjectileRecord;
use crate::decoder::QfMapper;
use crate::game_events::GameEvent;
use crate::netmessage_types;
use crate::netmessage_types::netmessage_type_from_int;
use crate::parser_settings::Parser;
use crate::parser_settings::ParserInputs;
use crate::parser_thread_settings::ChatMessageRecord;
use crate::parser_thread_settings::EconItem;
use crate::parser_thread_settings::ParserThread;
use crate::parser_thread_settings::PlayerEndMetaData;
use crate::parser_threads::demo_cmd_type_from_int;
use crate::read_bits::Bitreader;
use crate::sendtables::PropController;
use crate::variants::PropColumn;
use crate::{other_netmessages::Class, read_bits::DemoParserError};
use ahash::AHashMap;
use bitter::BitReader;
use csgoproto::demo::CDemoClassInfo;
use csgoproto::demo::CDemoFileHeader;
use csgoproto::demo::CDemoPacket;
use csgoproto::demo::CDemoSendTables;
use csgoproto::demo::EDemoCommands::*;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use netmessage_types::NetmessageType::*;
use protobuf::Message;
use snap::raw::Decoder as SnapDecoder;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug)]
pub struct DemoOutput {
    pub df: AHashMap<u32, PropColumn>,
    pub game_events: Vec<GameEvent>,
    pub skins: Vec<EconItem>,
    pub item_drops: Vec<EconItem>,
    pub chat_messages: Vec<ChatMessageRecord>,
    pub convars: AHashMap<String, String>,
    pub header: Option<AHashMap<String, String>>,
    pub player_md: Vec<PlayerEndMetaData>,
    pub game_events_counter: AHashMap<String, i32>,
    pub prop_info: Arc<PropController>,
    pub projectiles: Vec<ProjectileRecord>,
    pub ptr: usize,
}

impl Parser {
    fn create_parser_thread_input(&self, offset: usize, parse_all: bool) -> ParserThreadInput {
        ParserThreadInput {
            offset: offset,
            settings: self.settings.clone(),
            baselines: self.baselines.clone(),
            prop_controller: self.prop_controller.clone(),
            cls_by_id: self.cls_by_id.clone(),
            qfmap: self.qf_mapper.clone(),
            ge_list: self.ge_list.clone(),
            parse_all_packets: parse_all,
        }
    }

    pub fn parse_demo(&mut self) -> Result<DemoOutput, DemoParserError> {
        self.ptr = 16;
        self.fullpacket_offsets.push(16);

        let mut sendtable: Option<CDemoSendTables> = None;
        let mut handle: Option<JoinHandle<Result<_, _>>> = None;
        let mut threads_spawned = 0;
        let out = thread::scope(|s| {
            let mut handles = vec![];
            loop {
                // If sendtables is done
                if handle.is_some() && handle.as_ref().unwrap().is_finished() {
                    let mut class_result: ClassInfoThreadResult =
                        handle.take().unwrap().join().unwrap().unwrap();
                    self.insert_cls_thread_result(class_result);
                }
                // Spawn thread if done
                if self.fullpacket_offsets.len() > 0 && self.is_ready_to_spawn_thread() {
                    let offset = self.fullpacket_offsets.pop().unwrap();
                    let input = self.create_parser_thread_input(offset, false);
                    threads_spawned += 1;
                    handles.push(s.spawn(|| {
                        let mut parser = ParserThread::new(input).unwrap();
                        parser.start().unwrap();
                        parser.create_output()
                    }));
                }

                let before = self.ptr;
                let cmd = self.read_varint().unwrap();
                let tick = self.read_varint().unwrap();
                let size = self.read_varint().unwrap();
                self.tick = tick as i32;

                if self.ptr + size as usize >= self.bytes.len() {
                    break;
                }

                let msg_type = cmd & !64;
                let is_compressed = (cmd & 64) == 64;
                let demo_cmd = demo_cmd_type_from_int(msg_type as i32).unwrap();

                if demo_cmd == DEM_Packet || demo_cmd == DEM_AnimationData {
                    self.ptr += size as usize;
                    continue;
                }

                let bytes = match is_compressed {
                    true => SnapDecoder::new()
                        .decompress_vec(self.read_n_bytes(size).unwrap())
                        .unwrap(),
                    false => self.read_n_bytes(size).unwrap().to_vec(),
                };

                let ok: Result<(), DemoParserError> = match demo_cmd {
                    DEM_SendTables => {
                        sendtable = Some(Message::parse_from_bytes(&bytes).unwrap());
                        // self.parse_sendtable(&bytes)
                        Ok(())
                    }
                    DEM_FileHeader => self.parse_header(&bytes),
                    DEM_ClassInfo => {
                        let my_s = sendtable.clone();
                        let my_b = bytes.clone();
                        let want_prop = self.wanted_player_props.clone();
                        let want_prop_og = self.wanted_player_props_og_names.clone();
                        let real_to_og = self.real_name_to_og_name.clone();
                        handle = Some(thread::spawn(move || {
                            Parser::parse_class_info(
                                &my_b,
                                my_s.unwrap(),
                                want_prop,
                                want_prop_og,
                                real_to_og,
                            )
                        }));
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
                ok.unwrap();
            }
            while !self.is_ready_to_spawn_thread() {
                if handle.is_some() && handle.as_ref().unwrap().is_finished() {
                    let mut class_result: ClassInfoThreadResult =
                        handle.take().unwrap().join().unwrap().unwrap();
                    self.insert_cls_thread_result(class_result);
                    break;
                }
            }
            if threads_spawned == 0 && self.fullpacket_offsets.len() == 0 {
                let input = self.create_parser_thread_input(16, true);
                handles.push(s.spawn(|| {
                    let mut parser = ParserThread::new(input).unwrap();
                    parser.start().unwrap();
                    parser.create_output()
                }));
            } else {
                while self.fullpacket_offsets.len() > 0 {
                    let offset = self.fullpacket_offsets.pop().unwrap();
                    let input = self.create_parser_thread_input(offset, false);
                    handles.push(s.spawn(|| {
                        let mut parser = ParserThread::new(input).unwrap();
                        parser.start().unwrap();
                        parser.create_output()
                    }));
                }
            }
            let mut outputs: Vec<DemoOutput> = vec![];
            for handle in handles {
                outputs.push(handle.join().unwrap());
            }
            outputs.sort_by_key(|x| x.ptr);
            let mut dfs = outputs.iter().map(|x| x.df.clone()).collect();
            let all_dfs_combined = self.combine_dfs(&mut dfs);
            DemoOutput {
                chat_messages: outputs
                    .iter()
                    .flat_map(|x| x.chat_messages.clone())
                    .collect(),
                item_drops: outputs.iter().flat_map(|x| x.item_drops.clone()).collect(),
                player_md: outputs.iter().flat_map(|x| x.player_md.clone()).collect(),
                game_events: outputs.iter().flat_map(|x| x.game_events.clone()).collect(),
                skins: outputs.iter().flat_map(|x| x.skins.clone()).collect(),
                convars: outputs.iter().flat_map(|x| x.convars.clone()).collect(),
                df: all_dfs_combined,
                header: Some(self.header.clone()),
                game_events_counter: AHashMap::default(),
                prop_info: self.prop_controller.clone(),
                projectiles: outputs.iter().flat_map(|x| x.projectiles.clone()).collect(),
                ptr: self.ptr,
            }
        });
        self.prop_controller_is_set = false;
        Ok(out)
    }
    fn insert_cls_thread_result(&mut self, mut class_result: ClassInfoThreadResult) {
        class_result.prop_controller.wanted_player_props = self.wanted_player_props.clone();
        class_result.prop_controller.wanted_player_og_props =
            self.wanted_player_props_og_names.clone();
        class_result.prop_controller.real_name_to_og_name = self.real_name_to_og_name.clone();

        self.qf_map_set = true;
        self.cls_by_id_set = true;
        self.prop_controller_is_set = true;
        self.prop_controller = Arc::new(class_result.prop_controller);
        self.qf_mapper = Arc::new(class_result.qf_mapper);
        self.cls_by_id = Arc::new(class_result.cls_by_id);
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
}

impl Parser {
    pub fn is_ready_to_spawn_thread(&self) -> bool {
        /*
        println!(
            "{} {} {} {}",
            self.qf_map_set, self.cls_by_id_set, self.ge_list_set, self.prop_controller_is_set
        );
        */
        self.qf_map_set && self.cls_by_id_set && self.ge_list_set && self.prop_controller_is_set
    }
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
                GE_Source1LegacyGameEventList => {
                    let hm = self.parse_game_event_list(&msg_bytes)?;
                    // this can fail if user re-uses the same parser for multiple funcs
                    self.ge_list = Arc::new(hm);
                    self.ge_list_set = true;
                    //GE_LIST.set(hm);
                    Ok(())
                }
                // GE_Source1LegacyGameEvent => self.parse_event(&msg_bytes),
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
        bytes: &[u8],
        sendtables: CDemoSendTables,
        want_prop: Vec<String>,
        want_prop_og: Vec<String>,
        real_name_to_og_name: AHashMap<String, String>,
    ) -> Result<ClassInfoThreadResult, DemoParserError> {
        let (serializers, qf_mapper, p) =
            Parser::parse_sendtable(sendtables, want_prop, want_prop_og, real_name_to_og_name)?;
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();
        let mut cls_by_id = AHashMap::default();
        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();
            cls_by_id.insert(
                cls_id as u32,
                Class {
                    class_id: cls_id,
                    name: network_name.to_string(),
                    serializer: serializers[network_name].clone(),
                },
            );
        }
        Ok(ClassInfoThreadResult {
            cls_by_id: cls_by_id,
            qf_mapper: qf_mapper,
            prop_controller: p,
        })
    }
}

pub struct ParserThreadInput {
    pub offset: usize,
    pub settings: ParserInputs,
    pub baselines: AHashMap<u32, Vec<u8>>,
    pub prop_controller: Arc<PropController>,
    pub cls_by_id: Arc<AHashMap<u32, Class>>,
    pub qfmap: Arc<QfMapper>,
    pub ge_list: Arc<AHashMap<i32, Descriptor_t>>,
    pub parse_all_packets: bool,
}

pub struct ClassInfoThreadResult {
    pub cls_by_id: AHashMap<u32, Class>,
    pub qf_mapper: QfMapper,
    pub prop_controller: PropController,
}
