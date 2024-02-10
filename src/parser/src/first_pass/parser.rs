use crate::first_pass::parser_settings::FirstPassParser;
use crate::first_pass::parser_settings::ParserInputs;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Serializer;
use crate::first_pass::stringtables::parse_userinfo;
use crate::first_pass::stringtables::StringTable;
use crate::first_pass::stringtables::UserInfo;
use crate::maps::demo_cmd_type_from_int;
use crate::maps::netmessage_type_from_int;
use crate::maps::NetmessageType::*;
use crate::read_bytes::read_varint;
use crate::read_bytes::ProtoPacketParser;
use crate::second_pass::decoder::QfMapper;
use crate::second_pass::other_netmessages::Class;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::demo::CDemoFullPacket;
use csgoproto::demo::EDemoCommands::*;
use csgoproto::demo::{CDemoClassInfo, CDemoFileHeader, CDemoSendTables};
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use protobuf::Message;
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
    pub ge_list: &'a AHashMap<i32, Descriptor_t>,
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
    pub ge_list: &'a AHashMap<i32, Descriptor_t>,
    pub wanted_ticks: AHashSet<i32>,
    pub string_tables: Vec<StringTable>,
    pub stringtable_players: BTreeMap<u64, UserInfo>,
    pub added_temp_props: Vec<String>,
    pub wanted_players: AHashSet<u64>,
    pub header: AHashMap<String, String>,
}

impl<'a> FirstPassParser<'a> {
    pub fn parse_demo(&mut self, demo_bytes: &[u8]) -> Result<FirstPassOutput, DemoParserError> {
        FirstPassParser::handle_short_header(demo_bytes.len(), &demo_bytes[..HEADER_ENDS_AT_BYTE])?;
        self.ptr = HEADER_ENDS_AT_BYTE;
        let mut sendtable = None;
        let mut buf = vec![0_u8; 100_000];

        loop {
            let frame_starts_at = self.ptr;

            let cmd = read_varint(demo_bytes, &mut self.ptr)?;
            let tick = read_varint(demo_bytes, &mut self.ptr)?;
            let size = read_varint(demo_bytes, &mut self.ptr)?;
            self.tick = tick as i32;

            // Safety check
            if self.ptr + size as usize >= demo_bytes.len() {
                break;
            }

            let msg_type = cmd & !64;
            let is_compressed = (cmd & 64) == 64;
            let demo_cmd = demo_cmd_type_from_int(msg_type as i32).unwrap();

            // skip these for performance reasons
            if demo_cmd == DEM_Packet || demo_cmd == DEM_AnimationData {
                self.ptr += size as usize;
                continue;
            }
            let input = &demo_bytes[self.ptr..self.ptr + size as usize];
            FirstPassParser::resize_if_needed(&mut buf, decompress_len(input))?;

            self.ptr += size as usize;
            let bytes = match is_compressed {
                true => match SnapDecoder::new().decompress(input, &mut buf) {
                    Ok(idx) => &buf[..idx],
                    Err(e) => return Err(DemoParserError::DecompressionFailure(format!("{}", e))),
                },
                false => input,
            };

            let ok: Result<(), DemoParserError> = match demo_cmd {
                DEM_SendTables => {
                    sendtable = match Message::parse_from_bytes(&bytes) {
                        Ok(m) => Some(m),
                        Err(_e) => return Err(DemoParserError::MalformedMessage),
                    };
                    Ok(())
                }
                DEM_FileHeader => self.parse_header(&bytes),
                DEM_ClassInfo => {
                    let table = match sendtable.take() {
                        Some(table) => table,
                        None => return Err(DemoParserError::NoSendTableMessage),
                    };
                    self.parse_class_info(&bytes, table)?;
                    Ok(())
                }
                DEM_SignonPacket => self.parse_packet(&bytes),
                DEM_Stop => break,
                DEM_FullPacket => {
                    self.parse_full_packet(&bytes).unwrap();
                    self.fullpacket_offsets.push(frame_starts_at);
                    Ok(())
                }
                _ => Ok(()),
            };
            ok?;
        }
        self.fallback_if_first_pass_missing_data()?;
        Ok(self.create_first_pass_output())
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
        let event_list: CSVCMsg_GameEventList =
            Message::parse_from_bytes(&crate::first_pass::fallbackbytes::GAME_EVENT_LIST_FALLBACK_BYTES).unwrap();
        for event_desc in event_list.descriptors {
            self.ge_list.insert(event_desc.eventid(), event_desc);
        }
        Ok(())
    }
    pub fn create_first_pass_output(&self) -> FirstPassOutput {
        FirstPassOutput {
            header: self.header.clone(),
            fullpacket_offsets: self.fullpacket_offsets.clone(),
            settings: &self.settings,
            baselines: self.baselines.clone(),
            prop_controller: &self.prop_controller,
            cls_by_id: &self.cls_by_id.as_ref().unwrap(),
            qfmap: &self.qf_mapper,
            ge_list: &self.ge_list,
            // arc?
            wanted_players: self.wanted_players.clone(),
            wanted_ticks: self.wanted_ticks.clone(),
            string_tables: self.string_tables.clone(),
            stringtable_players: self.stringtable_players.clone(),
            added_temp_props: self.added_temp_props.clone(),
        }
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
    pub fn parse_game_event_list(&mut self, bytes: &[u8]) -> Result<AHashMap<i32, Descriptor_t>, DemoParserError> {
        let event_list: CSVCMsg_GameEventList = Message::parse_from_bytes(bytes).unwrap();
        let mut hm: AHashMap<i32, Descriptor_t> = AHashMap::default();
        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        Ok(hm)
    }
    pub fn parse_full_packet(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let full_packet: CDemoFullPacket = Message::parse_from_bytes(bytes).unwrap();
        for item in &full_packet.string_table.tables {
            if item.table_name.as_ref().unwrap() == "instancebaseline" {
                for i in &item.items {
                    let k = i.str().parse::<u32>().unwrap_or(999999);
                    self.baselines.insert(k, i.data.as_ref().unwrap().clone());
                }
            }
            if item.table_name == Some("userinfo".to_string()) {
                for i in &item.items {
                    if let Ok(player) = parse_userinfo(&i.data()) {
                        if player.steamid != 0 {
                            self.stringtable_players.insert(player.steamid, player);
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
        let mut p = ProtoPacketParser {
            bytes: bytes,
            start: 0,
            end: 0,
            ptr: 0,
        };
        p.read_proto_packet().unwrap();
        let mut bitreader = Bitreader::new(&bytes[p.start..p.end]);
        // Inner loop
        while bitreader.bits_remaining().unwrap() > 8 {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            let msg_bytes = bitreader.read_n_bytes(size as usize)?;

            let ok = match netmessage_type_from_int(msg_type as i32) {
                GE_Source1LegacyGameEventList => {
                    let hm = self.parse_game_event_list(&msg_bytes)?;
                    self.ge_list = hm;
                    self.ge_list_set = true;
                    Ok(())
                }
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
        let _no_clue_what_this_is = i32::from_le_bytes(bytes[12..].try_into().unwrap());
        Ok(())
    }

    pub fn parse_class_info(&mut self, bytes: &[u8], sendtables: CDemoSendTables) -> Result<(), DemoParserError> {
        let (mut serializers, qf_mapper, p) = self.parse_sendtable(sendtables);
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();
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

            cls_by_id[cls_id as usize] = Class {
                class_id: cls_id,
                name: network_name.to_string(),
                serializer: serializers.remove(network_name).unwrap(), // [network_name].clone(),
            }
        }
        self.cls_by_id = Some(Arc::new(cls_by_id));
        self.qf_mapper = qf_mapper;
        self.prop_controller = p;
        return Ok(());
    }
}
