use crate::first_pass::parser::HEADER_ENDS_AT_BYTE;
use crate::first_pass::parser_settings::FirstPassParser;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::read_bits::read_varint;
use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::stringtables::parse_userinfo;
use crate::maps::demo_cmd_type_from_int;
use crate::maps::netmessage_type_from_int;
use crate::maps::NetmessageType::*;
use crate::second_pass::collect_data::ProjectileRecord;
use crate::second_pass::game_events::GameEvent;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::parser_settings::*;
use crate::second_pass::variants::PropColumn;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::demo::*;
use csgoproto::netmessages::*;
use csgoproto::networkbasetypes::CNETMsg_Tick;
use protobuf::Message;
use snap::raw::decompress_len;
use snap::raw::Decoder as SnapDecoder;
use EDemoCommands::*;

const OUTER_BUF_DEFAULT_LEN: usize = 400_000;
const INNER_BUF_DEFAULT_LEN: usize = 8192 * 15;

#[derive(Debug)]
pub struct SecondPassOutput {
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
    pub voice_data: Vec<CSVCMsg_VoiceData>,
}
impl<'a> SecondPassParser<'a> {
    pub fn start(&mut self, demo_bytes: &[u8]) -> Result<(), DemoParserError> {
        let started_at = self.ptr;
        // re-use these to avoid allocation
        let mut buf = vec![0_u8; INNER_BUF_DEFAULT_LEN];
        let mut buf2 = vec![0_u8; OUTER_BUF_DEFAULT_LEN];
        loop {
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
            let demo_cmd = demo_cmd_type_from_int(msg_type as i32)?;

            if demo_cmd == DEM_AnimationData || demo_cmd == DEM_SendTables || demo_cmd == DEM_StringTables {
                self.ptr += size as usize;
                continue;
            }

            let input = &demo_bytes[self.ptr..self.ptr + size as usize];
            self.ptr += size as usize;
            let bytes = match is_compressed {
                true => {
                    FirstPassParser::resize_if_needed(&mut buf2, decompress_len(input))?;
                    match SnapDecoder::new().decompress(input, &mut buf2) {
                        Ok(idx) => &buf2[..idx],
                        Err(e) => return Err(DemoParserError::DecompressionFailure(format!("{}", e))),
                    }
                }
                false => input,
            };

            let ok = match demo_cmd {
                DEM_SignonPacket => self.parse_packet(&bytes, &mut buf),
                DEM_Packet => self.parse_packet(&bytes, &mut buf),
                DEM_FullPacket => {
                    match self.parse_all_packets {
                        true => {
                            self.parse_full_packet(&bytes, false)?;
                        }
                        false => {
                            if self.fullpackets_parsed == 0 && started_at != HEADER_ENDS_AT_BYTE {
                                self.parse_full_packet(&bytes, true)?;
                                self.fullpackets_parsed += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                DEM_Stop => break,
                _ => Ok(()),
            };
            ok?;
            self.collect_entities();
        }
        Ok(())
    }

    pub fn parse_packet(&mut self, bytes: &[u8], buf: &mut Vec<u8>) -> Result<(), DemoParserError> {
        let msg: CDemoPacket = match Message::parse_from_bytes(bytes) {
            Err(_) => return Err(DemoParserError::MalformedMessage),
            Ok(msg) => msg,
        };
        let mut bitreader = Bitreader::new(msg.data());
        let mut wrong_order_events = vec![];

        while bitreader.bits_remaining().unwrap_or(0) > 8 {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            if buf.len() < size as usize {
                buf.resize(size as usize, 0)
            }
            bitreader.read_n_bytes_mut(size as usize, buf)?;
            let msg_bytes = &buf[..size as usize];
            let ok = match netmessage_type_from_int(msg_type as i32) {
                svc_PacketEntities => self.parse_packet_ents(msg_bytes, false),
                svc_CreateStringTable => self.parse_create_stringtable(msg_bytes),
                svc_UpdateStringTable => self.update_string_table(msg_bytes),
                svc_ServerInfo => self.parse_server_info(msg_bytes),
                CS_UM_SendPlayerItemDrops => self.parse_item_drops(msg_bytes),
                CS_UM_EndOfMatchAllPlayersData => self.parse_player_end_msg(msg_bytes),
                UM_SayText2 => self.parse_chat_messages(msg_bytes),
                net_SetConVar => self.create_custom_event_parse_convars(msg_bytes),
                CS_UM_PlayerStatsUpdate => self.parse_player_stats_update(msg_bytes),
                CS_UM_ServerRankUpdate => self.create_custom_event_rank_update(msg_bytes),
                net_Tick => self.parse_net_tick(msg_bytes),
                svc_ClearAllStringTables => self.clear_stringtables(),
                svc_VoiceData => {
                    if let Ok(m) = Message::parse_from_bytes(msg_bytes) {
                        self.voice_data.push(m);
                    }
                    Ok(())
                }
                GE_Source1LegacyGameEvent => match self.parse_event(msg_bytes) {
                    Ok(Some(event)) => {
                        wrong_order_events.push(event);
                        Ok(())
                    }
                    Ok(None) => Ok(()),
                    Err(e) => return Err(e),
                },
                _ => Ok(()),
            };
            ok?
        }
        if !wrong_order_events.is_empty() {
            self.resolve_wrong_order_event(&mut wrong_order_events)?;
        }
        Ok(())
    }
    pub fn parse_net_tick(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let message: CNETMsg_Tick = match Message::parse_from_bytes(&bytes) {
            Ok(message) => message,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        self.net_tick = message.tick();
        Ok(())
    }
    pub fn parse_full_packet(&mut self, bytes: &[u8], should_parse_entities: bool) -> Result<(), DemoParserError> {
        self.string_tables = vec![];

        let full_packet: CDemoFullPacket = match Message::parse_from_bytes(bytes) {
            Err(_e) => return Err(DemoParserError::MalformedMessage),
            Ok(p) => p,
        };

        for item in &full_packet.string_table.tables {
            if item.table_name == Some("instancebaseline".to_string()) {
                for i in &item.items {
                    let k = i.str().parse::<u32>().unwrap_or(u32::MAX);
                    self.baselines.insert(k, i.data().to_vec());
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
        let packet = match full_packet.packet.0 {
            Some(packet) => packet,
            None => return Err(DemoParserError::MalformedMessage),
        };
        let mut bitreader = Bitreader::new(packet.data());
        let mut buf = vec![0; 5_00_000];
        // Inner loop
        while bitreader.bits_remaining().unwrap_or(0) > 8 {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            if buf.len() < size as usize {
                buf.resize(size as usize, 0)
            }
            bitreader.read_n_bytes_mut(size as usize, &mut buf)?;
            let msg_bytes = &buf[..size as usize];
            let ok = match netmessage_type_from_int(msg_type as i32) {
                svc_PacketEntities => {
                    if should_parse_entities {
                        self.parse_packet_ents(&msg_bytes, true)?;
                    }
                    Ok(())
                }
                svc_CreateStringTable => self.parse_create_stringtable(&msg_bytes),
                svc_UpdateStringTable => self.update_string_table(&msg_bytes),
                CS_UM_SendPlayerItemDrops => self.parse_item_drops(&msg_bytes),
                CS_UM_EndOfMatchAllPlayersData => self.parse_player_end_msg(&msg_bytes),
                UM_SayText2 => self.parse_chat_messages(&msg_bytes),
                net_SetConVar => self.create_custom_event_parse_convars(&msg_bytes),
                CS_UM_PlayerStatsUpdate => self.parse_player_stats_update(&msg_bytes),
                svc_ServerInfo => self.parse_server_info(&msg_bytes),
                net_Tick => self.parse_net_tick(&msg_bytes),
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
    pub fn parse_server_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let server_info: CSVCMsg_ServerInfo = match Message::parse_from_bytes(bytes) {
            Err(_e) => return Err(DemoParserError::MalformedMessage),
            Ok(p) => p,
        };
        let class_count = server_info.max_classes();
        self.cls_bits = Some((class_count as f32 + 1.).log2().ceil() as u32);
        Ok(())
    }
    pub fn parse_user_command_cmd(&mut self, _data: &[u8]) -> Result<(), DemoParserError> {
        // Only in pov demos. Maybe implement sometime. Includes buttons etc.
        Ok(())
    }
}
