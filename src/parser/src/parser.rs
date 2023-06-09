use crate::collect_data::ProjectileRecord;
use crate::decoder::QfMapper;
use crate::game_events::GameEvent;
use crate::netmessage_types;
use crate::netmessage_types::netmessage_type_from_int;
use crate::parser_settings::Parser;
use crate::parser_settings::ParserInputs;
use crate::parser_thread_settings::*;
use crate::parser_threads::demo_cmd_type_from_int;
use crate::prop_controller::PropController;
use crate::read_bits::Bitreader;
use crate::variants::PropColumn;
use crate::{other_netmessages::Class, read_bits::DemoParserError};
use ahash::AHashMap;
use ahash::AHashSet;
use bitter::BitReader;
use csgoproto::demo::EDemoCommands::*;
use csgoproto::demo::{CDemoClassInfo, CDemoFileHeader, CDemoPacket, CDemoSendTables};
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use netmessage_types::NetmessageType::*;
use protobuf::Message;
use snap::raw::Decoder as SnapDecoder;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

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
    pub game_events_counter: AHashSet<String>,
    pub prop_info: Arc<PropController>,
    pub projectiles: Vec<ProjectileRecord>,
    pub ptr: usize,
}

impl Parser {
    pub fn create_parser_thread_input(&self, offset: usize, parse_all: bool) -> ParserThreadInput {
        ParserThreadInput {
            offset: offset,
            settings: self.settings.clone(),
            baselines: self.baselines.clone(),
            prop_controller: self.prop_controller.clone(),
            cls_by_id: self.cls_by_id.clone(),
            qfmap: self.qf_mapper.clone(),
            ge_list: self.ge_list.clone(),
            parse_all_packets: parse_all,
            // arc?
            wanted_ticks: self.wanted_ticks.clone(),
        }
    }

    pub fn parse_demo(&mut self) -> Result<DemoOutput, DemoParserError> {
        Parser::handle_short_header(self.bytes.len(), &self.bytes[..16])?;
        self.ptr = 16;
        self.fullpacket_offsets.push(16);
        let mut sendtable: Option<CDemoSendTables> = None;
        let mut handle: Option<JoinHandle<Result<_, _>>> = None;
        let mut handles = vec![];
        loop {
            // After this we can start spawning threads
            if self.should_finalize_serial_part(&handle) {
                self.finalize_serial_part(&mut handle);
            }
            if self.should_spawn_thread() {
                self.spawn_thread(&mut handles);
            }
            let frame_starts_at = self.ptr;

            let cmd = self.read_varint()?;
            let tick = self.read_varint()?;
            let size = self.read_varint()?;
            self.tick = tick as i32;

            // Safety check
            if self.ptr + size as usize >= self.bytes.len() {
                break;
            }
            let msg_type = cmd & !64;
            let is_compressed = (cmd & 64) == 64;
            let demo_cmd = demo_cmd_type_from_int(msg_type as i32).unwrap();

            // Early exit these for performance reasons
            if demo_cmd == DEM_Packet || demo_cmd == DEM_AnimationData {
                self.ptr += size as usize;
                continue;
            }
            let bytes = match is_compressed {
                true => SnapDecoder::new().decompress_vec(self.read_n_bytes(size).unwrap()).unwrap(),
                false => self.read_n_bytes(size)?.to_vec(),
            };
            let ok: Result<(), DemoParserError> = match demo_cmd {
                DEM_SendTables => {
                    sendtable = Some(Message::parse_from_bytes(&bytes).unwrap());
                    Ok(())
                }
                DEM_FileHeader => self.parse_header(&bytes),
                DEM_ClassInfo => {
                    handle = self.spawn_clsinfo_thread(sendtable.take(), bytes);
                    Ok(())
                }
                DEM_SignonPacket => self.parse_packet(&bytes),
                DEM_Stop => break,
                DEM_FullPacket => {
                    self.fullpacket_offsets.push(frame_starts_at);
                    Ok(())
                }
                _ => Ok(()),
            };
            ok?;
        }
        // If clsinfo thread is very slow then need to join it here
        // since normally the join is done in the above loop
        self.make_sure_serial_is_done(&mut handle);
        // if demo does not have fullpackets, spawn one thread that parses entire demo
        if self.threads_spawned == 0 && self.fullpacket_offsets.len() == 0 {
            let input = self.create_parser_thread_input(16, true);
            handles.push(thread::spawn(|| {
                let mut parser = ParserThread::new(input).unwrap();
                parser.start().unwrap();
                parser.create_output()
            }));
        } else {
            // spawn rest of the threads if any
            while self.fullpacket_offsets.len() > 0 {
                self.spawn_thread(&mut handles);
            }
        }
        let mut outputs: Vec<DemoOutput> = vec![];
        for handle in handles {
            outputs.push(handle.join().unwrap());
        }
        Ok(self.combine_thread_outputs(&mut outputs))
    }
    fn make_sure_serial_is_done(&mut self, handle: &mut Option<JoinHandle<Result<ClassInfoThreadResult, DemoParserError>>>) {
        while !self.is_ready_to_spawn_thread() {
            if handle.is_some() && handle.as_ref().unwrap().is_finished() {
                let class_result: ClassInfoThreadResult = handle.take().unwrap().join().unwrap().unwrap();
                self.insert_cls_thread_result(class_result);
                break;
            }
        }
    }
    pub fn spawn_thread(&mut self, handles: &mut Vec<JoinHandle<DemoOutput>>) {
        let offset = self.fullpacket_offsets.pop().unwrap();
        let input = self.create_parser_thread_input(offset, false);
        handles.push(thread::spawn(|| {
            let mut parser = ParserThread::new(input).unwrap();
            parser.start().unwrap();
            parser.create_output()
        }));
        self.threads_spawned += 1;
    }
    fn should_spawn_thread(&self) -> bool {
        self.fullpacket_offsets.len() > 0 && self.is_ready_to_spawn_thread()
    }
    fn should_finalize_serial_part(&self, handle: &Option<JoinHandle<Result<ClassInfoThreadResult, DemoParserError>>>) -> bool {
        handle.is_some() && handle.as_ref().unwrap().is_finished()
    }
    fn finalize_serial_part(&mut self, handle: &mut Option<JoinHandle<Result<ClassInfoThreadResult, DemoParserError>>>) {
        let class_result: ClassInfoThreadResult = handle.take().unwrap().join().unwrap().unwrap();
        self.insert_cls_thread_result(class_result);
    }
    fn spawn_clsinfo_thread(
        &mut self,
        sendtable: Option<CDemoSendTables>,
        bytes: Vec<u8>,
    ) -> Option<JoinHandle<Result<ClassInfoThreadResult, DemoParserError>>> {
        let my_s = sendtable.clone();
        let my_b = bytes;
        let want_prop = self.wanted_player_props.clone();
        let want_prop_og = self.wanted_player_props_og_names.clone();
        let real_to_og = self.real_name_to_og_name.clone();
        Some(thread::spawn(move || {
            Parser::parse_class_info(&my_b, my_s.unwrap(), want_prop, want_prop_og, real_to_og)
        }))
    }
    fn combine_thread_outputs(&mut self, outputs: &mut Vec<DemoOutput>) -> DemoOutput {
        // Combines all inner DemoOutputs into one big output
        outputs.sort_by_key(|x| x.ptr);
        let mut dfs = outputs.iter().map(|x| x.df.clone()).collect();
        let all_dfs_combined = self.combine_dfs(&mut dfs);
        let all_game_events: AHashSet<String> =
            AHashSet::from_iter(outputs.iter().flat_map(|x| x.game_events_counter.iter().cloned()));
        DemoOutput {
            chat_messages: outputs.iter().flat_map(|x| x.chat_messages.clone()).collect(),
            item_drops: outputs.iter().flat_map(|x| x.item_drops.clone()).collect(),
            player_md: outputs.iter().flat_map(|x| x.player_md.clone()).collect(),
            game_events: outputs.iter().flat_map(|x| x.game_events.clone()).collect(),
            skins: outputs.iter().flat_map(|x| x.skins.clone()).collect(),
            convars: outputs.iter().flat_map(|x| x.convars.clone()).collect(),
            df: all_dfs_combined,
            header: Some(self.header.clone()),
            game_events_counter: all_game_events,
            prop_info: self.prop_controller.clone(),
            projectiles: outputs.iter().flat_map(|x| x.projectiles.clone()).collect(),
            ptr: self.ptr,
        }
    }
    fn insert_cls_thread_result(&mut self, mut class_result: ClassInfoThreadResult) {
        class_result.prop_controller.wanted_player_props = self.wanted_player_props.clone();
        class_result.prop_controller.wanted_player_og_props = self.wanted_player_props_og_names.clone();
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
                    Ok(())
                }
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
        self.header
            .insert("demo_file_stamp".to_string(), header.demo_file_stamp().to_string());
        self.header
            .insert("demo_version_guid".to_string(), header.demo_version_guid().to_string());
        self.header
            .insert("network_protocol".to_string(), header.network_protocol().to_string());
        self.header
            .insert("server_name".to_string(), header.server_name().to_string());
        self.header
            .insert("client_name".to_string(), header.client_name().to_string());
        self.header.insert("map_name".to_string(), header.map_name().to_string());
        self.header
            .insert("game_directory".to_string(), header.game_directory().to_string());
        self.header
            .insert("fullpackets_version".to_string(), header.fullpackets_version().to_string());
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
        self.header.insert("addons".to_string(), header.addons().to_string());
        self.header
            .insert("demo_version_name".to_string(), header.demo_version_name().to_string());
        self.header.insert("addons".to_string(), header.addons().to_string());
        Ok(())
    }
    fn handle_short_header(file_len: usize, bytes: &[u8]) -> Result<(), DemoParserError> {
        match std::str::from_utf8(&bytes[..8]) {
            Ok(magic) => match magic {
                "PBDEMS2\0" => {}
                "HL2DEMO\0" => {
                    return Err(DemoParserError::Source1DemoError);
                }
                _ => {
                    return Err(DemoParserError::UnknownFile);
                }
            },
            Err(_) => {}
        };
        // hmmmm not sure where the 18 comes from if the header is only 16?
        // can be used to check that file ends early
        let file_length_expected = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) + 18;
        let missing_bytes = file_length_expected - file_len as u32;
        if missing_bytes != 0 {
            return Err(DemoParserError::DemoEndsEarly(format!(
                "demo ends early. Expected legth: {}, file lenght: {}. Missing: {:.2}%",
                file_length_expected,
                file_len,
                100.0 - (file_len as f32 / file_length_expected as f32 * 100.0),
            )));
        }
        // seems to be byte offset to where DEM_END command happens. After that comes Spawngroups and fileinfo. odd...
        let _no_clue_what_this_is = i32::from_le_bytes(bytes[12..].try_into().unwrap());
        Ok(())
    }

    pub fn parse_class_info(
        bytes: &[u8],
        sendtables: CDemoSendTables,
        want_prop: Vec<String>,
        want_prop_og: Vec<String>,
        real_name_to_og_name: AHashMap<String, String>,
    ) -> Result<ClassInfoThreadResult, DemoParserError> {
        let (mut serializers, qf_mapper, p) = Parser::parse_sendtable(sendtables, want_prop, want_prop_og, real_name_to_og_name)?;
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
                    serializer: serializers.remove(network_name).unwrap(), // [network_name].clone(),
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
    pub wanted_ticks: AHashSet<i32>,
}

pub struct ClassInfoThreadResult {
    pub cls_by_id: AHashMap<u32, Class>,
    pub qf_mapper: QfMapper,
    pub prop_controller: PropController,
}
