//! Decoder for `CMsgServerUserCmd.delta_data` payloads emitted by CS2's
//! `codegen_delta_encoder`.
//!
//! Singular fields retain protobuf wire encoding except for wire type 7,
//! which resets a field to its declared default. The repeated input-history
//! and subtick fields use the replacement-list encoding observed in current
//! CS2 demos. Unknown or malformed operations fail the whole delta so callers
//! can keep the previous per-player baseline unchanged.

use csgoproto::CBaseUserCmdExecutionNotes;
use csgoproto::CInButtonStatePb;
use csgoproto::CMsgQAngle;
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
        }
    }

    fn child(self, field: u64) -> Option<Self> {
        match (self, field) {
            (Self::CsgoUserCmd, 1) => Some(Self::BaseUserCmd),
            (Self::BaseUserCmd, 3) => Some(Self::Buttons),
            (Self::BaseUserCmd, 4) => Some(Self::QAngle),
            (Self::InputHistory, 2 | 69) => Some(Self::QAngle),
            _ => None,
        }
    }

    fn reset_fields(self) -> &'static [(u64, u8)] {
        match self {
            Self::CsgoUserCmd => &[(1, 2), (2, 2), (6, 0), (7, 0), (9, 0), (11, 0), (12, 0), (13, 0)],
            Self::BaseUserCmd => &[
                (1, 0),
                (2, 0),
                (3, 2),
                (4, 2),
                (5, 5),
                (6, 5),
                (7, 5),
                (8, 0),
                (9, 0),
                (10, 0),
                (11, 0),
                (12, 0),
                (14, 0),
                (17, 0),
                (18, 2),
                (19, 2),
                (20, 0),
                (21, 0),
                (22, 2),
            ],
            Self::Buttons => &[(1, 0), (2, 0), (3, 0)],
            Self::QAngle => &[(1, 5), (2, 5), (3, 5)],
            Self::InputHistory => &[(2, 2), (4, 0), (5, 5), (6, 0), (7, 5), (64, 0), (65, 0)],
            Self::SubtickMove => &[(1, 0), (2, 0), (3, 5), (4, 5), (5, 5), (8, 5), (9, 5)],
        }
    }

    fn write_default(self, field: u64, wire_type: u8, out: &mut Vec<u8>) -> Option<()> {
        match wire_type {
            0 => {
                let value = match (self, field) {
                    (Self::CsgoUserCmd, 6 | 7) | (Self::InputHistory, 65) => u64::MAX,
                    (Self::BaseUserCmd, 14) => 0x00ff_ffff,
                    _ => 0,
                };
                write_varint(value, out);
            }
            1 => out.extend_from_slice(&[0; 8]),
            2 => {
                let nested = match self.child(field) {
                    Some(child) => child.explicit_defaults()?,
                    None => Vec::new(),
                };
                write_varint(nested.len() as u64, out);
                out.extend_from_slice(&nested);
            }
            5 => out.extend_from_slice(&[0; 4]),
            _ => return None,
        }
        Some(())
    }

    fn explicit_defaults(self) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        for &(field, wire_type) in self.reset_fields() {
            write_varint((field << 3) | u64::from(wire_type), &mut out);
            self.write_default(field, wire_type, &mut out)?;
        }
        Some(out)
    }
}

fn sanitize_message(mut bytes: &[u8], schema: MessageSchema) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(bytes.len());
    while !bytes.is_empty() {
        let key = read_varint(&mut bytes)?;
        let field = key >> 3;
        let wire_type = (key & 0x07) as u8;
        if field == 0 {
            return None;
        }

        if wire_type == 7 {
            let normal_wire_type = schema.field_wire_type(field)?;
            write_varint((field << 3) | u64::from(normal_wire_type), &mut out);
            schema.write_default(field, normal_wire_type, &mut out)?;
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
                    sanitize_message(value, child)?
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
    Some(out)
}

fn decode_repeated<M>(payloads: &[prost::bytes::Bytes], schema: MessageSchema) -> Option<Vec<M>>
where
    M: Message + Default,
{
    let mut messages = Vec::new();
    for payload in payloads {
        let mut bytes = payload.as_ref();
        if bytes.first() == Some(&0x0f) {
            messages.clear();
            bytes = &bytes[1..];
        }
        while !bytes.is_empty() {
            let key = read_varint(&mut bytes)?;
            if key & 0x07 != 2 {
                return None;
            }
            let index = usize::try_from(key >> 3).ok()?;
            if index != messages.len() {
                return None;
            }
            let length = usize::try_from(read_varint(&mut bytes)?).ok()?;
            let (message, rest) = bytes.split_at_checked(length)?;
            let message = sanitize_message(message, schema)?;
            messages.push(M::decode(message.as_slice()).ok()?);
            bytes = rest;
        }
    }
    Some(messages)
}

fn replace_if_some<T>(target: &mut Option<T>, value: Option<T>) {
    if let Some(value) = value {
        *target = Some(value);
    }
}

fn merge_buttons(target: &mut Option<CInButtonStatePb>, delta: CInButtonStatePb) {
    let target = target.get_or_insert_default();
    replace_if_some(&mut target.buttonstate1, delta.buttonstate1);
    replace_if_some(&mut target.buttonstate2, delta.buttonstate2);
    replace_if_some(&mut target.buttonstate3, delta.buttonstate3);
}

fn merge_qangle(target: &mut Option<CMsgQAngle>, delta: CMsgQAngle) {
    let target = target.get_or_insert_default();
    replace_if_some(&mut target.x, delta.x);
    replace_if_some(&mut target.y, delta.y);
    replace_if_some(&mut target.z, delta.z);
}

pub(super) fn apply_delta(baseline: &CsgoUserCmdPb, delta_data: &[u8]) -> Option<CsgoUserCmdPb> {
    let sanitized = sanitize_message(delta_data, MessageSchema::CsgoUserCmd)?;
    let delta = DeltaCsgoUserCmdPb::decode(sanitized.as_slice()).ok()?;
    let mut next = baseline.clone();

    if !delta.input_history_delta.is_empty() {
        next.input_history = decode_repeated(&delta.input_history_delta, MessageSchema::InputHistory)?;
    }
    replace_if_some(&mut next.attack1_start_history_index, delta.attack1_start_history_index);
    replace_if_some(&mut next.attack2_start_history_index, delta.attack2_start_history_index);
    replace_if_some(&mut next.left_hand_desired, delta.left_hand_desired);
    replace_if_some(&mut next.is_predicting_body_shot_fx, delta.is_predicting_body_shot_fx);
    replace_if_some(&mut next.is_predicting_head_shot_fx, delta.is_predicting_head_shot_fx);
    replace_if_some(&mut next.is_predicting_kill_ragdolls, delta.is_predicting_kill_ragdolls);

    if let Some(delta_base) = delta.base {
        let base = next.base.get_or_insert_default();
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
            base.execution_notes = Some(CBaseUserCmdExecutionNotes::decode(notes).ok()?);
        }
        if !delta_base.subtick_moves_delta.is_empty() {
            base.subtick_moves = decode_repeated(&delta_base.subtick_moves_delta, MessageSchema::SubtickMove)?;
        }
    }

    Some(next)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(buttons.buttonstate2, Some(0));
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
        assert_eq!(command.attack1_start_history_index, Some(-1));
        assert_eq!(command.base.unwrap().pawn_entity_handle, Some(0x00ff_ffff));
    }
}
