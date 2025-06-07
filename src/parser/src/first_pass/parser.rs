use crate::first_pass::parser_settings::FirstPassParser;
use crate::first_pass::parser_settings::ParserInputs;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::read_bits::read_varint;
use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Serializer;
use crate::first_pass::stringtables::parse_userinfo;
use crate::first_pass::stringtables::StringTable;
use crate::first_pass::stringtables::UserInfo;
use crate::maps::demo_cmd_type_from_int;

use crate::second_pass::decoder::QfMapper;
use crate::second_pass::other_netmessages::Class;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::csvc_msg_game_event_list::DescriptorT;
use csgoproto::message_type::NetMessageType::{self, *};
use csgoproto::CDemoClassInfo;
use csgoproto::CDemoFileHeader;
use csgoproto::CDemoFullPacket;
use csgoproto::CDemoPacket;
use csgoproto::CDemoSendTables;
use csgoproto::CsvcMsgGameEventList;
use csgoproto::EDemoCommands;
use prost::Message;
use snap::raw::decompress_len;
use snap::raw::Decoder as SnapDecoder;
use std::collections::BTreeMap;
use std::sync::Arc;

pub const HEADER_ENDS_AT_BYTE: usize = 16;

pub struct ParserThreadInput<'a> {
    pub offset: usize,
    pub settings: &'a ParserInputs<'a>,
    pub baselines: AHashMap<u32, Vec<u8>>,
    pub prop_controller: &'a PropController,
    pub cls_by_id: &'a Vec<Class>,
    pub qfmap: &'a QfMapper,
    pub ge_list: &'a AHashMap<i32, DescriptorT>,
    pub parse_all_packets: bool,
    pub wanted_ticks: AHashSet<i32>,
    pub string_tables: Vec<StringTable>,
    pub stringtable_players: BTreeMap<u64, UserInfo>,
}

pub struct ClassInfoThreadResult {
    pub cls_by_id: AHashMap<u32, Class>,
    pub qf_mapper: QfMapper,
    pub prop_controller: PropController,
}
#[derive(Debug, Clone)]
pub struct FirstPassOutput<'a> {
    pub fullpacket_offsets: Vec<usize>,
    pub settings: &'a ParserInputs<'a>,
    pub baselines: AHashMap<u32, Vec<u8>>,
    pub prop_controller: &'a PropController,
    pub cls_by_id: &'a Vec<Class>,
    pub qfmap: &'a QfMapper,
    pub ge_list: &'a AHashMap<i32, DescriptorT>,
    pub wanted_ticks: AHashSet<i32>,
    pub string_tables: Vec<StringTable>,
    pub stringtable_players: BTreeMap<i32, UserInfo>,
    pub added_temp_props: Vec<String>,
    pub wanted_players: AHashSet<u64>,
    pub header: AHashMap<String, String>,
    pub order_by_steamid: bool,
    pub list_props: bool,
}
#[derive(Debug)]
pub struct Frame {
    pub tick: i32,
    pub size: usize,
    pub frame_starts_at: usize,
    pub is_compressed: bool,
    pub demo_cmd: EDemoCommands,
}

impl<'a> FirstPassParser<'a> {
    pub fn parse_header_only(&mut self, demo_bytes: &'a [u8]) -> Result<AHashMap<String, String>, DemoParserError> {
        self.handle_short_header(demo_bytes.len(), &demo_bytes[..HEADER_ENDS_AT_BYTE])?;
        let frame = self.read_frame(demo_bytes)?;
        let bytes = self.slice_packet_bytes(demo_bytes, frame.size)?;
        if frame.demo_cmd != EDemoCommands::DemFileHeader {
            return Err(DemoParserError::MalformedMessage)
        }
        self.parse_header(bytes)?;
        Ok(self.header.clone())
    }
    pub fn parse_demo(&mut self, demo_bytes: &'a [u8], exit_early: bool) -> Result<FirstPassOutput, DemoParserError> {
        self.handle_short_header(demo_bytes.len(), &demo_bytes[..HEADER_ENDS_AT_BYTE])?;
        let mut reuseable_buffer = vec![0_u8; 100_000];
        // Loop that goes trough the entire file
        loop {
            if demo_bytes.len() < self.ptr { break; }
            if exit_early && self.cls_by_id.is_some() && !self.ge_list.is_empty() {
                break;
            }
            let frame = self.read_frame(demo_bytes)?;
            if self.is_packet_we_skip_on_first_pass(frame.demo_cmd) {
                self.ptr += frame.size;
                continue;
            }
            let bytes = match self.slice_packet_bytes(demo_bytes, frame.size) {
                Ok(b) => b,
                Err(_) => {
                    self.ptr += frame.size;
                    continue;
                }
            };
            let bytes = self.decompress_if_needed(&mut reuseable_buffer, bytes, &frame)?;
            self.ptr += frame.size;
            match frame.demo_cmd {
                EDemoCommands::DemSendTables => self.parse_sendtable_bytes(bytes)?,
                EDemoCommands::DemFileHeader => self.parse_header(bytes)?,
                EDemoCommands::DemClassInfo => self.parse_class_info(bytes)?,
                EDemoCommands::DemSignonPacket => self.parse_packet(bytes)?,
                EDemoCommands::DemFullPacket => self.parse_full_packet(bytes, &frame)?,
                EDemoCommands::DemStop => break,
                _ => {}
            };
        }
        self.fallback_if_first_pass_missing_data()?;
        self.create_first_pass_output()
    }

    fn parse_sendtable_bytes(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        self.sendtable_message = match CDemoSendTables::decode(bytes) {
            Ok(m) => Some(m),
            Err(_e) => return Err(DemoParserError::MalformedMessage),
        };
        Ok(())
    }
    fn read_frame(&mut self, demo_bytes: &[u8]) -> Result<Frame, DemoParserError> {
        let frame_starts_at = self.ptr;
        let cmd = read_varint(demo_bytes, &mut self.ptr)?;
        let tick = read_varint(demo_bytes, &mut self.ptr)?;
        let size = read_varint(demo_bytes, &mut self.ptr)?;
        self.tick = tick as i32;

        let msg_type = cmd & !64;
        let is_compressed = (cmd & 64) == 64;
        let demo_cmd = demo_cmd_type_from_int(msg_type as i32)?;

        Ok(Frame {
            size: size as usize,
            frame_starts_at,
            is_compressed,
            demo_cmd,
            tick: self.tick,
        })
    }
    fn is_packet_we_skip_on_first_pass(&self, demo_cmd: EDemoCommands) -> bool {
        demo_cmd == EDemoCommands::DemPacket || demo_cmd == EDemoCommands::DemAnimationData
    }
    fn slice_packet_bytes(&mut self, demo_bytes: &'a [u8], frame_size: usize) -> Result<&'a [u8], DemoParserError> {
        if self.ptr + frame_size as usize >= demo_bytes.len() {
            return Err(DemoParserError::MalformedMessage);
        }
        Ok(&demo_bytes[self.ptr..self.ptr + frame_size])
    }
    fn decompress_if_needed<'b>(&mut self, buf: &'b mut Vec<u8>, possibly_uncompressed_bytes: &'b [u8], frame: &Frame) -> Result<&'b [u8], DemoParserError> {
        match frame.is_compressed {
            true => {
                FirstPassParser::resize_if_needed(buf, decompress_len(possibly_uncompressed_bytes))?;
                match SnapDecoder::new().decompress(possibly_uncompressed_bytes, buf) {
                    Ok(idx) => Ok(&buf[..idx]),
                    Err(e) => return Err(DemoParserError::DecompressionFailure(format!("{}", e))),
                }
            }
            false => Ok(possibly_uncompressed_bytes),
        }
    }
    pub fn resize_if_needed(buf: &mut Vec<u8>, needed_len: Result<usize, snap::Error>) -> Result<(), DemoParserError> {
        match needed_len {
            Ok(len) => {
                if buf.len() < len {
                    buf.resize(len, 0)
                }
            }
            Err(e) => return Err(DemoParserError::DecompressionFailure(e.to_string())),
        };
        Ok(())
    }
    pub fn parse_fallback_event_list(&mut self) -> Result<(), DemoParserError> {
        let bytes = match self.fallback_bytes {
            Some(b) => b,
            None => crate::first_pass::fallbackbytes::GAME_EVENT_LIST_FALLBACK_BYTES,
        };
        let event_list = match CsvcMsgGameEventList::decode(bytes) {
            Ok(list) => list,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        for event_desc in event_list.descriptors {
            self.ge_list.insert(event_desc.eventid(), event_desc);
        }
        Ok(())
    }
    pub fn create_first_pass_output(&self) -> Result<FirstPassOutput, DemoParserError> {
        let cls_by_id = match &self.cls_by_id {
            Some(c) => c,
            None => return Err(DemoParserError::ClassMapperNotFoundFirstPass),
        };
        Ok(FirstPassOutput {
            order_by_steamid: self.order_by_steamid,
            header: self.header.clone(),
            fullpacket_offsets: self.fullpacket_offsets.clone(),
            settings: &self.settings,
            baselines: self.baselines.clone(),
            prop_controller: &self.prop_controller,
            cls_by_id: &cls_by_id,
            qfmap: &self.qf_mapper,
            ge_list: &self.ge_list,
            // arc?
            wanted_players: self.wanted_players.clone(),
            wanted_ticks: self.wanted_ticks.clone(),
            string_tables: self.string_tables.clone(),
            stringtable_players: self.stringtable_players.clone(),
            added_temp_props: self.added_temp_props.clone(),
            list_props: self.list_props,
        })
    }
    fn fallback_if_first_pass_missing_data(&mut self) -> Result<(), DemoParserError> {
        if !self.fullpacket_offsets.contains(&HEADER_ENDS_AT_BYTE) {
            self.fullpacket_offsets.push(HEADER_ENDS_AT_BYTE);
        }
        if self.ge_list.is_empty() {
            self.parse_fallback_event_list()?;
        }
        Ok(())
    }
    // Message that should come before first game event
    pub fn parse_game_event_list(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let event_list = match CsvcMsgGameEventList::decode(bytes) {
            Ok(list) => list,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        let mut hm: AHashMap<i32, DescriptorT> = AHashMap::default();
        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.ge_list = hm;
        Ok(())
    }
    pub fn parse_full_packet(&mut self, bytes: &[u8], frame: &Frame) -> Result<(), DemoParserError> {
        self.fullpacket_offsets.push(frame.frame_starts_at);

        let full_packet = match CDemoFullPacket::decode(bytes) {
            Ok(list) => list,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        if let Some(string_table) = full_packet.string_table {
            for item in &string_table.tables {
                if item.table_name() == "instancebaseline" {
                    for i in &item.items {
                        let k = i.str().parse::<u32>().unwrap_or(u32::MAX);
                        self.baselines.insert(k, i.data().to_vec());
                    }
                }
                if item.table_name() == "userinfo" {
                    for i in &item.items {
                        if let Ok(player) = parse_userinfo(&i.data()) {
                            if player.steamid != 0 {
                                self.stringtable_players.insert(player.userid, player);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> FirstPassParser<'a> {
    pub fn parse_packet(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let msg = match CDemoPacket::decode(bytes) {
            Err(_) => return Err(DemoParserError::MalformedMessage),
            Ok(msg) => msg,
        };
        let mut bitreader = Bitreader::new(msg.data());

        while bitreader.bits_remaining().unwrap_or(0) > 8 {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            let msg_bytes = bitreader.read_n_bytes(size as usize)?;

            let ok = match NetMessageType::from(msg_type as i32) {
                GE_Source1LegacyGameEventList => self.parse_game_event_list(&msg_bytes),
                svc_CreateStringTable => self.parse_create_stringtable(&msg_bytes),
                svc_UpdateStringTable => self.update_string_table(&msg_bytes),
                svc_ClearAllStringTables => self.clear_stringtables(),
                _ => Ok(()),
            };
            ok?
        }
        Ok(())
    }
    fn clear_stringtables(&mut self) -> Result<(), DemoParserError> {
        self.string_tables = vec![];
        Ok(())
    }
    pub fn parse_header(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let header = match CDemoFileHeader::decode(bytes) {
            Ok(list) => list,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        self.header.insert("demo_file_stamp".to_string(), header.demo_file_stamp.to_string());
        self.header.insert("demo_version_guid".to_string(), header.demo_version_guid().to_string());
        self.header.insert("network_protocol".to_string(), header.network_protocol().to_string());
        self.header.insert("server_name".to_string(), header.server_name().to_string());
        self.header.insert("client_name".to_string(), header.client_name().to_string());
        self.header.insert("map_name".to_string(), header.map_name().to_string());
        self.header.insert("game_directory".to_string(), header.game_directory().to_string());
        self.header.insert("fullpackets_version".to_string(), header.fullpackets_version().to_string());
        self.header
            .insert("allow_clientside_entities".to_string(), header.allow_clientside_entities().to_string());
        self.header
            .insert("allow_clientside_particles".to_string(), header.allow_clientside_particles().to_string());
        self.header
            .insert("allow_clientside_particles".to_string(), header.allow_clientside_particles().to_string());
        self.header.insert("addons".to_string(), header.addons().to_string());
        self.header.insert("demo_version_name".to_string(), header.demo_version_name().to_string());
        self.header.insert("addons".to_string(), header.addons().to_string());
        Ok(())
    }
    fn handle_short_header(&mut self, file_len: usize, bytes: &[u8]) -> Result<(), DemoParserError> {
        if bytes.len() < 16 {
            return Err(DemoParserError::OutOfBytesError);
        }
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
        let file_length_expected = match bytes[8..12].try_into() {
            Err(_) => return Err(DemoParserError::OutOfBytesError),
            Ok(arr) => u32::from_le_bytes(arr) + 18,
        };
        let missing_percentage = 100.0 - (file_len as f32 / file_length_expected as f32 * 100.0);
        if missing_percentage > 10.0 {
            return Err(DemoParserError::DemoEndsEarly(format!(
                "demo ends early. Expected legth: {}, file lenght: {}. Missing: {:.2}%",
                file_length_expected,
                file_len,
                100.0 - (file_len as f32 / file_length_expected as f32 * 100.0),
            )));
        }
        // seems to be byte offset to where DEM_END command happens. After that comes Spawngroups and fileinfo. odd...
        let _no_clue_what_this_is = match bytes[8..12].try_into() {
            Err(_) => return Err(DemoParserError::OutOfBytesError),
            Ok(arr) => i32::from_le_bytes(arr),
        };
        self.ptr = HEADER_ENDS_AT_BYTE;
        Ok(())
    }

    pub fn parse_class_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let (mut serializers, qf_mapper, p) = self.parse_sendtable()?;
        let msg = match CDemoClassInfo::decode(bytes) {
            Err(_) => return Err(DemoParserError::MalformedMessage),
            Ok(msg) => msg,
        };
        let mut cls_by_id = vec![
            Class {
                class_id: 0,
                name: "None".to_string(),
                serializer: Serializer {
                    fields: vec![],
                    name: "None".to_string(),
                },
            };
            msg.classes.len() + 1
        ];
        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();

            if let Some(ser) = serializers.remove(network_name) {
                cls_by_id[cls_id as usize] = Class {
                    class_id: cls_id,
                    name: network_name.to_string(),
                    serializer: ser,
                }
            }
        }
        self.cls_by_id = Some(Arc::new(cls_by_id));
        self.qf_mapper = qf_mapper;
        self.prop_controller = p;
        return Ok(());
    }
}
