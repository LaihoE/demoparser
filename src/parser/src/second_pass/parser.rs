use crate::first_pass::parser::Frame;
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
use crate::second_pass::entities::Entity;
use crate::second_pass::game_events::EventField;
use crate::second_pass::game_events::GameEvent;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::parser_settings::*;
use crate::second_pass::variants::PropColumn;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::cs_usercmd::CSGOUserCmdPB;
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
            let frame = self.read_frame(demo_bytes)?;
            if frame.demo_cmd == DEM_AnimationData || frame.demo_cmd == DEM_SendTables || frame.demo_cmd == DEM_StringTables {
                self.ptr += frame.size as usize;
                continue;
            }

            let bytes = self.slice_packet_bytes(demo_bytes, frame.size)?;
            let bytes = self.decompress_if_needed(&mut buf, bytes, &frame)?;
            self.ptr += frame.size;

            let ok = match frame.demo_cmd {
                DEM_SignonPacket => self.parse_packet(&bytes, &mut buf2),
                DEM_Packet => self.parse_packet(&bytes, &mut buf2),
                DEM_Stop => break,
                DEM_FullPacket => {
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
    fn parse_full_packet_and_break_if_needed(
        &mut self,
        bytes: &[u8],
        buf: &mut Vec<u8>,
        started_at: usize,
    ) -> Result<bool, DemoParserError> {
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
            frame_starts_at: frame_starts_at,
            is_compressed: is_compressed,
            demo_cmd: demo_cmd,
            tick: self.tick,
        })
    }
    fn slice_packet_bytes(&mut self, demo_bytes: &'a [u8], frame_size: usize) -> Result<&'a [u8], DemoParserError> {
        if self.ptr + frame_size as usize >= demo_bytes.len() {
            return Err(DemoParserError::MalformedMessage);
        }
        Ok(&demo_bytes[self.ptr..self.ptr + frame_size])
    }
    fn decompress_if_needed<'b>(
        &mut self,
        buf: &'b mut Vec<u8>,
        possibly_uncompressed_bytes: &'b [u8],
        frame: &Frame,
    ) -> Result<&'b [u8], DemoParserError> {
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
        let msg: CDemoPacket = match Message::parse_from_bytes(bytes) {
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
            let ok = match netmessage_type_from_int(msg_type as i32) {
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
        let m: CSVCMsg_UserCommands = Message::parse_from_bytes(bytes).unwrap();

        for cmd in m.commands {
            let user_cmd = CSGOUserCmdPB::parse_from_bytes(cmd.data()).unwrap();
            let mut fields = vec![];
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::F32(user_cmd.base.viewangles.x())),
                name: "X".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::F32(user_cmd.base.viewangles.y())),
                name: "Y".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::F32(user_cmd.base.viewangles.z())),
                name: "Z".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::F32(user_cmd.base.forwardmove())),
                name: "forward_move".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::I32(user_cmd.base.impulse())),
                name: "impulse".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::I32(user_cmd.base.mousedx())),
                name: "mouse_x".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::I32(user_cmd.base.mousedy())),
                name: "mouse_y".to_string(),
            });

            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::U64(
                    user_cmd.base.buttons_pb.buttonstate1(),
                )),
                name: "button_state_1".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::U64(
                    user_cmd.base.buttons_pb.buttonstate2(),
                )),
                name: "button_state_2".to_string(),
            });
            fields.push(EventField {
                data: Some(crate::second_pass::variants::Variant::U64(
                    user_cmd.base.buttons_pb.buttonstate3(),
                )),
                name: "button_state_3".to_string(),
            });

            self.game_events.push(GameEvent {
                name: "user_cmd".to_string(),
                fields: fields,
                tick: self.tick,
            })
        }
        Ok(())
    }

    pub fn parse_voice_data(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if let Ok(m) = Message::parse_from_bytes(bytes) {
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
        let message: CNETMsg_Tick = match Message::parse_from_bytes(&bytes) {
            Ok(message) => message,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        self.net_tick = message.tick();
        Ok(())
    }

    pub fn parse_full_packet(
        &mut self,
        bytes: &[u8],
        should_parse_entities: bool,
        buf: &mut Vec<u8>,
    ) -> Result<(), DemoParserError> {
        self.string_tables = vec![];
        let full_packet: CDemoFullPacket = match Message::parse_from_bytes(bytes) {
            Err(_e) => return Err(DemoParserError::MalformedMessage),
            Ok(p) => p,
        };
        self.parse_full_packet_stringtables(&full_packet);
        let mut bitreader = Bitreader::new(full_packet.packet.data());
        self.parse_packet_from_bitreader(&mut bitreader, buf, should_parse_entities, true)
    }

    pub fn parse_full_packet_stringtables(&mut self, full_packet: &CDemoFullPacket) {
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
                            self.stringtable_players.insert(player.userid, player);
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
