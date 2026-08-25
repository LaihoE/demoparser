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
use csgoproto::CMsgServerUserCmd;
use csgoproto::CnetMsgTick;
use csgoproto::CsgoUserCmdPb;
use csgoproto::CsvcMsgServerInfo;
use csgoproto::CsvcMsgUserCommands;
use csgoproto::CsvcMsgVoiceData;
use csgoproto::EDemoCommands::*;
use prost::Message;
use snap::raw::decompress_len;
use snap::raw::Decoder as SnapDecoder;

use super::usercmd_delta::{apply_delta_with_error, DeltaDecodeError, RepeatedDecodeError};
use super::variants::{InputHistory, InterpolationInfo, UserCmdSubtickMove};

const OUTER_BUF_DEFAULT_LEN: usize = 400_000;
const INNER_BUF_DEFAULT_LEN: usize = 8192 * 15;

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn record_repeated_failure(stats: &mut UserCmdDecodeStats, reason: RepeatedDecodeError) {
    match reason {
        RepeatedDecodeError::Malformed => stats.delta_repeated_malformed += 1,
        RepeatedDecodeError::Truncated => stats.delta_repeated_truncated += 1,
        RepeatedDecodeError::InvalidIndex => stats.delta_repeated_invalid_index += 1,
        RepeatedDecodeError::IndexOutOfBounds => stats.delta_repeated_out_of_bounds += 1,
        RepeatedDecodeError::InvalidMessage => stats.delta_repeated_invalid_message += 1,
        RepeatedDecodeError::InvalidNestedMessage => stats.delta_repeated_invalid_nested += 1,
    }
}

// --- env-gated phase profiling (CS2_PROF=1) ---------------------------------
#[inline]
pub(crate) fn prof_on() -> bool {
    static PROF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PROF.get_or_init(|| std::env::var("CS2_PROF").is_ok())
}
thread_local! {
    static PROF_ENTS_NS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PROF_COLLECT_NS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(crate) static PROF_PATHS_NS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(crate) static PROF_DECODE_NS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

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
    /// Live player roster from CCSPlayerController entities (final per-player state).
    /// Populated even when CCSUsrMsg_EndOfMatchAllPlayersData is absent (community/casual
    /// demos), where `player_md` ends up empty. Use as a fallback when `player_md` is empty.
    pub roster: Vec<PlayerEndMetaData>,
    pub game_events_counter: AHashSet<String>,
    pub uniq_prop_names: AHashSet<String>,
    pub prop_info: PropController,
    pub projectiles: Vec<ProjectileRecord>,
    pub ptr: usize,
    pub voice_data: Vec<(i32, CsvcMsgVoiceData)>,
    pub df_per_player: AHashMap<u64, AHashMap<u32, PropColumn>>,
    pub entities: Vec<Option<Entity>>,
    pub last_tick: i32,
    pub usercmd_stats: UserCmdDecodeStats,
}
impl<'a> SecondPassParser<'a> {
    pub fn start(&mut self, demo_bytes: &'a [u8]) -> Result<(), DemoParserError> {
        if prof_on() {
            PROF_ENTS_NS.with(|c| c.set(0));
            PROF_COLLECT_NS.with(|c| c.set(0));
            PROF_PATHS_NS.with(|c| c.set(0));
            PROF_DECODE_NS.with(|c| c.set(0));
        }
        let started_at = self.ptr;
        // re-use these to avoid allocation
        let mut buf = vec![0_u8; INNER_BUF_DEFAULT_LEN];
        let mut buf2 = vec![0_u8; OUTER_BUF_DEFAULT_LEN];

        loop {
            // Need at least a few bytes to read frame header (3 varints, minimum 1 byte each)
            if self.ptr + 3 > demo_bytes.len() {
                break;
            }
            let frame = match self.read_frame(demo_bytes) {
                Ok(f) => f,
                Err(DemoParserError::OutOfBytesError) => break,
                Err(e) => return Err(e),
            };
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
            #[cfg(test)]
            if self
                .usercmd_capture_counts
                .as_ref()
                .is_some_and(|counts| self.usercmd_records.len() == counts.values().sum::<usize>())
            {
                break;
            }
        }
        if prof_on() {
            let ents = PROF_ENTS_NS.with(|c| c.get());
            let coll = PROF_COLLECT_NS.with(|c| c.get());
            let paths = PROF_PATHS_NS.with(|c| c.get());
            let dec = PROF_DECODE_NS.with(|c| c.get());
            eprintln!("[prof] parse_packet_ents: {:.3}s | collect_*: {:.3}s", ents as f64 / 1e9, coll as f64 / 1e9);
            eprintln!(
                "[prof]   within ents: parse_paths {:.3}s | decode_entity_update {:.3}s",
                paths as f64 / 1e9,
                dec as f64 / 1e9
            );
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
                        let _pt = prof_on().then(std::time::Instant::now);
                        self.parse_packet_ents(msg_bytes, is_fullpacket)?;
                        if let Some(t) = _pt {
                            PROF_ENTS_NS.with(|c| c.set(c.get() + t.elapsed().as_nanos() as u64));
                        }
                        if !is_fullpacket {
                            let _ct = prof_on().then(std::time::Instant::now);
                            self.collect_entities();
                            if let Some(t) = _ct {
                                PROF_COLLECT_NS.with(|c| c.set(c.get() + t.elapsed().as_nanos() as u64));
                            }
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
                svc_UserCmds => self.parse_user_cmd(msg_bytes, is_fullpacket),
                GE_FireBulletsId => self.create_custom_event_fire_bullets(msg_bytes),
                GE_PlayerBulletHitId => self.create_custom_event_player_bullet_hit(msg_bytes),
                _ => Ok(()),
            };
            ok?
        }
        if !wrong_order_events.is_empty() {
            self.resolve_wrong_order_event(&mut wrong_order_events)?;
        }
        Ok(())
    }
    pub fn parse_user_cmd(&mut self, bytes: &[u8], is_fullpacket: bool) -> Result<(), DemoParserError> {
        // We simply inject the values into the entities as if they came from packet_ents like any other val.

        // This method is quite expensive so early exit it if not needed.
        if !self.parse_usercmd {
            return Ok(());
        }

        let msg = match CsvcMsgUserCommands::decode(bytes) {
            Ok(m) => m,
            Err(error) => {
                #[cfg(test)]
                {
                    use std::sync::atomic::{AtomicUsize, Ordering};
                    static PRINTED_ERRORS: AtomicUsize = AtomicUsize::new(0);
                    if PRINTED_ERRORS.fetch_add(1, Ordering::Relaxed) < 20 {
                        eprintln!(
                            "CsvcMsgUserCommands decode failed at parser tick {} bytes={} error={:?}",
                            self.tick,
                            bytes.len(),
                            error
                        );
                    }
                }
                return Ok(());
            }
        };
        if is_fullpacket {
            // A DemFullPacket is a seek/checkpoint snapshot. Its embedded
            // usercmds are not transports delivered to the client command
            // handler and would otherwise duplicate the following packet.
            return Ok(());
        }
        self.process_user_commands(msg.commands)
    }

    fn process_user_commands(&mut self, commands: Vec<CMsgServerUserCmd>) -> Result<(), DemoParserError> {
        for cmd in commands {
            let player_slot = cmd.player_slot();
            #[cfg(test)]
            if let Some(sink) = self.usercmd_transport_sink.as_mut() {
                let mut server_cmd_has_bits = 0_u32;
                if cmd.data.is_some() {
                    server_cmd_has_bits |= 1;
                }
                if cmd.delta_data.is_some() {
                    server_cmd_has_bits |= 2;
                }
                sink(UserCmdTransportTestRecord {
                    player_slot,
                    command_number: cmd.cmd_number.unwrap_or_default(),
                    server_tick_executed: cmd.server_tick_executed(),
                    client_tick: cmd.client_tick(),
                    server_cmd_has_bits,
                });
            }
            if player_slot < 0 {
                continue;
            }
            let Some(command_number) = cmd.cmd_number else {
                continue;
            };
            let data = cmd.data.as_ref().filter(|data| !data.is_empty());
            let delta_data = cmd.delta_data.as_ref().filter(|data| !data.is_empty());
            let record_is_full = data.is_some();
            let mut next = None;
            #[cfg(test)]
            let mut test_baseline_command_number = None;
            #[cfg(test)]
            let mut test_baseline_source = None;

            if let Some(data) = data {
                self.usercmd_stats.full_data += 1;
                next = match CsgoUserCmdPb::decode(data.as_ref()) {
                    Ok(command) => Some(command),
                    Err(_) => {
                        self.usercmd_stats.full_decode_failures += 1;
                        continue;
                    }
                };
            } else if delta_data.is_some() {
                self.usercmd_stats.delta_data += 1;
                let state = self.usercmd_states.entry(player_slot).or_default();
                // client.dll passes the player's cached current command number
                // to the 150-slot ring lookup. The ring size determines only
                // the storage slot; it is not a fixed command-number delta.
                let Some(requested_baseline_number) = state.current_command_number else {
                    self.usercmd_stats.baseline_missing += 1;
                    continue;
                };
                if command_number < requested_baseline_number {
                    // A full packet can arrive before older usercmd transports
                    // in the demo file. client.dll never applies those older
                    // deltas against the future command now in its cache.
                    self.usercmd_stats.baseline_mismatch += 1;
                    continue;
                }
                if let Some((baseline, source)) = state.resolve_baseline(requested_baseline_number) {
                    next = Some(baseline.clone());
                    #[cfg(test)]
                    {
                        test_baseline_command_number = Some(requested_baseline_number);
                        test_baseline_source = Some(source);
                    }
                } else if state.ring.slot_command_number(requested_baseline_number).is_some() {
                    self.usercmd_stats.baseline_mismatch += 1;
                    continue;
                } else {
                    self.usercmd_stats.baseline_missing += 1;
                    continue;
                }
            } else {
                continue;
            }

            if let Some(delta_data) = delta_data {
                let mut last_error = None;
                next = next.as_ref().and_then(|baseline| {
                    match apply_delta_with_error(baseline, delta_data.as_ref()) {
                        Ok(command) => Some(command),
                        Err(error) => {
                            last_error = Some(error);
                            None
                        }
                    }
                });
                self.usercmd_stats.delta_applied += u64::from(next.is_some());
                if next.is_none() {
                    self.usercmd_stats.delta_decode_failures += 1;
                    match last_error.unwrap_or(DeltaDecodeError::Proto) {
                        DeltaDecodeError::Sanitize => self.usercmd_stats.delta_sanitize_failures += 1,
                        DeltaDecodeError::Proto => self.usercmd_stats.delta_proto_failures += 1,
                        DeltaDecodeError::InputHistoryRepeated(reason) => {
                            self.usercmd_stats.delta_repeated_failures += 1;
                            self.usercmd_stats.delta_input_history_failures += 1;
                            record_repeated_failure(&mut self.usercmd_stats, reason);
                        }
                        DeltaDecodeError::SubtickRepeated(reason) => {
                            self.usercmd_stats.delta_repeated_failures += 1;
                            self.usercmd_stats.delta_subtick_failures += 1;
                            record_repeated_failure(&mut self.usercmd_stats, reason);
                        }
                        DeltaDecodeError::Nested => self.usercmd_stats.delta_nested_failures += 1,
                    }
                    continue;
                }
            }

            let Some(next) = next else {
                continue;
            };
            #[cfg(test)]
            {
                if let Some(sink) = self.usercmd_record_sink.as_mut() {
                    sink(UserCmdTestRecordRef {
                        player_slot,
                        command_number,
                        server_tick_executed: cmd.server_tick_executed(),
                        client_tick: cmd.client_tick(),
                        baseline_command_number: test_baseline_command_number,
                        baseline_source: test_baseline_source,
                        delta_data: delta_data.map(|value| value.as_ref()),
                        command: &next,
                    });
                }
                let capture_key = (
                    player_slot,
                    command_number,
                    cmd.server_tick_executed(),
                    cmd.client_tick(),
                );
                if let Some(target_count) = self
                    .usercmd_capture_counts
                    .as_ref()
                    .and_then(|counts| counts.get(&capture_key))
                    .copied()
                {
                let captured_count = self
                    .usercmd_captured_counts
                    .entry(capture_key)
                    .or_default();
                if *captured_count < target_count {
                    self.usercmd_records.push(UserCmdTestRecord {
                        player_slot,
                        command_number,
                        server_tick_executed: cmd.server_tick_executed(),
                        client_tick: cmd.client_tick(),
                        baseline_command_number: test_baseline_command_number,
                        baseline_source: test_baseline_source,
                        delta_data: delta_data.map(|value| value.to_vec()),
                        command: next.clone(),
                    });
                    *captured_count += 1;
                }
                }
            }
            if self.usercmd_seen.insert((player_slot, command_number)) {
                self.record_user_cmd_metrics(&next, player_slot);
                if record_is_full {
                    self.usercmd_stats.decoded_full_usercmds += 1;
                } else {
                    self.usercmd_stats.decoded_delta_usercmds += 1;
                }
            }
            self.usercmd_states.entry(player_slot).or_default().ring.insert(command_number, next.clone());
            self.usercmd_states.entry(player_slot).or_default().current_command_number = Some(command_number);
            self.apply_user_cmd(
                &next,
                command_number,
                player_slot,
                cmd.server_tick_executed(),
                cmd.client_tick(),
            );
        }
        Ok(())
    }

    fn record_user_cmd_metrics(&mut self, user_cmd: &CsgoUserCmdPb, player_slot: i32) {
        self.usercmd_stats.decoded_usercmds += 1;
        if (0..64).contains(&player_slot) {
            self.usercmd_stats.player_slot_mask |= 1_u64 << player_slot;
        }
        let Some(base) = user_cmd.base.as_ref() else {
            return;
        };
        if base.buttons_pb.as_ref().map(|buttons| buttons.buttonstate1()).unwrap_or(0) != 0 {
            self.usercmd_stats.decoded_nonzero_buttons += 1;
        }
        if base.mousedx() != 0 || base.mousedy() != 0 {
            self.usercmd_stats.decoded_mouse_movement += 1;
        }
        if base.weaponselect() != 0 {
            self.usercmd_stats.decoded_weapon_selection += 1;
        }
        let subtick_count = base.subtick_moves.len() as u64;
        if subtick_count != 0 {
            self.usercmd_stats.decoded_subtick_moves += 1;
        }
        self.usercmd_stats.max_subtick_moves = self.usercmd_stats.max_subtick_moves.max(subtick_count);
        let input_history_count = user_cmd.input_history.len() as u64;
        self.usercmd_stats.max_input_history = self.usercmd_stats.max_input_history.max(input_history_count);
        if base
            .execution_notes
            .as_ref()
            .and_then(|notes| notes.ignored_reason.as_deref())
            == Some("cannot_move")
        {
            self.usercmd_stats.decoded_cannot_move += 1;
        }
        let entity_id = (base.pawn_entity_handle() & 0x7ff) as usize;
        if let Some(Some(entity)) = self.entities.get(entity_id) {
            if let Some(life_state_id) = self.prop_controller.special_ids.life_state {
                if matches!(entity.props.get(&life_state_id), Some(Variant::U32(value)) if *value != 0) {
                    self.usercmd_stats.decoded_dead += 1;
                }
            }
        }
    }

    fn apply_user_cmd(
        &mut self,
        user_cmd: &CsgoUserCmdPb,
        command_number: i32,
        player_slot: i32,
        server_tick: i32,
        transport_client_tick: i32,
    ) {
        let Some(base) = user_cmd.base.as_ref() else {
            return;
        };
        let entity_id = base.pawn_entity_handle() & 0x7ff;
        let Some(Some(ent)) = self.entities.get_mut(entity_id as usize) else {
            return;
        };

        ent.props.insert(USERCMD_COMMAND_NUMBER, Variant::I32(command_number));
        ent.props.insert(USERCMD_PLAYER_SLOT, Variant::I32(player_slot));
        ent.props.insert(USERCMD_SERVER_TICK_EXECUTED, Variant::I32(server_tick));
        ent.props
            .insert(USERCMD_PAWN_ENTITY_HANDLE, Variant::U32(base.pawn_entity_handle()));

        let history = user_cmd
            .input_history
            .iter()
            .map(|input| {
                let view_angles = input.view_angles.clone().unwrap_or_default();
                InputHistory {
                    player_tick_count: input.player_tick_count(),
                    player_tick_fraction: input.player_tick_fraction(),
                    render_tick_count: input.render_tick_count(),
                    render_tick_fraction: input.render_tick_fraction(),
                    x: view_angles.x(),
                    y: view_angles.y(),
                    z: view_angles.z(),
                    cl_interp: input.cl_interp.as_ref().map(|value| InterpolationInfo {
                        src_tick: None,
                        dst_tick: None,
                        frac: Some(value.frac()),
                    }),
                    sv_interp0: input.sv_interp0.as_ref().map(|value| InterpolationInfo {
                        src_tick: Some(value.src_tick()),
                        dst_tick: Some(value.dst_tick()),
                        frac: Some(value.frac()),
                    }),
                    sv_interp1: input.sv_interp1.as_ref().map(|value| InterpolationInfo {
                        src_tick: Some(value.src_tick()),
                        dst_tick: Some(value.dst_tick()),
                        frac: Some(value.frac()),
                    }),
                    player_interp: input.player_interp.as_ref().map(|value| InterpolationInfo {
                        src_tick: Some(value.src_tick()),
                        dst_tick: Some(value.dst_tick()),
                        frac: Some(value.frac()),
                    }),
                    frame_number: input.frame_number,
                    target_ent_index: input.target_ent_index,
                    shoot_position: input.shoot_position.as_ref().map(|value| [value.x(), value.y(), value.z()]),
                    target_head_pos_check: input.target_head_pos_check.as_ref().map(|value| [value.x(), value.y(), value.z()]),
                    target_abs_pos_check: input.target_abs_pos_check.as_ref().map(|value| [value.x(), value.y(), value.z()]),
                    target_abs_ang_check: input.target_abs_ang_check.as_ref().map(|value| [value.x(), value.y(), value.z()]),
                }
            })
            .collect();
        ent.props.insert(USERCMD_INPUT_HISTORY_BASEID, Variant::InputHistory(history));
        let subtick_moves: Vec<UserCmdSubtickMove> = base
            .subtick_moves
            .iter()
            .map(|subtick| UserCmdSubtickMove {
                when: subtick.when(),
                button: subtick.button(),
                pressed: subtick.pressed(),
                analog_forward: subtick.analog_forward_delta(),
                analog_left: subtick.analog_left_delta(),
                pitch_delta: subtick.pitch_delta(),
                yaw_delta: subtick.yaw_delta(),
            })
            .collect();
        ent.props.insert(USERCMD_SUBTICK_MOVES_BASEID, Variant::UserCmdSubtickMoves(subtick_moves.clone()));
        ent.props.insert(
            USERCMD_SUBTICK_MOVE_ANALOG_FORWARD_DELTA,
            Variant::F32Vec(subtick_moves.iter().map(|value| value.analog_forward).collect()),
        );
        ent.props.insert(
            USERCMD_SUBTICK_MOVE_ANALOG_LEFT_DELTA,
            Variant::F32Vec(subtick_moves.iter().map(|value| value.analog_left).collect()),
        );
        ent.props.insert(
            USERCMD_SUBTICK_MOVE_BUTTON,
            Variant::U64Vec(subtick_moves.iter().map(|value| value.button).collect()),
        );
        ent.props.insert(
            USERCMD_SUBTICK_MOVE_WHEN,
            Variant::F32Vec(subtick_moves.iter().map(|value| value.when).collect()),
        );
        ent.props.insert(
            USERCMD_SUBTICK_MOVE_PITCH_DELTA,
            Variant::F32Vec(subtick_moves.iter().map(|value| value.pitch_delta).collect()),
        );
        ent.props.insert(
            USERCMD_SUBTICK_MOVE_YAW_DELTA,
            Variant::F32Vec(subtick_moves.iter().map(|value| value.yaw_delta).collect()),
        );
        ent.props.insert(USERCMD_LEFTMOVE, Variant::F32(base.leftmove()));
        ent.props.insert(USERCMD_FORWARDMOVE, Variant::F32(base.forwardmove()));
        ent.props.insert(USERCMD_UPMOVE, Variant::F32(base.upmove()));
        ent.props.insert(USERCMD_IMPULSE, Variant::I32(base.impulse()));
        ent.props.insert(USERCMD_MOUSE_DX, Variant::I32(base.mousedx()));
        ent.props.insert(USERCMD_MOUSE_DY, Variant::I32(base.mousedy()));
        ent.props.insert(USERCMD_WEAPON_SELECT, Variant::I32(base.weaponselect()));
        ent.props.insert(USERCMD_LEGACY_COMMAND_NUMBER, Variant::I32(base.legacy_command_number()));
        ent.props.insert(USERCMD_BASE_CLIENT_TICK, Variant::I32(base.client_tick()));
        ent.props
            .insert(USERCMD_PREDICTION_OFFSET_TICKS_X256, Variant::U32(base.prediction_offset_ticks_x256()));
        ent.props.insert(USERCMD_RANDOM_SEED, Variant::I32(base.random_seed()));
        ent.props.insert(USERCMD_CMD_FLAGS, Variant::I32(base.cmd_flags()));
        ent.props.insert(USERCMD_TRANSPORT_CLIENT_TICK, Variant::I32(transport_client_tick));
        ent.props.insert(USERCMD_SUBTICK_LEFT_HAND_DESIRED, Variant::Bool(user_cmd.left_hand_desired()));
        ent.props
            .insert(USERCMD_ATTACK_START_HISTORY_INDEX_1, Variant::I32(user_cmd.attack1_start_history_index()));
        ent.props
            .insert(USERCMD_ATTACK_START_HISTORY_INDEX_2, Variant::I32(user_cmd.attack2_start_history_index()));
        ent.props
            .insert(USERCMD_IS_PREDICTING_BODY_SHOT_FX, Variant::Bool(user_cmd.is_predicting_body_shot_fx()));
        ent.props
            .insert(USERCMD_IS_PREDICTING_HEAD_SHOT_FX, Variant::Bool(user_cmd.is_predicting_head_shot_fx()));
        ent.props
            .insert(USERCMD_IS_PREDICTING_KILL_RAGDOLLS, Variant::Bool(user_cmd.is_predicting_kill_ragdolls()));
        if let Some(move_crc) = base.move_crc.as_ref() {
            ent.props.insert(USERCMD_MOVE_CRC, Variant::String(bytes_to_hex(move_crc.as_ref())));
        }
        if let Some(execution_notes) = base.execution_notes.as_ref() {
            if let Some(ignored_reason) = execution_notes.ignored_reason.as_ref() {
                ent.props.insert(USERCMD_EXECUTION_NOTES, Variant::String(ignored_reason.clone()));
            }
        }
        if let Some(viewangles) = base.viewangles.as_ref() {
            ent.props.insert(USERCMD_VIEWANGLE_X, Variant::F32(viewangles.x()));
            ent.props.insert(USERCMD_VIEWANGLE_Y, Variant::F32(viewangles.y()));
            ent.props.insert(USERCMD_VIEWANGLE_Z, Variant::F32(viewangles.z()));
        }
        if let Some(buttons_pb) = base.buttons_pb.as_ref() {
            ent.props.insert(USERCMD_BUTTONSTATE_1, Variant::U64(buttons_pb.buttonstate1()));
            ent.props.insert(USERCMD_BUTTONSTATE_2, Variant::U64(buttons_pb.buttonstate2()));
            ent.props.insert(USERCMD_BUTTONSTATE_3, Variant::U64(buttons_pb.buttonstate3()));
        }
        ent.props
            .insert(USERCMD_CONSUMED_SERVER_ANGLE_CHANGES, Variant::U32(base.consumed_server_angle_changes()));
    }

    pub fn parse_voice_data(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if let Ok(m) = CsvcMsgVoiceData::decode(bytes) {
            self.voice_data.push((self.tick, m));
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
