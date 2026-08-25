//! Decoder for `CMsgServerUserCmd.delta_data` payloads emitted by CS2's
//! `codegen_delta_encoder`.
//!
//! Singular fields retain protobuf wire encoding except for wire type 7,
//! which clears protobuf presence so generated getters fall back to their
//! declared defaults. The repeated input-history
//! and subtick fields use indexed repeated-entry operations observed in current
//! CS2 demos. Unknown or malformed operations fail the whole delta so callers
//! can keep the previous command-ring baseline unchanged.

use csgoproto::CBaseUserCmdExecutionNotes;
use csgoproto::CInButtonStatePb;
use csgoproto::CMsgQAngle;
use csgoproto::CMsgVector;
use csgoproto::CSubtickMoveStep;
use csgoproto::CsgoInputHistoryEntryPb;
use csgoproto::CsgoInterpolationInfoPb;
use csgoproto::CsgoInterpolationInfoPbCl;
use csgoproto::CsgoUserCmdPb;
use prost::Message;

#[derive(Clone, PartialEq, Message)]
struct DeltaBaseUserCmdPb {
    #[prost(int32, optional, tag = "1")]
    legacy_command_number: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    client_tick: Option<i32>,
    #[prost(uint32, optional, tag = "17")]
    prediction_offset_ticks_x256: Option<u32>,
    #[prost(message, optional, tag = "3")]
    buttons_pb: Option<CInButtonStatePb>,
    #[prost(message, optional, tag = "4")]
    viewangles: Option<CMsgQAngle>,
    #[prost(float, optional, tag = "5")]
    forwardmove: Option<f32>,
    #[prost(float, optional, tag = "6")]
    leftmove: Option<f32>,
    #[prost(float, optional, tag = "7")]
    upmove: Option<f32>,
    #[prost(int32, optional, tag = "8")]
    impulse: Option<i32>,
    #[prost(int32, optional, tag = "9")]
    weaponselect: Option<i32>,
    #[prost(int32, optional, tag = "10")]
    random_seed: Option<i32>,
    #[prost(int32, optional, tag = "11")]
    mousedx: Option<i32>,
    #[prost(int32, optional, tag = "12")]
    mousedy: Option<i32>,
    #[prost(uint32, optional, tag = "14")]
    pawn_entity_handle: Option<u32>,
    #[prost(bytes = "bytes", repeated, tag = "18")]
    subtick_moves_delta: Vec<prost::bytes::Bytes>,
    #[prost(bytes = "bytes", optional, tag = "19")]
    move_crc: Option<prost::bytes::Bytes>,
    #[prost(uint32, optional, tag = "20")]
    consumed_server_angle_changes: Option<u32>,
    #[prost(int32, optional, tag = "21")]
    cmd_flags: Option<i32>,
    #[prost(bytes = "bytes", optional, tag = "22")]
    execution_notes: Option<prost::bytes::Bytes>,
}

#[derive(Clone, PartialEq, Message)]
struct DeltaCsgoUserCmdPb {
    #[prost(message, optional, tag = "1")]
    base: Option<DeltaBaseUserCmdPb>,
    #[prost(bytes = "bytes", repeated, tag = "2")]
    input_history_delta: Vec<prost::bytes::Bytes>,
    #[prost(int32, optional, tag = "6")]
    attack1_start_history_index: Option<i32>,
    #[prost(int32, optional, tag = "7")]
    attack2_start_history_index: Option<i32>,
    #[prost(bool, optional, tag = "9")]
    left_hand_desired: Option<bool>,
    #[prost(bool, optional, tag = "11")]
    is_predicting_body_shot_fx: Option<bool>,
    #[prost(bool, optional, tag = "12")]
    is_predicting_head_shot_fx: Option<bool>,
    #[prost(bool, optional, tag = "13")]
    is_predicting_kill_ragdolls: Option<bool>,
}

fn read_varint(bytes: &mut &[u8]) -> Option<u64> {
    let mut value = 0_u64;
    for shift in (0..70).step_by(7) {
        let (&byte, rest) = bytes.split_first()?;
        *bytes = rest;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

fn write_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[derive(Clone, Copy)]
enum MessageSchema {
    CsgoUserCmd,
    BaseUserCmd,
    Buttons,
    QAngle,
    InputHistory,
    SubtickMove,
    Interpolation,
    InterpolationCl,
    Vector,
    ExecutionNotes,
}

impl MessageSchema {
    fn field_wire_type(self, field: u64) -> Option<u8> {
        match self {
            Self::CsgoUserCmd => match field {
                1 | 2 => Some(2),
                6 | 7 | 9 | 11 | 12 | 13 => Some(0),
                _ => None,
            },
            Self::BaseUserCmd => match field {
                1 | 2 | 8 | 9 | 10 | 11 | 12 | 14 | 17 | 20 | 21 => Some(0),
                3 | 4 | 18 | 19 | 22 => Some(2),
                5 | 6 | 7 => Some(5),
                _ => None,
            },
            Self::Buttons => match field {
                1..=3 => Some(0),
                _ => None,
            },
            Self::QAngle => match field {
                1..=3 => Some(5),
                _ => None,
            },
            Self::InputHistory => match field {
                2 | 12..=15 | 66..=69 => Some(2),
                4 | 6 | 64 | 65 => Some(0),
                5 | 7 => Some(5),
                _ => None,
            },
            Self::SubtickMove => match field {
                1 | 2 => Some(0),
                3 | 4 | 5 | 8 | 9 => Some(5),
                _ => None,
            },
            Self::Interpolation => match field {
                1 | 2 => Some(0),
                3 => Some(5),
                _ => None,
            },
            Self::InterpolationCl => match field {
                3 => Some(5),
                _ => None,
            },
            Self::Vector => match field {
                1..=4 => Some(5),
                _ => None,
            },
            Self::ExecutionNotes => match field {
                1 => Some(2),
                _ => None,
            },
        }
    }

    fn child(self, field: u64) -> Option<Self> {
        match (self, field) {
            (Self::CsgoUserCmd, 1) => Some(Self::BaseUserCmd),
            (Self::BaseUserCmd, 3) => Some(Self::Buttons),
            (Self::BaseUserCmd, 4) => Some(Self::QAngle),
            (Self::BaseUserCmd, 22) => Some(Self::ExecutionNotes),
            (Self::InputHistory, 2 | 69) => Some(Self::QAngle),
            (Self::InputHistory, 12) => Some(Self::InterpolationCl),
            (Self::InputHistory, 13 | 14 | 15) => Some(Self::Interpolation),
            (Self::InputHistory, 66 | 67 | 68) => Some(Self::Vector),
            _ => None,
        }
    }

}

#[derive(Debug, Default)]
struct SanitizedMessage {
    bytes: Vec<u8>,
    clears: Vec<Vec<u64>>,
}

fn sanitize_message(mut bytes: &[u8], schema: MessageSchema) -> Option<SanitizedMessage> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut clears = Vec::new();
    while !bytes.is_empty() {
        let key = read_varint(&mut bytes)?;
        let field = key >> 3;
        let wire_type = (key & 0x07) as u8;
        if field == 0 {
            return None;
        }

        if wire_type == 7 {
            schema.field_wire_type(field)?;
            clears.push(vec![field]);
            continue;
        }

        write_varint(key, &mut out);
        match wire_type {
            0 => {
                let value = read_varint(&mut bytes)?;
                write_varint(value, &mut out);
            }
            1 => {
                let (value, rest) = bytes.split_at_checked(8)?;
                out.extend_from_slice(value);
                bytes = rest;
            }
            2 => {
                let length = usize::try_from(read_varint(&mut bytes)?).ok()?;
                let (value, rest) = bytes.split_at_checked(length)?;
                let value = if let Some(child) = schema.child(field) {
                    let child = sanitize_message(value, child)?;
                    for mut path in child.clears {
                        path.insert(0, field);
                        clears.push(path);
                    }
                    child.bytes
                } else {
                    value.to_vec()
                };
                write_varint(value.len() as u64, &mut out);
                out.extend_from_slice(&value);
                bytes = rest;
            }
            5 => {
                let (value, rest) = bytes.split_at_checked(4)?;
                out.extend_from_slice(value);
                bytes = rest;
            }
            _ => return None,
        }
    }
    Some(SanitizedMessage { bytes: out, clears })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RepeatedDecodeError {
    Malformed,
    Truncated,
    InvalidIndex,
    IndexOutOfBounds,
    InvalidMessage,
    InvalidNestedMessage,
}

fn merge_repeated<M, F>(
    baseline: &[M],
    payloads: &[prost::bytes::Bytes],
    schema: MessageSchema,
    mut merge: F,
) -> Result<Vec<M>, RepeatedDecodeError>
where
    M: Message + Default + Clone,
    F: FnMut(&M, M, &[Vec<u64>]) -> M,
{
    // codegen_delta_encoder uses the repeated-field payload as an operation
    // stream, not as a sequence of ordinary protobuf list entries.  A wire-7
    // tag sets the resulting list length and a wire-2 tag patches an existing
    // index.  The game rejects an index equal to the current length, so an
    // operation can never append implicitly.
    const CODEGEN_REPEATED_LIMIT: usize = 0x100;
    let mut messages = baseline.to_vec();
    for payload in payloads {
        if payload.is_empty() {
            continue;
        }
        let mut bytes = payload.as_ref();
        while !bytes.is_empty() {
            let key = read_varint(&mut bytes).ok_or(RepeatedDecodeError::Malformed)?;
            let index = usize::try_from(key >> 3).map_err(|_| RepeatedDecodeError::InvalidIndex)?;
            match key & 0x07 {
                7 => {
                    if index > CODEGEN_REPEATED_LIMIT {
                        return Err(RepeatedDecodeError::InvalidIndex);
                    }
                    if index > messages.len() {
                        messages.resize(index, M::default());
                    } else {
                        messages.truncate(index);
                    }
                }
                2 => {
                    if index >= messages.len() {
                        return Err(RepeatedDecodeError::IndexOutOfBounds);
                    }
                    let length = usize::try_from(
                        read_varint(&mut bytes).ok_or(RepeatedDecodeError::Malformed)?,
                    )
                    .map_err(|_| RepeatedDecodeError::InvalidIndex)?;
                    let (message, rest) = bytes
                        .split_at_checked(length)
                        .ok_or(RepeatedDecodeError::Truncated)?;
                    let message = sanitize_message(message, schema)
                        .ok_or(RepeatedDecodeError::InvalidNestedMessage)?;
                    let decoded = M::decode(message.bytes.as_slice())
                        .map_err(|_| RepeatedDecodeError::InvalidMessage)?;
                    let current = messages[index].clone();
                    messages[index] = merge(&current, decoded, &message.clears);
                    bytes = rest;
                }
                _ => return Err(RepeatedDecodeError::Malformed),
            }
        }
    }
    Ok(messages)
}

fn replace_if_some<T>(target: &mut Option<T>, value: Option<T>) {
    if let Some(value) = value {
        *target = Some(value);
    }
}

fn merge_buttons(target: &mut Option<CInButtonStatePb>, delta: CInButtonStatePb) {
    let target = target.get_or_insert_with(|| Default::default());
    replace_if_some(&mut target.buttonstate1, delta.buttonstate1);
    replace_if_some(&mut target.buttonstate2, delta.buttonstate2);
    replace_if_some(&mut target.buttonstate3, delta.buttonstate3);
}

fn merge_qangle(target: &mut Option<CMsgQAngle>, delta: CMsgQAngle) {
    let target = target.get_or_insert_with(|| Default::default());
    replace_if_some(&mut target.x, delta.x);
    replace_if_some(&mut target.y, delta.y);
    replace_if_some(&mut target.z, delta.z);
}

fn merge_vector(target: &mut Option<CMsgVector>, delta: CMsgVector) {
    let target = target.get_or_insert_default();
    replace_if_some(&mut target.x, delta.x);
    replace_if_some(&mut target.y, delta.y);
    replace_if_some(&mut target.z, delta.z);
    replace_if_some(&mut target.w, delta.w);
}

fn merge_interpolation(target: &mut Option<CsgoInterpolationInfoPb>, delta: CsgoInterpolationInfoPb) {
    let target = target.get_or_insert_default();
    replace_if_some(&mut target.src_tick, delta.src_tick);
    replace_if_some(&mut target.dst_tick, delta.dst_tick);
    replace_if_some(&mut target.frac, delta.frac);
}

fn merge_interpolation_cl(target: &mut Option<CsgoInterpolationInfoPbCl>, delta: CsgoInterpolationInfoPbCl) {
    let target = target.get_or_insert_default();
    replace_if_some(&mut target.frac, delta.frac);
}

fn merge_input_history(
    target: &CsgoInputHistoryEntryPb,
    delta: CsgoInputHistoryEntryPb,
    clears: &[Vec<u64>],
) -> CsgoInputHistoryEntryPb {
    let mut next = target.clone();
    if let Some(view_angles) = delta.view_angles {
        merge_qangle(&mut next.view_angles, view_angles);
    }
    replace_if_some(&mut next.render_tick_count, delta.render_tick_count);
    replace_if_some(&mut next.render_tick_fraction, delta.render_tick_fraction);
    replace_if_some(&mut next.player_tick_count, delta.player_tick_count);
    replace_if_some(&mut next.player_tick_fraction, delta.player_tick_fraction);
    if let Some(value) = delta.cl_interp {
        merge_interpolation_cl(&mut next.cl_interp, value);
    }
    if let Some(value) = delta.sv_interp0 {
        merge_interpolation(&mut next.sv_interp0, value);
    }
    if let Some(value) = delta.sv_interp1 {
        merge_interpolation(&mut next.sv_interp1, value);
    }
    if let Some(value) = delta.player_interp {
        merge_interpolation(&mut next.player_interp, value);
    }
    replace_if_some(&mut next.frame_number, delta.frame_number);
    replace_if_some(&mut next.target_ent_index, delta.target_ent_index);
    if let Some(value) = delta.shoot_position {
        merge_vector(&mut next.shoot_position, value);
    }
    if let Some(value) = delta.target_head_pos_check {
        merge_vector(&mut next.target_head_pos_check, value);
    }
    if let Some(value) = delta.target_abs_pos_check {
        merge_vector(&mut next.target_abs_pos_check, value);
    }
    if let Some(value) = delta.target_abs_ang_check {
        merge_qangle(&mut next.target_abs_ang_check, value);
    }
    for path in clears {
        apply_input_history_clear_path(&mut next, path);
    }
    next
}

fn merge_subtick_move(
    target: &CSubtickMoveStep,
    delta: CSubtickMoveStep,
    clears: &[Vec<u64>],
) -> CSubtickMoveStep {
    let mut next = target.clone();
    replace_if_some(&mut next.button, delta.button);
    replace_if_some(&mut next.pressed, delta.pressed);
    replace_if_some(&mut next.when, delta.when);
    replace_if_some(&mut next.analog_forward_delta, delta.analog_forward_delta);
    replace_if_some(&mut next.analog_left_delta, delta.analog_left_delta);
    replace_if_some(&mut next.pitch_delta, delta.pitch_delta);
    replace_if_some(&mut next.yaw_delta, delta.yaw_delta);
    for path in clears {
        apply_subtick_clear_path(&mut next, path);
    }
    next
}

fn merge_execution_notes(target: &mut Option<CBaseUserCmdExecutionNotes>, delta: CBaseUserCmdExecutionNotes) {
    let target = target.get_or_insert_default();
    replace_if_some(&mut target.ignored_reason, delta.ignored_reason);
}

fn apply_buttons_clear_path(target: &mut CInButtonStatePb, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    if !rest.is_empty() {
        return;
    }
    match field {
        1 => target.buttonstate1 = None,
        2 => target.buttonstate2 = None,
        3 => target.buttonstate3 = None,
        _ => {}
    }
}

fn apply_qangle_clear_path(target: &mut CMsgQAngle, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    if !rest.is_empty() {
        return;
    }
    match field {
        1 => target.x = None,
        2 => target.y = None,
        3 => target.z = None,
        _ => {}
    }
}

fn apply_vector_clear_path(target: &mut CMsgVector, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    if !rest.is_empty() {
        return;
    }
    match field {
        1 => target.x = None,
        2 => target.y = None,
        3 => target.z = None,
        4 => target.w = None,
        _ => {}
    }
}

fn apply_interpolation_clear_path(target: &mut CsgoInterpolationInfoPb, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    if !rest.is_empty() {
        return;
    }
    match field {
        1 => target.src_tick = None,
        2 => target.dst_tick = None,
        3 => target.frac = None,
        _ => {}
    }
}

fn apply_interpolation_cl_clear_path(target: &mut CsgoInterpolationInfoPbCl, path: &[u64]) {
    if path == [3] {
        target.frac = None;
    }
}

fn apply_execution_notes_clear_path(target: &mut CBaseUserCmdExecutionNotes, path: &[u64]) {
    if path == [1] {
        target.ignored_reason = None;
    }
}

fn apply_input_history_clear_path(target: &mut CsgoInputHistoryEntryPb, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    match field {
        2 if rest.is_empty() => target.view_angles = None,
        2 => {
            if let Some(value) = target.view_angles.as_mut() {
                apply_qangle_clear_path(value, rest);
            }
        }
        4 if rest.is_empty() => target.render_tick_count = None,
        5 if rest.is_empty() => target.render_tick_fraction = None,
        6 if rest.is_empty() => target.player_tick_count = None,
        7 if rest.is_empty() => target.player_tick_fraction = None,
        12 if rest.is_empty() => target.cl_interp = None,
        12 => {
            if let Some(value) = target.cl_interp.as_mut() {
                apply_interpolation_cl_clear_path(value, rest);
            }
        }
        13 if rest.is_empty() => target.sv_interp0 = None,
        13 => {
            if let Some(value) = target.sv_interp0.as_mut() {
                apply_interpolation_clear_path(value, rest);
            }
        }
        14 if rest.is_empty() => target.sv_interp1 = None,
        14 => {
            if let Some(value) = target.sv_interp1.as_mut() {
                apply_interpolation_clear_path(value, rest);
            }
        }
        15 if rest.is_empty() => target.player_interp = None,
        15 => {
            if let Some(value) = target.player_interp.as_mut() {
                apply_interpolation_clear_path(value, rest);
            }
        }
        64 if rest.is_empty() => target.frame_number = None,
        65 if rest.is_empty() => target.target_ent_index = None,
        66 if rest.is_empty() => target.shoot_position = None,
        66 => {
            if let Some(value) = target.shoot_position.as_mut() {
                apply_vector_clear_path(value, rest);
            }
        }
        67 if rest.is_empty() => target.target_head_pos_check = None,
        67 => {
            if let Some(value) = target.target_head_pos_check.as_mut() {
                apply_vector_clear_path(value, rest);
            }
        }
        68 if rest.is_empty() => target.target_abs_pos_check = None,
        68 => {
            if let Some(value) = target.target_abs_pos_check.as_mut() {
                apply_vector_clear_path(value, rest);
            }
        }
        69 if rest.is_empty() => target.target_abs_ang_check = None,
        69 => {
            if let Some(value) = target.target_abs_ang_check.as_mut() {
                apply_qangle_clear_path(value, rest);
            }
        }
        _ => {}
    }
}

fn apply_subtick_clear_path(target: &mut CSubtickMoveStep, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    if !rest.is_empty() {
        return;
    }
    match field {
        1 => target.button = None,
        2 => target.pressed = None,
        3 => target.when = None,
        4 => target.analog_forward_delta = None,
        5 => target.analog_left_delta = None,
        8 => target.pitch_delta = None,
        9 => target.yaw_delta = None,
        _ => {}
    }
}

fn apply_base_clear_path(target: &mut csgoproto::CBaseUserCmdPb, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    match field {
        1 if rest.is_empty() => target.legacy_command_number = None,
        2 if rest.is_empty() => target.client_tick = None,
        3 if rest.is_empty() => target.buttons_pb = None,
        3 => {
            if let Some(value) = target.buttons_pb.as_mut() {
                apply_buttons_clear_path(value, rest);
            }
        }
        4 if rest.is_empty() => target.viewangles = None,
        4 => {
            if let Some(value) = target.viewangles.as_mut() {
                apply_qangle_clear_path(value, rest);
            }
        }
        5 if rest.is_empty() => target.forwardmove = None,
        6 if rest.is_empty() => target.leftmove = None,
        7 if rest.is_empty() => target.upmove = None,
        8 if rest.is_empty() => target.impulse = None,
        9 if rest.is_empty() => target.weaponselect = None,
        10 if rest.is_empty() => target.random_seed = None,
        11 if rest.is_empty() => target.mousedx = None,
        12 if rest.is_empty() => target.mousedy = None,
        14 if rest.is_empty() => target.pawn_entity_handle = None,
        17 if rest.is_empty() => target.prediction_offset_ticks_x256 = None,
        18 if rest.is_empty() => target.subtick_moves.clear(),
        19 if rest.is_empty() => target.move_crc = None,
        20 if rest.is_empty() => target.consumed_server_angle_changes = None,
        21 if rest.is_empty() => target.cmd_flags = None,
        22 if rest.is_empty() => target.execution_notes = None,
        22 => {
            if let Some(value) = target.execution_notes.as_mut() {
                apply_execution_notes_clear_path(value, rest);
            }
        }
        _ => {}
    }
}

fn apply_csgo_clear_path(target: &mut CsgoUserCmdPb, path: &[u64]) {
    let Some((&field, rest)) = path.split_first() else {
        return;
    };
    match field {
        1 if rest.is_empty() => target.base = None,
        1 => {
            if let Some(value) = target.base.as_mut() {
                apply_base_clear_path(value, rest);
            }
        }
        2 if rest.is_empty() => target.input_history.clear(),
        6 if rest.is_empty() => target.attack1_start_history_index = None,
        7 if rest.is_empty() => target.attack2_start_history_index = None,
        9 if rest.is_empty() => target.left_hand_desired = None,
        11 if rest.is_empty() => target.is_predicting_body_shot_fx = None,
        12 if rest.is_empty() => target.is_predicting_head_shot_fx = None,
        13 if rest.is_empty() => target.is_predicting_kill_ragdolls = None,
        _ => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeltaDecodeError {
    Sanitize,
    Proto,
    InputHistoryRepeated(RepeatedDecodeError),
    SubtickRepeated(RepeatedDecodeError),
    Nested,
}

pub(super) fn apply_delta_with_error(
    baseline: &CsgoUserCmdPb,
    delta_data: &[u8],
) -> Result<CsgoUserCmdPb, DeltaDecodeError> {
    let sanitized = sanitize_message(delta_data, MessageSchema::CsgoUserCmd)
        .ok_or(DeltaDecodeError::Sanitize)?;
    let delta = DeltaCsgoUserCmdPb::decode(sanitized.bytes.as_slice())
        .map_err(|_| DeltaDecodeError::Proto)?;
    let mut next = baseline.clone();

    if !delta.input_history_delta.is_empty() {
        next.input_history = merge_repeated(
            &next.input_history,
            &delta.input_history_delta,
            MessageSchema::InputHistory,
            merge_input_history,
        )
        .map_err(DeltaDecodeError::InputHistoryRepeated)?;
    }
    replace_if_some(&mut next.attack1_start_history_index, delta.attack1_start_history_index);
    replace_if_some(&mut next.attack2_start_history_index, delta.attack2_start_history_index);
    replace_if_some(&mut next.left_hand_desired, delta.left_hand_desired);
    replace_if_some(&mut next.is_predicting_body_shot_fx, delta.is_predicting_body_shot_fx);
    replace_if_some(&mut next.is_predicting_head_shot_fx, delta.is_predicting_head_shot_fx);
    replace_if_some(&mut next.is_predicting_kill_ragdolls, delta.is_predicting_kill_ragdolls);

    if let Some(delta_base) = delta.base {
        let base = next.base.get_or_insert(Default::default());
        replace_if_some(&mut base.legacy_command_number, delta_base.legacy_command_number);
        replace_if_some(&mut base.client_tick, delta_base.client_tick);
        replace_if_some(&mut base.prediction_offset_ticks_x256, delta_base.prediction_offset_ticks_x256);
        if let Some(buttons) = delta_base.buttons_pb {
            merge_buttons(&mut base.buttons_pb, buttons);
        }
        if let Some(viewangles) = delta_base.viewangles {
            merge_qangle(&mut base.viewangles, viewangles);
        }
        replace_if_some(&mut base.forwardmove, delta_base.forwardmove);
        replace_if_some(&mut base.leftmove, delta_base.leftmove);
        replace_if_some(&mut base.upmove, delta_base.upmove);
        replace_if_some(&mut base.impulse, delta_base.impulse);
        replace_if_some(&mut base.weaponselect, delta_base.weaponselect);
        replace_if_some(&mut base.random_seed, delta_base.random_seed);
        replace_if_some(&mut base.mousedx, delta_base.mousedx);
        replace_if_some(&mut base.mousedy, delta_base.mousedy);
        replace_if_some(&mut base.pawn_entity_handle, delta_base.pawn_entity_handle);
        replace_if_some(&mut base.move_crc, delta_base.move_crc);
        replace_if_some(&mut base.consumed_server_angle_changes, delta_base.consumed_server_angle_changes);
        replace_if_some(&mut base.cmd_flags, delta_base.cmd_flags);
        if let Some(notes) = delta_base.execution_notes {
            merge_execution_notes(
                &mut base.execution_notes,
                CBaseUserCmdExecutionNotes::decode(notes)
                    .map_err(|_| DeltaDecodeError::Nested)?,
            );
        }
        if !delta_base.subtick_moves_delta.is_empty() {
            base.subtick_moves = merge_repeated(
                &base.subtick_moves,
                &delta_base.subtick_moves_delta,
                MessageSchema::SubtickMove,
                merge_subtick_move,
            )
            .map_err(DeltaDecodeError::SubtickRepeated)?;
        }
    }

    for path in &sanitized.clears {
        apply_csgo_clear_path(&mut next, path);
    }

    Ok(next)
}

pub(super) fn apply_delta(baseline: &CsgoUserCmdPb, delta_data: &[u8]) -> Option<CsgoUserCmdPb> {
    apply_delta_with_error(baseline, delta_data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::first_pass::parser_settings::{FirstPassParser, ParserInputs};
    use crate::second_pass::parser_settings::{
        create_huffman_lookup_table, SecondPassParser, UserCmdTestRecordRef,
        UserCmdTransportTestRecord,
    };
    use ahash::AHashMap;
    use std::collections::{BTreeMap, VecDeque};
    use std::fmt::Debug;
    use std::fs::File;
    use std::io::{BufReader, ErrorKind, Read};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct DllOracleRecord {
        ordinal: u64,
        status: DllOracleStatus,
        player_slot: i32,
        command_number: i32,
        server_tick_executed: i32,
        client_tick: i32,
        ring_command_number: i32,
        cache_command_number: i32,
        server_cmd_has_bits: u32,
        command: Option<CsgoUserCmdPb>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DllOracleStatus {
        Decoded,
        Rejected,
    }

    struct BinaryDllOracle {
        reader: BufReader<File>,
        version: u32,
        record_header_size: u32,
        transport_records: u64,
        decoded_records: u64,
        rejected_records: u64,
    }

    type TransportKey = (i32, i32, i32, i32);

    struct KeyedOracleState {
        oracle: AHashMap<TransportKey, VecDeque<DllOracleRecord>>,
        current: Option<DllOracleRecord>,
        current_rust_decoded: bool,
        parser_transports: u64,
        parser_unmatched_bootstrap: u64,
        matched_decoded: u64,
        matched_rejected: u64,
        aligned: bool,
    }

    const BINARY_V1_FILE_HEADER_SIZE: u32 = 40;
    const BINARY_V1_RECORD_HEADER_SIZE: u32 = 44;
    const BINARY_V2_FILE_HEADER_SIZE: u32 = 80;
    const BINARY_V2_RECORD_HEADER_SIZE: u32 = 48;
    fn read_u32(reader: &mut impl Read) -> Result<u32, String> {
        let mut bytes = [0_u8; 4];
        reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i32(reader: &mut impl Read) -> Result<i32, String> {
        let mut bytes = [0_u8; 4];
        reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_u64(reader: &mut impl Read) -> Result<u64, String> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_u32_or_eof(reader: &mut impl Read) -> Result<Option<u32>, String> {
        let mut bytes = [0_u8; 4];
        match reader.read(&mut bytes[..1]) {
            Ok(0) => return Ok(None),
            Ok(1) => {}
            Ok(_) => unreachable!(),
            Err(error) => return Err(error.to_string()),
        }
        reader.read_exact(&mut bytes[1..]).map_err(|error| {
            if error.kind() == ErrorKind::UnexpectedEof {
                "truncated binary oracle record-size prefix".to_string()
            } else {
                error.to_string()
            }
        })?;
        Ok(Some(u32::from_le_bytes(bytes)))
    }

    impl BinaryDllOracle {
        fn open(path: &PathBuf) -> Result<Self, String> {
            let file = File::open(path).map_err(|error| format!("open binary DLL oracle: {error}"))?;
            let mut reader = BufReader::new(file);
            let mut magic = [0_u8; 8];
            reader.read_exact(&mut magic).map_err(|error| error.to_string())?;
            if &magic != b"CS2UCMD1" {
                return Err(format!("invalid binary DLL oracle magic: {magic:?}"));
            }
            let version = read_u32(&mut reader)?;
            let file_header_size = read_u32(&mut reader)?;
            let record_header_size = read_u32(&mut reader)?;
            let _flags = read_u32(&mut reader)?;
            let _max_records = read_u64(&mut reader)?;
            let _pre_decode_hook_rva = read_u64(&mut reader)?;
            let (minimum_file_header_size, minimum_record_header_size, transport_records, decoded_records, rejected_records) =
                match version {
                    1 => (
                        BINARY_V1_FILE_HEADER_SIZE,
                        BINARY_V1_RECORD_HEADER_SIZE,
                        0,
                        0,
                        0,
                    ),
                    2 => {
                        let _post_decode_hook_rva = read_u64(&mut reader)?;
                        let transport_records = read_u64(&mut reader)?;
                        let decoded_records = read_u64(&mut reader)?;
                        let rejected_records = read_u64(&mut reader)?;
                        let write_failures = read_u64(&mut reader)?;
                        if write_failures != 0 {
                            return Err(format!(
                                "binary DLL oracle reports {write_failures} write failure(s)"
                            ));
                        }
                        (
                            BINARY_V2_FILE_HEADER_SIZE,
                            BINARY_V2_RECORD_HEADER_SIZE,
                            transport_records,
                            decoded_records,
                            rejected_records,
                        )
                    }
                    _ => return Err(format!("unsupported binary DLL oracle version: {version}")),
                };
            if file_header_size < minimum_file_header_size || record_header_size < minimum_record_header_size {
                return Err(format!(
                    "binary DLL oracle header sizes are too small: file={file_header_size}, record={record_header_size}"
                ));
            }
            if file_header_size > minimum_file_header_size {
                let mut extension = vec![0_u8; (file_header_size - minimum_file_header_size) as usize];
                reader.read_exact(&mut extension).map_err(|error| error.to_string())?;
            }
            Ok(Self {
                reader,
                version,
                record_header_size,
                transport_records,
                decoded_records,
                rejected_records,
            })
        }

        fn read_record(&mut self) -> Result<Option<DllOracleRecord>, String> {
            let Some(record_size) = read_u32_or_eof(&mut self.reader)? else {
                return Ok(None);
            };
            let payload_size = read_u32(&mut self.reader)?;
            let ordinal = read_u64(&mut self.reader)?;
            let status = if self.version == 1 {
                DllOracleStatus::Decoded
            } else {
                match read_u32(&mut self.reader)? {
                    1 => DllOracleStatus::Decoded,
                    2 => DllOracleStatus::Rejected,
                    value => {
                        return Err(format!(
                            "binary DLL oracle record {ordinal} has unknown status {value}"
                        ))
                    }
                }
            };
            let player_slot = read_i32(&mut self.reader)?;
            let command_number = read_u32(&mut self.reader)? as i32;
            let server_tick_executed = read_i32(&mut self.reader)?;
            let client_tick = read_i32(&mut self.reader)?;
            let ring_command_number = read_u32(&mut self.reader)? as i32;
            let cache_command_number = read_i32(&mut self.reader)?;
            let server_cmd_has_bits = read_u32(&mut self.reader)?;
            let minimum_record_header_size = if self.version == 1 {
                BINARY_V1_RECORD_HEADER_SIZE
            } else {
                BINARY_V2_RECORD_HEADER_SIZE
            };
            if self.record_header_size > minimum_record_header_size {
                let mut extension = vec![0_u8; (self.record_header_size - minimum_record_header_size) as usize];
                self.reader.read_exact(&mut extension).map_err(|error| error.to_string())?;
            }
            if record_size != self.record_header_size + payload_size {
                return Err(format!(
                    "binary DLL oracle record {ordinal} has inconsistent sizes: record={record_size}, header={}, payload={payload_size}",
                    self.record_header_size
                ));
            }
            if status == DllOracleStatus::Decoded &&
                (ring_command_number != command_number || cache_command_number != command_number)
            {
                return Err(format!(
                    "binary DLL oracle record {ordinal} failed command validation: command={command_number}, ring={ring_command_number}, cache={cache_command_number}"
                ));
            }
            let mut payload = vec![0_u8; payload_size as usize];
            self.reader.read_exact(&mut payload).map_err(|error| error.to_string())?;
            let command = match status {
                DllOracleStatus::Decoded => Some(
                    CsgoUserCmdPb::decode(payload.as_slice())
                        .map_err(|error| {
                            let prefix: Vec<u8> = payload.iter().copied().take(128).collect();
                            format!(
                                "decode binary DLL oracle record {ordinal}: {error}; payload_len={}; prefix={}",
                                payload.len(),
                                encode_hex(&prefix)
                            )
                        })?,
                ),
                DllOracleStatus::Rejected => {
                    if !payload.is_empty() {
                        return Err(format!(
                            "rejected binary DLL oracle record {ordinal} unexpectedly contains a payload"
                        ));
                    }
                    None
                }
            };
            Ok(Some(DllOracleRecord {
                ordinal,
                status,
                player_slot,
                command_number,
                server_tick_executed,
                client_tick,
                ring_command_number,
                cache_command_number,
                server_cmd_has_bits,
                command,
            }))
        }
    }

    fn decode_hex(value: &str) -> Vec<u8> {
        assert_eq!(value.len() % 2, 0, "odd-length hex payload");
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let text = std::str::from_utf8(pair).expect("hex is UTF-8");
                u8::from_str_radix(text, 16).expect("valid payload hex")
            })
            .collect()
    }

    fn encode_hex(value: &[u8]) -> String {
        value.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn load_dll_oracle(path: &PathBuf) -> Vec<DllOracleRecord> {
        std::fs::read_to_string(path)
            .expect("read DLL TSV")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("ordinal\t"))
            .map(|line| {
                let columns = line.split('\t').collect::<Vec<_>>();
                assert_eq!(columns.len(), 10, "unexpected DLL TSV row: {line}");
                let payload_len = columns[8].parse::<usize>().expect("payload_len");
                let payload = decode_hex(columns[9]);
                assert_eq!(payload.len(), payload_len, "payload_len mismatch");
                DllOracleRecord {
                    ordinal: columns[0].parse().expect("ordinal"),
                    status: DllOracleStatus::Decoded,
                    player_slot: columns[1].parse().expect("player_slot"),
                    command_number: columns[2].parse().expect("command_number"),
                    server_tick_executed: columns[3].parse().expect("server_tick_executed"),
                    client_tick: columns[4].parse().expect("client_tick"),
                    ring_command_number: 0,
                    cache_command_number: 0,
                    server_cmd_has_bits: u32::from_str_radix(
                        columns[7].trim_start_matches("0x"),
                        16,
                    )
                    .expect("server_cmd_has_bits"),
                    command: Some(
                        CsgoUserCmdPb::decode(payload.as_slice())
                            .expect("DLL payload is CSGOUserCmdPB"),
                    ),
                }
            })
            .collect()
    }

    fn scalar_diff<T: PartialEq + Debug>(path: &str, expected: &T, actual: &T) -> Option<String> {
        (expected != actual).then(|| format!("{path}: DLL={expected:?}, Rust={actual:?}"))
    }

    fn qangle_diff(path: &str, expected: &Option<CMsgQAngle>, actual: &Option<CMsgQAngle>) -> Option<String> {
        match (expected, actual) {
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => scalar_diff(path, expected, actual),
            (Some(expected), Some(actual)) => scalar_diff(&format!("{path}.x"), &expected.x, &actual.x)
                .or_else(|| scalar_diff(&format!("{path}.y"), &expected.y, &actual.y))
                .or_else(|| scalar_diff(&format!("{path}.z"), &expected.z, &actual.z)),
        }
    }

    fn vector_diff(path: &str, expected: &Option<CMsgVector>, actual: &Option<CMsgVector>) -> Option<String> {
        match (expected, actual) {
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => scalar_diff(path, expected, actual),
            (Some(expected), Some(actual)) => scalar_diff(&format!("{path}.x"), &expected.x, &actual.x)
                .or_else(|| scalar_diff(&format!("{path}.y"), &expected.y, &actual.y))
                .or_else(|| scalar_diff(&format!("{path}.z"), &expected.z, &actual.z))
                .or_else(|| scalar_diff(&format!("{path}.w"), &expected.w, &actual.w)),
        }
    }

    fn interpolation_diff(
        path: &str,
        expected: &Option<CsgoInterpolationInfoPb>,
        actual: &Option<CsgoInterpolationInfoPb>,
    ) -> Option<String> {
        match (expected, actual) {
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => scalar_diff(path, expected, actual),
            (Some(expected), Some(actual)) => scalar_diff(&format!("{path}.src_tick"), &expected.src_tick, &actual.src_tick)
                .or_else(|| scalar_diff(&format!("{path}.dst_tick"), &expected.dst_tick, &actual.dst_tick))
                .or_else(|| scalar_diff(&format!("{path}.frac"), &expected.frac, &actual.frac)),
        }
    }

    fn interpolation_cl_diff(
        path: &str,
        expected: &Option<CsgoInterpolationInfoPbCl>,
        actual: &Option<CsgoInterpolationInfoPbCl>,
    ) -> Option<String> {
        match (expected, actual) {
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => scalar_diff(path, expected, actual),
            (Some(expected), Some(actual)) => scalar_diff(&format!("{path}.frac"), &expected.frac, &actual.frac),
        }
    }

    fn first_usercmd_difference(expected: &CsgoUserCmdPb, actual: &CsgoUserCmdPb) -> Option<String> {
        match (&expected.base, &actual.base) {
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => return scalar_diff("base", &expected.base, &actual.base),
            (Some(expected), Some(actual)) => {
                macro_rules! base_scalar {
                    ($field:ident) => {
                        if let Some(diff) = scalar_diff(concat!("base.", stringify!($field)), &expected.$field, &actual.$field) {
                            return Some(diff);
                        }
                    };
                }
                base_scalar!(legacy_command_number);
                base_scalar!(client_tick);
                base_scalar!(prediction_offset_ticks_x256);
                match (&expected.buttons_pb, &actual.buttons_pb) {
                    (None, None) => {}
                    (Some(_), None) | (None, Some(_)) => return scalar_diff("base.buttons_pb", &expected.buttons_pb, &actual.buttons_pb),
                    (Some(expected), Some(actual)) => {
                        if let Some(diff) = scalar_diff("base.buttons_pb.buttonstate1", &expected.buttonstate1, &actual.buttonstate1)
                            .or_else(|| scalar_diff("base.buttons_pb.buttonstate2", &expected.buttonstate2, &actual.buttonstate2))
                            .or_else(|| scalar_diff("base.buttons_pb.buttonstate3", &expected.buttonstate3, &actual.buttonstate3))
                        {
                            return Some(diff);
                        }
                    }
                }
                if let Some(diff) = qangle_diff("base.viewangles", &expected.viewangles, &actual.viewangles) {
                    return Some(diff);
                }
                base_scalar!(forwardmove);
                base_scalar!(leftmove);
                base_scalar!(upmove);
                base_scalar!(impulse);
                base_scalar!(weaponselect);
                base_scalar!(random_seed);
                base_scalar!(mousedx);
                base_scalar!(mousedy);
                base_scalar!(pawn_entity_handle);
                if expected.subtick_moves.len() != actual.subtick_moves.len() {
                    return Some(format!(
                        "base.subtick_moves.length: DLL={}, Rust={}",
                        expected.subtick_moves.len(),
                        actual.subtick_moves.len()
                    ));
                }
                for (index, (expected, actual)) in expected.subtick_moves.iter().zip(&actual.subtick_moves).enumerate() {
                    macro_rules! subtick_scalar {
                        ($field:ident) => {
                            if let Some(diff) = scalar_diff(
                                &format!("base.subtick_moves[{index}].{}", stringify!($field)),
                                &expected.$field,
                                &actual.$field,
                            ) {
                                return Some(diff);
                            }
                        };
                    }
                    subtick_scalar!(button);
                    subtick_scalar!(pressed);
                    subtick_scalar!(when);
                    subtick_scalar!(analog_forward_delta);
                    subtick_scalar!(analog_left_delta);
                    subtick_scalar!(pitch_delta);
                    subtick_scalar!(yaw_delta);
                }
                base_scalar!(move_crc);
                base_scalar!(consumed_server_angle_changes);
                base_scalar!(cmd_flags);
                if let Some(diff) = scalar_diff("base.execution_notes", &expected.execution_notes, &actual.execution_notes) {
                    return Some(diff);
                }
            }
        }

        if expected.input_history.len() != actual.input_history.len() {
            return Some(format!(
                "input_history.length: DLL={}, Rust={}",
                expected.input_history.len(),
                actual.input_history.len()
            ));
        }
        for (index, (expected, actual)) in expected.input_history.iter().zip(&actual.input_history).enumerate() {
            let prefix = format!("input_history[{index}]");
            if let Some(diff) = qangle_diff(&format!("{prefix}.view_angles"), &expected.view_angles, &actual.view_angles)
                .or_else(|| scalar_diff(&format!("{prefix}.render_tick_count"), &expected.render_tick_count, &actual.render_tick_count))
                .or_else(|| scalar_diff(&format!("{prefix}.render_tick_fraction"), &expected.render_tick_fraction, &actual.render_tick_fraction))
                .or_else(|| scalar_diff(&format!("{prefix}.player_tick_count"), &expected.player_tick_count, &actual.player_tick_count))
                .or_else(|| scalar_diff(&format!("{prefix}.player_tick_fraction"), &expected.player_tick_fraction, &actual.player_tick_fraction))
                .or_else(|| interpolation_cl_diff(&format!("{prefix}.cl_interp"), &expected.cl_interp, &actual.cl_interp))
                .or_else(|| interpolation_diff(&format!("{prefix}.sv_interp0"), &expected.sv_interp0, &actual.sv_interp0))
                .or_else(|| interpolation_diff(&format!("{prefix}.sv_interp1"), &expected.sv_interp1, &actual.sv_interp1))
                .or_else(|| interpolation_diff(&format!("{prefix}.player_interp"), &expected.player_interp, &actual.player_interp))
                .or_else(|| scalar_diff(&format!("{prefix}.frame_number"), &expected.frame_number, &actual.frame_number))
                .or_else(|| scalar_diff(&format!("{prefix}.target_ent_index"), &expected.target_ent_index, &actual.target_ent_index))
                .or_else(|| vector_diff(&format!("{prefix}.shoot_position"), &expected.shoot_position, &actual.shoot_position))
                .or_else(|| vector_diff(&format!("{prefix}.target_head_pos_check"), &expected.target_head_pos_check, &actual.target_head_pos_check))
                .or_else(|| vector_diff(&format!("{prefix}.target_abs_pos_check"), &expected.target_abs_pos_check, &actual.target_abs_pos_check))
                .or_else(|| qangle_diff(&format!("{prefix}.target_abs_ang_check"), &expected.target_abs_ang_check, &actual.target_abs_ang_check))
            {
                return Some(diff);
            }
        }

        scalar_diff("attack1_start_history_index", &expected.attack1_start_history_index, &actual.attack1_start_history_index)
            .or_else(|| scalar_diff("attack2_start_history_index", &expected.attack2_start_history_index, &actual.attack2_start_history_index))
            .or_else(|| scalar_diff("left_hand_desired", &expected.left_hand_desired, &actual.left_hand_desired))
            .or_else(|| scalar_diff("is_predicting_body_shot_fx", &expected.is_predicting_body_shot_fx, &actual.is_predicting_body_shot_fx))
            .or_else(|| scalar_diff("is_predicting_head_shot_fx", &expected.is_predicting_head_shot_fx, &actual.is_predicting_head_shot_fx))
            .or_else(|| scalar_diff("is_predicting_kill_ragdolls", &expected.is_predicting_kill_ragdolls, &actual.is_predicting_kill_ragdolls))
    }

    fn keyed_transport_key(record: &UserCmdTransportTestRecord) -> TransportKey {
        (
            record.player_slot,
            record.command_number,
            record.server_tick_executed,
            record.client_tick,
        )
    }

    fn finish_keyed_transport(state: &mut KeyedOracleState) {
        let Some(expected) = state.current.take() else {
            return;
        };
        match (expected.status, state.current_rust_decoded) {
            (DllOracleStatus::Decoded, false) => panic!(
                "Rust rejected DLL-decoded usercmd at slot {} command {}",
                expected.player_slot, expected.command_number
            ),
            (DllOracleStatus::Rejected, false) => state.matched_rejected += 1,
            (DllOracleStatus::Decoded, true) => {}
            (DllOracleStatus::Rejected, true) => unreachable!(
                "Rust decoded DLL-rejected usercmd at slot {} command {}",
                expected.player_slot, expected.command_number
            ),
        }
        state.current_rust_decoded = false;
    }

    fn begin_keyed_transport(state: &mut KeyedOracleState, actual: UserCmdTransportTestRecord) {
        finish_keyed_transport(state);
        state.parser_transports += 1;
        let key = keyed_transport_key(&actual);
        let expected = state.oracle.get_mut(&key).and_then(VecDeque::pop_front);
        let Some(expected) = expected else {
            if !state.aligned && actual.server_cmd_has_bits & 1 == 0 {
                state.parser_unmatched_bootstrap += 1;
                return;
            }
            panic!(
                "Rust transport has no DLL oracle record at parser transport {}: slot={} command={} server_tick={} client_tick={} delta={}",
                state.parser_transports,
                actual.player_slot,
                actual.command_number,
                actual.server_tick_executed,
                actual.client_tick,
                actual.server_cmd_has_bits & 2 != 0,
            );
        };
        state.aligned = true;
        state.current = Some(expected);
    }

    fn compare_keyed_decoded(state: &mut KeyedOracleState, actual: UserCmdTestRecordRef<'_>) {
        let expected = state.current.as_ref().unwrap_or_else(|| {
            panic!("Rust decoded a usercmd without a keyed DLL transport record")
        });
        assert!(!state.current_rust_decoded, "Rust decoded one keyed transport command more than once");
        assert_eq!(
            expected.status,
            DllOracleStatus::Decoded,
            "Rust decoded a DLL-rejected keyed usercmd at slot {} command {}: server_tick={} client_tick={} presence=0x{:x} ring={} cache={}; Rust baseline={:?}/{:?} data={} delta={} payload={}",
            expected.player_slot,
            expected.command_number,
            expected.server_tick_executed,
            expected.client_tick,
            expected.server_cmd_has_bits,
            expected.ring_command_number,
            expected.cache_command_number,
            actual.baseline_command_number,
            actual.baseline_source,
            actual.delta_data.is_none(),
            actual.delta_data.is_some(),
            actual.delta_data.map(encode_hex).unwrap_or_default(),
        );
        let expected_command = expected.command.as_ref().expect("decoded keyed DLL record payload");
        if expected_command != actual.command {
            panic!(
                "DLL/Rust keyed usercmd mismatch at slot {} command {}: {}; baseline={:?}/{:?}; delta_data={}",
                expected.player_slot,
                expected.command_number,
                first_usercmd_difference(expected_command, actual.command)
                    .unwrap_or_else(|| "message inequality without a projected field difference".to_string()),
                actual.baseline_command_number,
                actual.baseline_source,
                actual.delta_data.map(encode_hex).unwrap_or_default(),
            );
        }
        state.current_rust_decoded = true;
        state.matched_decoded += 1;
    }

    #[test]
    #[ignore = "requires a real CS2 demo and an instrumented client.dll TSV"]
    fn matches_instrumented_dll_usercmds_field_for_field() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let demo_path = std::env::var_os("CS2_USERCMD_DEMO")
            .map(PathBuf::from)
            .unwrap_or_else(|| manifest_dir.join("../../issue340_pr343_sample.dem"));
        let tsv_path = std::env::var_os("CS2_DLL_USERCMD_TSV")
            .map(PathBuf::from)
            .unwrap_or_else(|| manifest_dir.join("../../tools/usercmd_probe/dll.tsv"));
        let oracle = load_dll_oracle(&tsv_path);
        assert!(!oracle.is_empty(), "DLL oracle is empty");

        let mut capture_counts = AHashMap::default();
        for record in &oracle {
            *capture_counts
                .entry((
                    record.player_slot,
                    record.command_number,
                    record.server_tick_executed,
                    record.client_tick,
                ))
                .or_insert(0_usize) += 1;
        }

        let demo = std::fs::read(&demo_path).expect("read demo");
        let huffman_lookup_table = create_huffman_lookup_table();
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec!["usercmd_command_number".to_string()],
            wanted_other_props: vec![],
            wanted_prop_states: AHashMap::default(),
            wanted_ticks: vec![],
            wanted_events: vec![],
            parse_ents: false,
            parse_projectiles: false,
            parse_grenades: false,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &huffman_lookup_table,
            order_by_steamid: false,
            list_props: false,
            fallback_bytes: None,
        };
        let mut first_pass = FirstPassParser::new(&settings);
        let first_pass_output = first_pass.parse_demo(&demo, false).expect("first pass");
        let mut second_pass = SecondPassParser::new(first_pass_output, 16, true, None).expect("second pass init");
        second_pass.usercmd_capture_counts = Some(capture_counts);
        second_pass.start(&demo).expect("second pass");

        let mut actual = BTreeMap::<_, VecDeque<_>>::new();
        for record in &second_pass.usercmd_records {
            actual
                .entry((
                    record.player_slot,
                    record.command_number,
                    record.server_tick_executed,
                    record.client_tick,
                ))
                .or_default()
                .push_back(record);
        }
        for expected in &oracle {
            let key = (
                expected.player_slot,
                expected.command_number,
                expected.server_tick_executed,
                expected.client_tick,
            );
            let actual = actual.get_mut(&key).and_then(VecDeque::pop_front).unwrap_or_else(|| {
                panic!(
                    "missing Rust command for DLL ordinal {} slot {} command {}; stats={:?}",
                    expected.ordinal, expected.player_slot, expected.command_number, second_pass.usercmd_stats
                )
            });
            assert_eq!(
                expected.server_tick_executed, actual.server_tick_executed,
                "server_tick_executed mismatch at DLL ordinal {}",
                expected.ordinal
            );
            assert_eq!(
                expected.client_tick, actual.client_tick,
                "client_tick mismatch at DLL ordinal {}",
                expected.ordinal
            );
            let expected_command = expected.command.as_ref().expect("decoded TSV DLL record payload");
            if expected_command != &actual.command {
                panic!(
                    "DLL/Rust usercmd mismatch at ordinal {} slot {} command {}: {}; baseline={:?}/{:?}; delta_data={}",
                    expected.ordinal,
                    expected.player_slot,
                    expected.command_number,
                    first_usercmd_difference(expected_command, &actual.command)
                        .unwrap_or_else(|| "message inequality without a projected field difference".to_string()),
                    actual.baseline_command_number,
                    actual.baseline_source,
                    actual.delta_data.as_deref().map(encode_hex).unwrap_or_default(),
                );
            }
        }
        assert_eq!(
            second_pass.usercmd_records.len(),
            oracle.len(),
            "captured Rust command count differs from DLL oracle"
        );
        assert!(actual.values().all(VecDeque::is_empty), "Rust captured extra duplicate commands");
    }

    #[test]
    #[ignore = "requires the complete instrumented client.dll binary usercmd stream"]
    fn matches_full_instrumented_dll_usercmd_stream_field_for_field() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let demo_path = std::env::var_os("CS2_USERCMD_DEMO")
            .map(PathBuf::from)
            .unwrap_or_else(|| manifest_dir.join("../../issue340_pr343_sample.dem"));
        let stream_path = std::env::var_os("CS2_DLL_USERCMD_STREAM")
            .map(PathBuf::from)
            .unwrap_or_else(|| manifest_dir.join("../../tools/usercmd_probe/dll_usercmd_full.pbstream"));
        let mut oracle_reader = BinaryDllOracle::open(&stream_path).expect("open binary DLL oracle");
        assert_eq!(oracle_reader.version, 2, "complete binary comparison requires a v2 DLL oracle");
        assert_eq!(
            oracle_reader.decoded_records + oracle_reader.rejected_records,
            oracle_reader.transport_records,
            "DLL oracle header counts are incomplete"
        );
        let expected_transport_records = oracle_reader.transport_records;
        let expected_decoded_records = oracle_reader.decoded_records;
        let expected_rejected_records = oracle_reader.rejected_records;
        let mut oracle_by_key: AHashMap<TransportKey, VecDeque<DllOracleRecord>> = AHashMap::default();
        while let Some(record) = oracle_reader.read_record().expect("read keyed DLL oracle") {
            oracle_by_key
                .entry((
                    record.player_slot,
                    record.command_number,
                    record.server_tick_executed,
                    record.client_tick,
                ))
                .or_default()
                .push_back(record);
        }
        let state = Arc::new(Mutex::new(KeyedOracleState {
            oracle: oracle_by_key,
            current: None,
            current_rust_decoded: false,
            parser_transports: 0,
            parser_unmatched_bootstrap: 0,
            matched_decoded: 0,
            matched_rejected: 0,
            aligned: false,
        }));

        let demo = std::fs::read(&demo_path).expect("read demo");
        let huffman_lookup_table = create_huffman_lookup_table();
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec!["usercmd_command_number".to_string()],
            wanted_other_props: vec![],
            wanted_prop_states: AHashMap::default(),
            wanted_ticks: vec![],
            wanted_events: vec![],
            parse_ents: false,
            parse_projectiles: false,
            parse_grenades: false,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &huffman_lookup_table,
            order_by_steamid: false,
            list_props: false,
            fallback_bytes: None,
        };
        let mut first_pass = FirstPassParser::new(&settings);
        let first_pass_output = first_pass.parse_demo(&demo, false).expect("first pass");
        let mut second_pass = SecondPassParser::new(first_pass_output, 16, true, None).expect("second pass init");
        let transport_state = Arc::clone(&state);
        second_pass.usercmd_transport_sink = Some(Box::new(move |actual| {
            let mut state = transport_state.lock().expect("lock keyed DLL transport oracle");
            begin_keyed_transport(&mut state, actual);
        }));
        let sink_state = Arc::clone(&state);
        second_pass.usercmd_record_sink = Some(Box::new(move |actual| {
            let mut state = sink_state.lock().expect("lock keyed DLL oracle");
            compare_keyed_decoded(&mut state, actual);
        }));
        second_pass.start(&demo).expect("second pass");
        second_pass.usercmd_record_sink = None;
        second_pass.usercmd_transport_sink = None;

        let mut state = state.lock().expect("lock completed streaming DLL oracle");
        finish_keyed_transport(&mut state);
        assert!(state.aligned, "Rust never reached a full usercmd anchor in the DLL oracle");
        assert_eq!(
            state.parser_transports - state.parser_unmatched_bootstrap,
            expected_transport_records,
            "parser transport count differs from the DLL oracle after the initial missing-baseline records"
        );
        let mut unmatched_oracle_records = 0_u64;
        let mut unmatched_oracle_decoded = 0_u64;
        let mut unmatched_oracle_rejected = 0_u64;
        let mut first_unmatched_oracle = None;
        for (key, records) in &state.oracle {
            for record in records {
                unmatched_oracle_records += 1;
                match record.status {
                    DllOracleStatus::Decoded => unmatched_oracle_decoded += 1,
                    DllOracleStatus::Rejected => unmatched_oracle_rejected += 1,
                }
                if first_unmatched_oracle.is_none() {
                    first_unmatched_oracle = Some((*key, record.ordinal, record.status));
                }
            }
        }
        assert_eq!(
            unmatched_oracle_decoded,
            0,
            "DLL oracle contains decoded payloads absent from Rust parsing: records={}, rejected_only={}, first={:?}, parser_bootstrap={}",
            unmatched_oracle_records,
            unmatched_oracle_rejected,
            first_unmatched_oracle,
            state.parser_unmatched_bootstrap,
        );
        assert_eq!(
            state.matched_decoded,
            expected_decoded_records,
            "decoded DLL/Rust record count mismatch: oracle_only_rejected={}, first_oracle_only={:?}",
            unmatched_oracle_rejected,
            first_unmatched_oracle,
        );
        assert_eq!(
            state.matched_rejected + unmatched_oracle_rejected,
            expected_rejected_records,
            "rejected DLL/Rust record count mismatch: matched={}, oracle_only={}",
            state.matched_rejected,
            unmatched_oracle_rejected,
        );
    }

    #[test]
    fn merges_july_usercmd_fields_and_repeated_subticks() {
        let bytes = [
            0x0a, 0x40, 0x10, 0xa5, 0x54, 0x1a, 0x06, 0x08, 0x90, 0x08, 0x10, 0x80, 0x08, 0x22, 0x0a, 0x0d, 0x87, 0x85, 0x29, 0x40, 0x15, 0x36, 0x07, 0xc7,
            0x42, 0x35, 0x00, 0x00, 0x80, 0xbf, 0x50, 0xf8, 0xfb, 0xa7, 0xf7, 0x07, 0x58, 0x51, 0x60, 0x06, 0x92, 0x01, 0x17, 0x0f, 0x02, 0x14, 0x08, 0x80,
            0x08, 0x10, 0x01, 0x1d, 0x00, 0x00, 0xd8, 0x3e, 0x45, 0x3c, 0x4e, 0x11, 0xbf, 0x4d, 0xf0, 0x6a, 0xd5, 0x40,
        ];
        let mut baseline = CsgoUserCmdPb::default();
        baseline.base = Some(Default::default());
        baseline.base.as_mut().unwrap().forwardmove = Some(0.75);
        baseline.base.as_mut().unwrap().viewangles = Some(CMsgQAngle {
            z: Some(17.0),
            ..Default::default()
        });
        baseline.base.as_mut().unwrap().subtick_moves = vec![
            CSubtickMoveStep::default(),
            CSubtickMoveStep {
                button: Some(0x200),
                pressed: Some(true),
                ..Default::default()
            },
        ];

        let command = apply_delta(&baseline, &bytes).unwrap();
        let base = command.base.unwrap();
        let buttons = base.buttons_pb.unwrap();
        assert_eq!(buttons.buttonstate1, Some(0x410));
        assert_eq!(buttons.buttonstate2, Some(0x400));
        assert_eq!(base.forwardmove, Some(0.75));
        assert_eq!(base.leftmove, Some(-1.0));
        assert_eq!(base.viewangles.unwrap().z, Some(17.0));
        assert_eq!(base.subtick_moves.len(), 1);
        assert_eq!(base.subtick_moves[0].button(), 0x400);
        assert!(base.subtick_moves[0].pressed());
        assert!((base.subtick_moves[0].when() - 0.421875).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_nonsequential_repeated_entries_without_mutating_baseline() {
        let baseline = CsgoUserCmdPb::default();
        let delta = [0x12, 0x02, 0x0a, 0x00];
        assert!(apply_delta(&baseline, &delta).is_none());
        assert_eq!(baseline, CsgoUserCmdPb::default());
    }

    #[test]
    fn expands_nested_clear_markers_and_preserves_omitted_fields() {
        let bytes = [
            0x0a, 0x12, 0x10, 0xa6, 0x54, 0x1a, 0x01, 0x17, 0x50, 0xed, 0xf1, 0xc9, 0xdd, 0x03, 0x97, 0x01, 0xa8, 0x01, 0x80, 0x01,
        ];
        let mut baseline = CsgoUserCmdPb::default();
        let mut base = csgoproto::CBaseUserCmdPb::default();
        base.forwardmove = Some(1.0);
        base.buttons_pb = Some(CInButtonStatePb {
            buttonstate1: Some(1),
            buttonstate2: Some(2),
            buttonstate3: Some(3),
        });
        baseline.base = Some(base);

        let command = apply_delta(&baseline, &bytes).unwrap();
        let base = command.base.unwrap();
        assert_eq!(base.forwardmove, Some(1.0));
        let buttons = base.buttons_pb.unwrap();
        assert_eq!(buttons.buttonstate1, Some(1));
        assert_eq!(buttons.buttonstate2, None);
        assert_eq!(buttons.buttonstate2(), 0);
        assert_eq!(buttons.buttonstate3, Some(3));
    }

    #[test]
    fn wire_seven_uses_declared_nonzero_defaults() {
        let mut baseline = CsgoUserCmdPb::default();
        baseline.attack1_start_history_index = Some(4);
        baseline.base = Some(csgoproto::CBaseUserCmdPb {
            pawn_entity_handle: Some(123),
            ..Default::default()
        });

        let command = apply_delta(&baseline, &[0x37, 0x0a, 0x01, 0x77]).unwrap();
        assert_eq!(command.attack1_start_history_index, None);
        assert_eq!(command.attack1_start_history_index(), -1);
        let base = command.base.unwrap();
        assert_eq!(base.pawn_entity_handle, None);
        assert_eq!(base.pawn_entity_handle(), 0x00ff_ffff);
    }

    #[test]
    fn repeated_delta_patches_an_existing_index() {
        let baseline = vec![
            CSubtickMoveStep {
                button: Some(1),
                pressed: Some(true),
                ..Default::default()
            },
            CSubtickMoveStep {
                button: Some(2),
                pressed: Some(true),
                ..Default::default()
            },
        ];
        let delta = CSubtickMoveStep {
            pressed: Some(false),
            ..Default::default()
        };
        let mut encoded_message = Vec::new();
        delta.encode(&mut encoded_message).unwrap();
        let mut payload = Vec::new();
        write_varint((1 << 3) | 2, &mut payload);
        write_varint(encoded_message.len() as u64, &mut payload);
        payload.extend_from_slice(&encoded_message);

        let result = merge_repeated(
            &baseline,
            &[prost::bytes::Bytes::from(payload)],
            MessageSchema::SubtickMove,
            merge_subtick_move,
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].button, Some(1));
        assert_eq!(result[1].button, Some(2));
        assert_eq!(result[1].pressed, Some(false));
    }

    #[test]
    fn repeated_delta_patch_at_zero_preserves_trailing_entries() {
        let baseline = vec![
            CSubtickMoveStep {
                button: Some(1),
                pitch_delta: Some(1.0),
                ..Default::default()
            },
            CSubtickMoveStep {
                button: Some(2),
                pressed: Some(true),
                ..Default::default()
            },
        ];
        let delta = CSubtickMoveStep {
            pitch_delta: Some(2.0),
            ..Default::default()
        };
        let mut encoded_message = Vec::new();
        delta.encode(&mut encoded_message).unwrap();
        let mut payload = Vec::new();
        write_varint(2, &mut payload);
        write_varint(encoded_message.len() as u64, &mut payload);
        payload.extend_from_slice(&encoded_message);

        let result = merge_repeated(
            &baseline,
            &[prost::bytes::Bytes::from(payload)],
            MessageSchema::SubtickMove,
            merge_subtick_move,
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].button, Some(1));
        assert_eq!(result[0].pitch_delta, Some(2.0));
        assert_eq!(result[1], baseline[1]);
    }

    #[test]
    fn repeated_wire_seven_truncates_at_the_index() {
        let baseline = vec![
            CSubtickMoveStep {
                button: Some(1),
                pressed: Some(true),
                ..Default::default()
            },
            CSubtickMoveStep {
                button: Some(2),
                pressed: Some(true),
                ..Default::default()
            },
            CSubtickMoveStep {
                button: Some(3),
                pressed: Some(true),
                ..Default::default()
            },
        ];
        let result = merge_repeated(
            &baseline,
            &[prost::bytes::Bytes::from(vec![0x17])],
            MessageSchema::SubtickMove,
            merge_subtick_move,
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].button, Some(1));
        assert_eq!(result[1].button, Some(2));
        assert_eq!(result[1].pressed, Some(true));
    }

    #[test]
    fn nested_input_history_delta_preserves_omitted_fields() {
        let baseline = CsgoInputHistoryEntryPb {
            player_tick_count: Some(100),
            view_angles: Some(CMsgQAngle {
                y: Some(20.0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let delta = CsgoInputHistoryEntryPb {
            player_tick_fraction: Some(0.5),
            view_angles: Some(CMsgQAngle {
                x: Some(10.0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = merge_input_history(&baseline, delta, &[]);
        assert_eq!(result.player_tick_count, Some(100));
        assert_eq!(result.player_tick_fraction, Some(0.5));
        assert_eq!(result.view_angles.unwrap().x, Some(10.0));
        assert_eq!(result.view_angles.unwrap().y, Some(20.0));
    }
}
