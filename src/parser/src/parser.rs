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
use crate::read_bytes::read_varint;
use crate::read_bytes::ProtoPacketParser;
use crate::stringtables::parse_userinfo;
use crate::stringtables::StringTable;
use crate::stringtables::UserInfo;
use crate::variants::PropColumn;
use crate::{other_netmessages::Class, read_bits::DemoParserError};
use ahash::AHashMap;
use ahash::AHashSet;
use bitter::BitReader;
use csgoproto::demo::CDemoFullPacket;
use csgoproto::demo::EDemoCommands::*;
use csgoproto::demo::{CDemoClassInfo, CDemoFileHeader, CDemoSendTables};
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use netmessage_types::NetmessageType::*;
use protobuf::Message;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use snap::raw::decompress_len;
use snap::raw::Decoder as SnapDecoder;
use std::collections::BTreeMap;
use std::sync::Arc;

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
    pub prop_info: PropController,
    pub projectiles: Vec<ProjectileRecord>,
    pub ptr: usize,
}

impl<'a> Parser<'a> {
    pub fn parse_demo(&mut self, outer_bytes: &[u8]) -> Result<DemoOutput, DemoParserError> {
        Parser::handle_short_header(outer_bytes.len(), &outer_bytes[..16])?;
        self.ptr = 16;
        let mut sendtable = None;
        let mut buf = vec![0_u8; 1_000_000];
        let mut biggest = 0;
        loop {
            let frame_starts_at = self.ptr;

            let cmd = read_varint(outer_bytes, &mut self.ptr)?;
            let tick = read_varint(outer_bytes, &mut self.ptr)?;
            let size = read_varint(outer_bytes, &mut self.ptr)?;

            biggest = u32::max(size, biggest);

            self.tick = tick as i32;
            // Safety check
            if self.ptr + size as usize >= outer_bytes.len() {
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
            let input = &outer_bytes[self.ptr..self.ptr + size as usize];
            Parser::resize_if_needed(&mut buf, decompress_len(input))?;

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
        self.check_needed()?;

        if self.is_multithreadable {
            self.parse_demo_multithread(outer_bytes)
        } else {
            self.parse_demo_single_thread(outer_bytes)
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
    fn check_needed(&mut self) -> Result<(), DemoParserError> {
        if !self.fullpacket_offsets.contains(&16) {
            self.fullpacket_offsets.push(16);
        }
        if self.ge_list.is_empty() {
            self.parse_fallback_event_list()?;
        }
        Ok(())
    }

    fn parse_demo_single_thread(&mut self, outer_bytes: &[u8]) -> Result<DemoOutput, DemoParserError> {
        let input = self.create_parser_thread_input(16, true);
        let mut parser = ParserThread::new(input).unwrap();
        parser.start(outer_bytes)?;
        let x = parser.create_output();
        for prop in &self.added_temp_props {
            self.wanted_player_props.retain(|x| x != prop);
            self.prop_controller.prop_infos.retain(|x| &x.prop_name != prop);
        }
        return Ok(self.combine_thread_outputs(&mut vec![x]));
    }
    fn parse_demo_multithread(&mut self, outer_bytes: &[u8]) -> Result<DemoOutput, DemoParserError> {
        let outputs: Vec<Result<DemoOutput, DemoParserError>> = self
            .fullpacket_offsets
            .par_iter()
            .map(|offset| {
                let input = self.create_parser_thread_input(*offset, false);
                let mut parser = ParserThread::new(input).unwrap();
                parser.start(outer_bytes)?;
                Ok(parser.create_output())
            })
            .collect();

        // check for errors
        let mut ok = vec![];
        for result in outputs {
            match result {
                Err(e) => return Err(e),
                Ok(r) => ok.push(r),
            };
        }
        for prop in &self.added_temp_props {
            self.wanted_player_props.retain(|x| x != prop);
            self.prop_controller.prop_infos.retain(|x| &x.prop_name != prop);
        }
        Ok(self.combine_thread_outputs(&mut ok))
    }

    // fn parse_stringtables_cmd(bytes: &[u8]) -> Result<(), DemoParserError> {}
    pub fn create_parser_thread_input(&self, offset: usize, parse_all: bool) -> ParserThreadInput {
        ParserThreadInput {
            offset: offset,
            settings: &self.settings,
            baselines: self.baselines.clone(),
            prop_controller: &self.prop_controller,
            cls_by_id: &self.cls_by_id.as_ref().unwrap(),
            qfmap: &self.qf_mapper,
            ge_list: &self.ge_list,
            parse_all_packets: parse_all,
            // arc?
            wanted_ticks: self.wanted_ticks.clone(),
            string_tables: self.string_tables.clone(),
            stringtable_players: self.stringtable_players.clone(),
        }
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
    fn combine_dfs(&self, v: &mut Vec<AHashMap<u32, PropColumn>>) -> AHashMap<u32, PropColumn> {
        let mut big: AHashMap<u32, PropColumn> = AHashMap::default();
        if v.len() == 1 {
            return v.remove(0);
        }
        for part_df in v {
            for (k, v) in part_df {
                if big.contains_key(k) {
                    big.get_mut(k).unwrap().extend_from(v);
                } else {
                    big.insert(*k, v.clone());
                }
            }
        }
        big
    }
}

impl<'a> Parser<'a> {
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
        while bitreader.reader.has_bits_remaining(8) {
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
        let (mut serializers, qf_mapper, p) = self.parse_sendtable(sendtables)?;

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
        self.cls_by_id = Some(Arc::new(cls_by_id));
        self.qf_mapper = qf_mapper;
        self.prop_controller = p;
        return Ok(());
    }
}
pub struct ParserThreadInput<'a> {
    pub offset: usize,
    pub settings: &'a ParserInputs<'a>,
    pub baselines: AHashMap<u32, Vec<u8>>,
    pub prop_controller: &'a PropController,
    pub cls_by_id: &'a AHashMap<u32, Class>,
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
