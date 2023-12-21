use ahash::AHashMap;
use csgoproto::netmessages::CSVCMsg_VoiceData;
use opus::Decoder;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::{i16, time::Instant};
use crate::read_bits::DemoParserError;


#[derive(Debug)]
struct VoicePacket {
    pub length: u16,
    pub voice_type: u8,
}
const FRAME_SIZE: usize = 480;
const AVG_BYTES_PER_PACKET: usize = 1600;

pub fn parse_voice_chunk(bytes: &[u8], decoder: &mut Decoder) -> Vec<i16> {
    // based on https://github.com/DandrewsDev/CS2VoiceData
    let mut decoded_bytes = vec![];
    let packet = VoicePacket {
        // sample_rate: u16::from_le_bytes(bytes[9..11].try_into().unwrap()),
        voice_type: u8::from_le_bytes([bytes[11]].try_into().unwrap()),
        length: u16::from_le_bytes(bytes[12..14].try_into().unwrap()),
    };
    if packet.voice_type == 6 {
        let mut ptr = 14;

        // read chunks until chunk_len == 65535
        while ptr < packet.length as usize {
            let mut output = vec![0; FRAME_SIZE];
            let chunk_len = u16::from_le_bytes(bytes[ptr..ptr + 2].try_into().unwrap());
            if chunk_len == 65535 {
                break;
            }
            ptr += 4;
            if let Ok(out_len) = decoder.decode(&bytes[ptr..ptr + chunk_len as usize], &mut output, false) {
                decoded_bytes.extend(&output[..out_len]);
            }
            ptr += chunk_len as usize;
        }
    }
    decoded_bytes
}

fn generate_wav_header(num_channels: u16, sample_rate: u32, bits_per_sample: u16, data_size: u32) -> Vec<u8> {
    let mut header = Vec::new();
    // RIFF header
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&((36 + data_size) as u32).to_le_bytes());
    header.extend_from_slice(b"WAVE");
    // Format chunk
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&(16 as u32).to_le_bytes());
    header.extend_from_slice(&(1 as u16).to_le_bytes());
    header.extend_from_slice(&num_channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&(sample_rate * num_channels as u32 * bits_per_sample as u32 / 8).to_le_bytes());
    header.extend_from_slice(&(num_channels * bits_per_sample / 8).to_le_bytes());
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    // Data chunk
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());
    header
}
pub fn convert_voice_data_to_wav(voice_data: Vec<CSVCMsg_VoiceData>) -> Result<Vec<(String, Vec<u8>)>, DemoParserError> {
    // Group by steamid
    let mut hm: AHashMap<u64, Vec<&CSVCMsg_VoiceData>> = AHashMap::default();
    for data in &voice_data {
        hm.entry(data.xuid()).or_insert(vec![]).push(data);
    }
    // Collect voice data per steamid
    let voice_data_wav: Vec<(String, Vec<u8>)> = hm
        .par_iter()
        .map(|(xuid, data)| {
            let mut decoder = Decoder::new(24000, opus::Channels::Mono).unwrap();
            let mut data_this_player = Vec::with_capacity(AVG_BYTES_PER_PACKET * data.len());
            // add header
            data_this_player.extend(generate_wav_header(1, 24000, 16, data_this_player.len() as u32));
            // add voice data
            for chunk in data {
                data_this_player.extend(
                    parse_voice_chunk(chunk.audio.voice_data(), &mut decoder)
                        .iter()
                        .flat_map(|x| x.to_le_bytes()),
                );
            }
            (xuid.to_string(), data_this_player)
        })
        .collect();
    Ok(voice_data_wav)
}
