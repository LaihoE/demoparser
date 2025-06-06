use crate::first_pass::parser::Frame;
use crate::first_pass::parser::HEADER_ENDS_AT_BYTE;
use crate::first_pass::parser_settings::FirstPassParser;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::prop_controller::*;
use crate::first_pass::read_bits::read_varint;
use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::stringtables::parse_userinfo;
use crate::maps::demo_cmd_type_from_int;
use crate::second_pass::collect_data::ProjectileRecord;
use crate::second_pass::entities::Entity;
use crate::second_pass::game_events::GameEvent;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::parser_settings::*;
use crate::second_pass::variants::PropColumn;
use crate::second_pass::variants::Variant;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::message_type::NetMessageType::{self, *};
use csgoproto::CDemoFullPacket;
use csgoproto::CDemoPacket;
use csgoproto::CnetMsgTick;
use csgoproto::CsgoUserCmdPb;
use csgoproto::CsvcMsgServerInfo;
use csgoproto::CsvcMsgUserCommands;
use csgoproto::CsvcMsgVoiceData;
use csgoproto::EDemoCommands::*;
use prost::Message;
use snap::raw::decompress_len;
use snap::raw::Decoder as SnapDecoder;

use super::variants::InputHistory;

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
    pub uniq_prop_names: AHashSet<String>,
    pub prop_info: PropController,
    pub projectiles: Vec<ProjectileRecord>,
    pub ptr: usize,
    pub voice_data: Vec<CsvcMsgVoiceData>,
    pub df_per_player: AHashMap<u64, AHashMap<u32, PropColumn>>,
    pub entities: Vec<Option<Entity>>,
    pub last_tick: i32,
}
impl<'a> SecondPassParser<'a> {
    pub fn start(&mut self, demo_bytes: &'a [u8]) -> Result<(), DemoParserError> {
        let started_at = self.ptr;
        // re-use these to avoid allocation
        let mut buf = vec![0_u8; INNER_BUF_DEFAULT_LEN];
        let mut buf2 = vec![0_u8; OUTER_BUF_DEFAULT_LEN];
        loop {
            if demo_bytes.len() < self.ptr { break; }
            let frame = self.read_frame(demo_bytes)?;
            if frame.demo_cmd == DemAnimationData || frame.demo_cmd == DemSendTables || frame.demo_cmd == DemStringTables {
                self.ptr += frame.size as usize;
                continue;
            }
            let bytes = match self.slice_packet_bytes(demo_bytes, frame.size) {
                Ok(b) => b,
                Err(_) => {
                    self.ptr += frame.size;
                    continue;
                }
            };
            let bytes = self.decompress_if_needed(&mut buf, bytes, &frame)?;
            self.ptr += frame.size;

            let ok = match frame.demo_cmd {
                DemSignonPacket => self.parse_packet(&bytes, &mut buf2),
                DemPacket => self.parse_packet(&bytes, &mut buf2),
                DemStop => break,
                DemUserCmd => Ok(()),
                DemFullPacket => {
                    if self.parse_full_packet_and_break_if_needed(&bytes, &mut buf2, started_at)? {
                        break;
                    }
                    Ok(())
                }
                _ => Ok(()),
            };
            ok?;
        }
        Ok(())
    }
    fn parse_full_packet_and_break_if_needed(&mut self, bytes: &[u8], buf: &mut Vec<u8>, started_at: usize) -> Result<bool, DemoParserError> {
        if let Some(start_end_offset) = self.start_end_offset {
            if self.ptr > start_end_offset.end {
                return Ok(true);
            } else {
                self.parse_full_packet(&bytes, true, buf)?;
                return Ok(false);
            }
        }
        match self.parse_all_packets {
            true => {
                self.parse_full_packet(&bytes, false, buf)?;
            }
            false => {
                if self.fullpackets_parsed == 0 && started_at != HEADER_ENDS_AT_BYTE {
                    self.parse_full_packet(&bytes, true, buf)?;
                    self.fullpackets_parsed += 1;
                } else {
                    return Ok(true);
                }
            }
        }
        return Ok(false);
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

    pub fn parse_packet(&mut self, bytes: &[u8], buf: &mut Vec<u8>) -> Result<(), DemoParserError> {
        let msg = match CDemoPacket::decode(bytes) {
            Err(_) => return Err(DemoParserError::MalformedMessage),
            Ok(msg) => msg,
        };
        let mut bitreader = Bitreader::new(msg.data());
        self.parse_packet_from_bitreader(&mut bitreader, buf, true, false)?;
        Ok(())
    }

    pub fn parse_packet_from_bitreader(
        &mut self,
        bitreader: &mut Bitreader,
        buf: &mut Vec<u8>,
        should_parse_entities: bool,
        is_fullpacket: bool,
    ) -> Result<(), DemoParserError> {
        let mut wrong_order_events = vec![];

        while bitreader.bits_remaining().unwrap_or(0) > 8 {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            if buf.len() < size as usize {
                buf.resize(size as usize, 0)
            }
            bitreader.read_n_bytes_mut(size as usize, buf)?;
            let msg_bytes = &buf[..size as usize];
            let ok = match NetMessageType::from(msg_type as i32) {
                svc_PacketEntities => {
                    if should_parse_entities {
                        self.parse_packet_ents(&msg_bytes, is_fullpacket)?;
                        if !is_fullpacket {
                            self.collect_entities();
                        }
                    }
                    Ok(())
                }
                svc_CreateStringTable => self.parse_create_stringtable(msg_bytes),
                svc_UpdateStringTable => self.update_string_table(msg_bytes),
                svc_ServerInfo => self.parse_server_info(msg_bytes),
                CS_UM_SendPlayerItemDrops => self.parse_item_drops(msg_bytes),
                CS_UM_EndOfMatchAllPlayersData => self.parse_player_end_msg(msg_bytes),
                UM_SayText2 => self.create_custom_event_chat_message(msg_bytes),
                UM_SayText => self.create_custom_event_server_message(msg_bytes),
                net_SetConVar => self.create_custom_event_parse_convars(msg_bytes),
                CS_UM_PlayerStatsUpdate => self.parse_player_stats_update(msg_bytes),
                CS_UM_ServerRankUpdate => self.create_custom_event_rank_update(msg_bytes),
                net_Tick => self.parse_net_tick(msg_bytes),
                svc_ClearAllStringTables => self.clear_stringtables(),
                svc_VoiceData => self.parse_voice_data(msg_bytes),
                GE_Source1LegacyGameEvent => self.parse_game_event(msg_bytes, &mut wrong_order_events),
                svc_UserCmds => self.parse_user_cmd(msg_bytes),
                _ => Ok(()),
            };
            ok?
        }
        if !wrong_order_events.is_empty() {
            self.resolve_wrong_order_event(&mut wrong_order_events)?;
        }
        Ok(())
    }
    pub fn parse_user_cmd(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        // We simply inject the values into the entities as if they came from packet_ents like any other val.

        // This method is quite expensive so early exit it if not needed.
        if !self.parse_usercmd {
            return Ok(());
        }

        let msg = match CsvcMsgUserCommands::decode(bytes) {
            Ok(m) => m,
            _ => return Ok(()),
        };
        for cmd in msg.commands {
            let user_cmd = match CsgoUserCmdPb::decode(cmd.data()) {
                Ok(m) => m,
                _ => return Ok(()),
            };

            if let Some(base) = user_cmd.base {
                let entity_id = base.pawn_entity_handle() & 0x7FF;
                if let Some(Some(ent)) = self.entities.get_mut(entity_id as usize) {
                    let mut history = vec![];
                    for input in user_cmd.input_history {
                        let ih = InputHistory {
                            player_tick_count: input.player_tick_count(),
                            player_tick_fraction: input.player_tick_fraction(),
                            render_tick_count: input.render_tick_count(),
                            render_tick_fraction: input.render_tick_fraction(),
                            x: input.view_angles.expect("CsgoInputHistoryEntryPb has no CMsgQAngle").x(),
                            y: input.view_angles.expect("CsgoInputHistoryEntryPb has no CMsgQAngle").y(),
                            z: input.view_angles.expect("CsgoInputHistoryEntryPb has no CMsgQAngle").z(),
                        };
                        history.push(ih);
                    }
                    ent.props.insert(USERCMD_INPUT_HISTORY_BASEID, Variant::InputHistory(history));
                    ent.props.insert(USERCMD_LEFTMOVE, Variant::F32(base.leftmove()));
                    ent.props.insert(USERCMD_FORWARDMOVE, Variant::F32(base.forwardmove()));
                    ent.props.insert(USERCMD_IMPULSE, Variant::I32(base.impulse()));
                    ent.props.insert(USERCMD_MOUSE_DX, Variant::I32(base.mousedx()));
                    ent.props.insert(USERCMD_MOUSE_DY, Variant::I32(base.mousedy()));
                    if let Some(viewangles) = base.viewangles {
                        ent.props.insert(USERCMD_VIEWANGLE_X, Variant::F32(viewangles.x()));
                        ent.props.insert(USERCMD_VIEWANGLE_Y, Variant::F32(viewangles.y()));
                        ent.props.insert(USERCMD_VIEWANGLE_Z, Variant::F32(viewangles.z()));
                    }
                    if let Some(buttons_pb) = base.buttons_pb {
                        ent.props.insert(USERCMD_BUTTONSTATE_1, Variant::U64(buttons_pb.buttonstate1()));
                        ent.props.insert(USERCMD_BUTTONSTATE_2, Variant::U64(buttons_pb.buttonstate2()));
                        ent.props.insert(USERCMD_BUTTONSTATE_3, Variant::U64(buttons_pb.buttonstate3()));
                    }
                    ent.props
                        .insert(USERCMD_CONSUMED_SERVER_ANGLE_CHANGES, Variant::U32(base.consumed_server_angle_changes()));
                }
            }
        }
        Ok(())
    }

    pub fn parse_voice_data(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if let Ok(m) = CsvcMsgVoiceData::decode(bytes) {
            self.voice_data.push(m);
        }
        Ok(())
    }
    pub fn parse_game_event(&mut self, bytes: &[u8], wrong_order_events: &mut Vec<GameEvent>) -> Result<(), DemoParserError> {
        match self.parse_event(bytes) {
            Ok(Some(event)) => {
                wrong_order_events.push(event);
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => return Err(e),
        }
    }

    pub fn parse_net_tick(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let message = match CnetMsgTick::decode(bytes) {
            Ok(message) => message,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        self.net_tick = message.tick();
        Ok(())
    }

    pub fn parse_full_packet(&mut self, bytes: &[u8], should_parse_entities: bool, buf: &mut Vec<u8>) -> Result<(), DemoParserError> {
        self.string_tables = vec![];
        let full_packet = match CDemoFullPacket::decode(bytes) {
            Err(_e) => return Err(DemoParserError::MalformedMessage),
            Ok(p) => p,
        };
        self.parse_full_packet_stringtables(&full_packet);
        if let Some(packet) = full_packet.packet {
            let mut bitreader = Bitreader::new(packet.data());
            self.parse_packet_from_bitreader(&mut bitreader, buf, should_parse_entities, true)
        } else {
            Ok(())
        }
    }

    pub fn parse_full_packet_stringtables(&mut self, full_packet: &CDemoFullPacket) {
        if let Some(string_table) = &full_packet.string_table {
            for item in &string_table.tables {
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
                                self.stringtable_players.insert(player.userid, player);
                            }
                        }
                    }
                }
            }
        }
    }
    fn clear_stringtables(&mut self) -> Result<(), DemoParserError> {
        self.string_tables = vec![];
        Ok(())
    }
    pub fn parse_server_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let server_info = match CsvcMsgServerInfo::decode(bytes) {
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
