use crate::first_pass::read_bits::read_varint;
use crate::first_pass::read_bits::DemoParserError;
use crate::maps::demo_cmd_type_from_int;
use csgoproto::EDemoCommands;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::sync::mpsc::Sender;

pub struct FrameParser {
    pub ptr: usize,
    pub fullpacket_offsets: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Frame {
    pub size: usize,
    pub tick: i32,
    pub frame_starts_at: usize,
    pub frame_ends_at: usize,
    pub is_compressed: bool,
    pub demo_cmd: EDemoCommands,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartEndOffset {
    pub start: usize,
    pub end: usize,
    pub msg_type: StartEndType,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartEndType {
    OK,
    EndOfMessages,
    MultithreadingWasNotOk,
}

impl FrameParser {
    pub fn new() -> Self {
        FrameParser {
            ptr: 0,
            fullpacket_offsets: vec![],
        }
    }
    #[inline(always)]
    fn read_frame(demo_bytes: &[u8], mut ptr: usize) -> Result<Frame, DemoParserError> {
        let frame_starts_at = ptr;
        let cmd = read_varint(demo_bytes, &mut ptr)?;
        let tick = read_varint(demo_bytes, &mut ptr)?;
        let size = read_varint(demo_bytes, &mut ptr)?;

        let msg_type = cmd & !64;
        let is_compressed = (cmd & 64) == 64;
        let demo_cmd = demo_cmd_type_from_int(msg_type as i32)?;

        Ok(Frame {
            frame_ends_at: ptr,
            size: size as usize,
            tick: tick as i32,
            frame_starts_at,
            is_compressed,
            demo_cmd,
        })
    }
    #[inline(always)]
    fn read_frame_mut_ptr(demo_bytes: &[u8], mut ptr: &mut usize) -> Result<Frame, DemoParserError> {
        let frame_starts_at = *ptr;
        let cmd = read_varint(demo_bytes, &mut ptr)?;
        let tick = read_varint(demo_bytes, &mut ptr)?;
        let size = read_varint(demo_bytes, &mut ptr)?;

        let msg_type = cmd & !64;
        let is_compressed = (cmd & 64) == 64;
        let demo_cmd = demo_cmd_type_from_int(msg_type as i32)?;

        Ok(Frame {
            frame_ends_at: *ptr,
            size: size as usize,
            tick: tick as i32,
            frame_starts_at,
            is_compressed,
            demo_cmd,
        })
    }

    pub fn try_find_beginning_of_a_frame(demo_bytes: &[u8], start: usize, end: usize) -> Result<usize, DemoParserError> {
        /*
        We jump into a random offset into the file and do the following:

        1. try to read a Frame
        2. if 1. succeeds jump to the end of that frame
        3. try to read a new frame from the offset found in last frame
        4. repeat steps 1..3 for N times and make sure they all have "increasing by 1" ticks (10001,10002,10003)

        if the Frame fails to read we simply move ptr += 1 and try again
        */
        let mut ptr = start;
        if ptr == 0 || ptr == 16 {
            return Ok(16);
        }
        let mut tot = 0;
        loop {
            tot += 1;
            if ptr >= demo_bytes.len() || tot > 100000 || ptr > end {
                return Err(DemoParserError::ClassNotFound);
            }
            ptr += 1;
            let frame = FrameParser::read_frame(demo_bytes, ptr);
            if let Ok(f) = frame {
                if let Ok(inner) = FrameParser::read_frame(demo_bytes, f.frame_ends_at + f.size) {
                    if let Ok(inner2) = FrameParser::read_frame(demo_bytes, inner.frame_ends_at + inner.size) {
                        if let Ok(inner3) = FrameParser::read_frame(demo_bytes, inner2.frame_ends_at + inner2.size) {
                            if f.tick + 1 == inner.tick && inner.tick + 1 == inner2.tick && inner2.tick + 1 == inner3.tick {
                                return Ok(f.frame_starts_at);
                            }
                        }
                    }
                }
            }
        }
    }
    fn split_file_into_n_chunks(demo_len: usize, n: usize) -> Vec<(usize, usize)> {
        let chunk_size = demo_len / n;
        let mut v = vec![];
        for idx in 0..n {
            v.push((idx * chunk_size, idx * chunk_size + chunk_size));
        }
        v
    }

    pub fn par_start(&mut self, demo_bytes: &[u8], sender: Sender<StartEndOffset>) -> Result<(), DemoParserError> {
        let start_pos = FrameParser::split_file_into_n_chunks(demo_bytes.len(), 12);
        let both: Vec<Vec<StartEndOffset>> = start_pos
            .par_iter()
            .map(|(start, end)| {
                if let Ok(idx) = FrameParser::try_find_beginning_of_a_frame(demo_bytes, *start, *end) {
                    FrameParser::start(demo_bytes, idx, *end, sender.clone()).unwrap()
                } else {
                    vec![]
                }
            })
            .collect();

        let mut offsets: Vec<StartEndOffset> = both.iter().flat_map(|x| x.clone()).collect();
        offsets.sort_by_key(|x| x.end);

        let is_ok = check_all_bytes_are_covered(offsets, demo_bytes.len(), sender.clone());
        let _ = sender.send({
            StartEndOffset {
                start: 0,
                end: 0,
                msg_type: if is_ok {
                    StartEndType::EndOfMessages
                } else {
                    StartEndType::MultithreadingWasNotOk
                },
            }
        });

        Ok(())
    }

    pub fn start(
        demo_bytes: &[u8],
        start: usize,
        end: usize,
        sender: Sender<StartEndOffset>,
    ) -> Result<Vec<StartEndOffset>, DemoParserError> {
        if start == end {
            return Ok(vec![]);
        }
        let mut ptr = start;
        let mut fullpacket_offsets = vec![];
        let mut outs = vec![];
        loop {
            if let Ok(frame) = FrameParser::read_frame_mut_ptr(demo_bytes, &mut ptr) {
                ptr += frame.size;
                if frame.demo_cmd == EDemoCommands::DemFullPacket {
                    if !fullpacket_offsets.is_empty() {
                        let _ = sender.send(StartEndOffset {
                            start: *fullpacket_offsets.last().unwrap_or(&16),
                            end: frame.frame_starts_at,
                            msg_type: StartEndType::OK,
                        });
                        outs.push(StartEndOffset {
                            start: *fullpacket_offsets.last().unwrap_or(&16),
                            end: frame.frame_starts_at,
                            msg_type: StartEndType::OK,
                        })
                    }
                    // The only time we send if no fullpackets have been found is the first fullpacket
                    if fullpacket_offsets.is_empty() && start == 16 {
                        let _ = sender.send(StartEndOffset {
                            start: 16,
                            end: frame.frame_starts_at,
                            msg_type: StartEndType::OK,
                        });
                        outs.push(StartEndOffset {
                            start: 16,
                            end: frame.frame_starts_at,
                            msg_type: StartEndType::OK,
                        })
                    }
                    fullpacket_offsets.push(frame.frame_starts_at);

                    // If we are past our designated end and we find a fullpacket we exit
                    if ptr > end {
                        return Ok(outs);
                    }
                }
            } else {
                return Ok(outs);
            }
        }
    }
}
fn check_all_bytes_are_covered(mut sorted_offsets: Vec<StartEndOffset>, demo_len: usize, sender: Sender<StartEndOffset>) -> bool {
    sorted_offsets.sort_by_key(|o| o.start);
    sorted_offsets.dedup();
    let mut send_ok = true;
    // Check that no gaps in the ranges
    for w in sorted_offsets.windows(2) {
        if w[0].end != w[1].start {
            return false;
        }
    }

    // Make sure we start at 16 and end at file length
    let smallest_start_byte = sorted_offsets.iter().map(|x| x.start).min();
    let largest_end_byte = sorted_offsets.iter().map(|x| x.end).max();

    match largest_end_byte {
        Some(idx) => {
            if idx != demo_len {
                let res = sender.send({
                    StartEndOffset {
                        start: idx,
                        end: demo_len,
                        msg_type: StartEndType::OK,
                    }
                });
                send_ok = res.is_ok();
                sorted_offsets.push(StartEndOffset {
                    start: idx,
                    end: demo_len,
                    msg_type: StartEndType::OK,
                })
            }
        }
        _ => {
            return false;
        }
    }
    let largest_end_byte = sorted_offsets.iter().map(|x| x.end).max();
    if let (Some(smallest), Some(largest)) = (smallest_start_byte, largest_end_byte) {
        if smallest == 16 && largest == demo_len && send_ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{check_all_bytes_are_covered, StartEndOffset};
    use crate::first_pass::frameparser::StartEndType;
    use std::sync::mpsc::channel;

    #[test]
    fn test_byte_coverage_ok() {
        let (s, _) = channel();
        let sorted_offsets = vec![
            StartEndOffset {
                start: 16,
                end: 20,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 20,
                end: 30,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 30,
                end: 40,
                msg_type: StartEndType::OK,
            },
        ];
        let is_ok = check_all_bytes_are_covered(sorted_offsets, 40, s);
        assert_eq!(is_ok, true);
    }
    #[test]
    fn test_byte_coverage_not_ok_range_gap() {
        let (s, _) = channel();
        let sorted_offsets = vec![
            StartEndOffset {
                start: 16,
                end: 20,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 20,
                end: 30,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 40,
                end: 50,
                msg_type: StartEndType::OK,
            },
        ];
        let is_ok = check_all_bytes_are_covered(sorted_offsets, 30, s);
        assert_eq!(is_ok, false);
    }
    #[test]
    fn test_byte_coverage_not_ok_missing_start_byte() {
        let (s, _) = channel();
        let sorted_offsets = vec![
            StartEndOffset {
                start: 14,
                end: 20,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 20,
                end: 30,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 30,
                end: 40,
                msg_type: StartEndType::OK,
            },
        ];
        let is_ok = check_all_bytes_are_covered(sorted_offsets, 50, s);
        assert_eq!(is_ok, false);
    }
    #[test]
    fn test_byte_coverage_not_ok_missing_end_byte() {
        let (s, _) = channel();
        let sorted_offsets = vec![
            StartEndOffset {
                start: 16,
                end: 20,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 20,
                end: 30,
                msg_type: StartEndType::OK,
            },
            StartEndOffset {
                start: 30,
                end: 40,
                msg_type: StartEndType::OK,
            },
        ];
        let is_ok = check_all_bytes_are_covered(sorted_offsets, 33, s);
        assert_eq!(is_ok, false);
    }
}
