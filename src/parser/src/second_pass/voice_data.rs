// there has to be a better way to disable multiple :D
#[cfg(feature = "voice")]
use crate::first_pass::read_bits::DemoParserError;
#[cfg(feature = "voice")]
use ahash::AHashMap;
#[cfg(feature = "voice")]
use csgoproto::{CsvcMsgVoiceData, VoiceDataFormatT::*};
#[cfg(feature = "voice")]
use opus::Decoder;
#[cfg(feature = "voice")]
use rayon::iter::IntoParallelRefIterator;
#[cfg(feature = "voice")]
use rayon::iter::ParallelIterator;

#[cfg(feature = "voice")]
#[derive(Debug)]
struct VoicePacket {
    pub length: u16,
    pub voice_type: u8,
}
#[cfg(feature = "voice")]
const FRAME_SIZE: usize = 480;
#[cfg(feature = "voice")]
const AVG_BYTES_PER_PACKET: usize = 1600;

#[cfg(feature = "voice")]
pub fn parse_voice_chunk_old_format(bytes: &[u8], decoder: &mut Decoder) -> Result<Vec<i16>, DemoParserError> {
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
            let output = vec![0; FRAME_SIZE];
            let chunk_len = u16::from_le_bytes(bytes[ptr..ptr + 2].try_into().unwrap());
            if chunk_len == 65535 {
                break;
            }
            ptr += 4;
            match decoder.decode(&bytes, &mut decoded_bytes, false) {
                Ok(n) => decoded_bytes.extend(&output[..n]),
                Err(_) => return Err(DemoParserError::MalformedVoicePacket),
            };
            ptr += chunk_len as usize;
        }
    }
    Ok(decoded_bytes)
}

#[cfg(feature = "voice")]
pub fn parse_voice_chunk_new_format(bytes: &[u8], decoder: &mut Decoder) -> Result<Vec<i16>, DemoParserError> {
    let mut decoded_bytes = vec![0; 1024];
    let n = match decoder.decode(&bytes, &mut decoded_bytes, false) {
        Ok(n) => n,
        Err(_) => return Err(DemoParserError::MalformedVoicePacket),
    };
    decoded_bytes.truncate(n);
    Ok(decoded_bytes)
}
#[cfg(feature = "voice")]
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
#[cfg(feature = "voice")]
pub fn convert_voice_data_to_wav(voice_data: Vec<CsvcMsgVoiceData>) -> Result<Vec<(String, Vec<u8>)>, DemoParserError> {
    // Group by steamid
    let mut hm: AHashMap<u64, Vec<&CsvcMsgVoiceData>> = AHashMap::default();
    for data in &voice_data {
        hm.entry(data.xuid()).or_insert(vec![]).push(data);
    }
    // Collect voice data per steamid
    let voice_data_wav: Vec<Result<(String, Vec<u8>), DemoParserError>> = hm
        .par_iter()
        .map(|(xuid, data)| {
            let mut decoder = Decoder::new(48000, opus::Channels::Mono).unwrap();
            let mut data_this_player = Vec::with_capacity(AVG_BYTES_PER_PACKET * data.len());
            // add voice data
            for chunk in data {
                if let Some(audio) = &chunk.audio {
                    match audio.format() {
                        VoicedataFormatOpus => data_this_player.extend(
                            parse_voice_chunk_new_format(audio.voice_data(), &mut decoder)?
                                .iter()
                                .flat_map(|x| x.to_le_bytes()),
                        ),
                        VoicedataFormatSteam => data_this_player.extend(
                            parse_voice_chunk_new_format(audio.voice_data(), &mut decoder)?
                                .iter()
                                .flat_map(|x| x.to_le_bytes()),
                        ),
                        VoicedataFormatEngine => {
                            return Err(DemoParserError::UnkVoiceFormat);
                        }
                    };
                }
            }
            let mut out = vec![];
            out.extend(generate_wav_header(1, 48000, 16, data_this_player.len() as u32));
            out.extend(data_this_player);
            Ok((xuid.to_string(), out))
        })
        .collect();

    // Check for errors
    let mut ok_packets = vec![];
    for data in voice_data_wav {
        ok_packets.push(data?);
    }
    Ok(ok_packets)
}
