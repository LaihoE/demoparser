use crate::first_pass::read_bits::read_varint;
use crate::first_pass::read_bits::DemoParserError;
use crate::maps::demo_cmd_type_from_int;
use csgoproto::demo::EDemoCommands::DEM_Stop;
use itertools::Itertools;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Instant;
// use varint_simd::decode_four_unsafe;
use csgoproto::demo::EDemoCommands;

pub struct FrameParser {
    pub ptr: usize,
    pub frames: Vec<Frame>,
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
}

impl FrameParser {
    pub fn new() -> Self {
        FrameParser {
            ptr: 0,
            frames: vec![],
            fullpacket_offsets: vec![],
        }
    }
    #[inline(always)]
    fn read_frame(&mut self, demo_bytes: &[u8]) -> Result<Frame, DemoParserError> {
        //let (cmd, tick, size, _, n1, n2, n3, _, _) = unsafe { decode_four_unsafe::<u8, u32, u32, u8>(&demo_bytes[self.ptr]) };
        //let frame_starts_at = self.ptr;
        //self.ptr += n1 as usize;
        //self.ptr += n2 as usize;
        //self.ptr += n3 as usize;

        let frame_starts_at = self.ptr;
        let cmd = read_varint(demo_bytes, &mut self.ptr)?;
        let tick = read_varint(demo_bytes, &mut self.ptr)?;
        let size = read_varint(demo_bytes, &mut self.ptr)?;

        let msg_type = cmd & !64;
        let is_compressed = (cmd & 64) == 64;
        let demo_cmd = demo_cmd_type_from_int(msg_type as i32)?;

        Ok(Frame {
            frame_ends_at: self.ptr,
            size: size as usize,
            tick: tick as i32,
            frame_starts_at: frame_starts_at,
            is_compressed: is_compressed,
            demo_cmd: demo_cmd,
        })
    }
    #[inline(always)]
    fn read_frame2(demo_bytes: &[u8], mut ptr: usize) -> Result<Frame, DemoParserError> {
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
            frame_starts_at: frame_starts_at,
            is_compressed: is_compressed,
            demo_cmd: demo_cmd,
        })
    }
    #[inline(always)]
    fn read_frame_mut_ptr(demo_bytes: &[u8], mut ptr: &mut usize) -> Result<Frame, DemoParserError> {
        if *ptr >= demo_bytes.len() {
            return Err(DemoParserError::ClassNotFound);
        }
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
            frame_starts_at: frame_starts_at,
            is_compressed: is_compressed,
            demo_cmd: demo_cmd,
        })
    }
    pub fn try_find_beginning(demo_bytes: &[u8], start: usize, end: usize) -> Result<usize, DemoParserError> {
        /*
        We jump into a random offset into the file and do the following:

        1. try to read a Frame
        2. if 1. succeeds jump to the end of that frame
        3. try to read a new frame from the offset found in last frame
        4. repeat steps 1..3 for N times and make sure they all have increasing ticks

        if the Frame fails to read we simply move ptr += 1
        */

        let mut ptr = start;
        if ptr == 0 || ptr == 16 {
            return Ok(16);
        }
        let mut tot = 0;
        loop {
            tot += 1;
            if ptr >= demo_bytes.len() || tot > 100000 {
                return Err(DemoParserError::ClassNotFound);
            }
            ptr += 1;
            let frame = FrameParser::read_frame2(demo_bytes, ptr);
            if let Ok(f) = frame {
                if let Ok(inner) = FrameParser::read_frame2(demo_bytes, f.frame_ends_at + f.size) {
                    if let Ok(inner2) = FrameParser::read_frame2(demo_bytes, inner.frame_ends_at + inner.size) {
                        if let Ok(inner3) = FrameParser::read_frame2(demo_bytes, inner2.frame_ends_at + inner2.size) {
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

    pub fn par_start(&mut self, demo_bytes: &[u8]) -> Result<Vec<StartEndOffset>, DemoParserError> {
        let b = Instant::now();
        let start_pos = FrameParser::split_file_into_n_chunks(demo_bytes.len(), 8);
        let mut fullpacket_offsets: Vec<usize> = start_pos
            .iter()
            .map(|(start, end)| (FrameParser::try_find_beginning(demo_bytes, *start, *end)))
            .filter_map(|x| x.ok())
            .collect();
        fullpacket_offsets.sort();
        fullpacket_offsets.dedup();

        let mut start_positions = vec![];
        let mut last = 0;
        for found_pos in fullpacket_offsets {
            start_positions.push((last, found_pos));
            last = found_pos;
        }
        start_positions.push((last, demo_bytes.len()));

        let both: Vec<Vec<usize>> = start_positions
            .par_iter()
            .map(|(start, end)| {
                if let Ok(idx) = FrameParser::try_find_beginning(demo_bytes, *start, *end) {
                    FrameParser::start(demo_bytes, idx, *end).unwrap()
                } else {
                    vec![]
                }
            })
            .collect();
        let mut offsets: Vec<usize> = both.iter().flat_map(|x| x.clone()).collect();
        offsets.sort();
        offsets.dedup();

        let mut outputs = vec![];
        outputs.push(StartEndOffset {
            start: 16,
            end: offsets.first().cloned().unwrap_or(usize::MAX),
        });
        for window in offsets.windows(2) {
            if window.len() == 2 {
                outputs.push(StartEndOffset {
                    start: window[0],
                    end: window[1],
                });
            }
        }
        // println!("FP TOOK {:?}", b.elapsed());
        Ok(outputs)
    }

    pub fn start(demo_bytes: &[u8], start: usize, end: usize) -> Result<(Vec<usize>), DemoParserError> {
        let mut ptr = start;
        let mut fullpacket_offsets = vec![];
        loop {
            if let Ok(frame) = FrameParser::read_frame_mut_ptr(demo_bytes, &mut ptr) {
                ptr += frame.size;
                if frame.demo_cmd == csgoproto::demo::EDemoCommands::DEM_FullPacket {
                    fullpacket_offsets.push(frame.frame_starts_at);
                }
                if ptr > end || frame.demo_cmd == csgoproto::demo::EDemoCommands::DEM_Stop {
                    return Ok(fullpacket_offsets);
                }
            } else {
                return Ok(fullpacket_offsets);
            }
        }
    }
}
