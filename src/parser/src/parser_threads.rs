use super::netmessage_types;
use super::read_bits::DemoParserError;
use crate::netmessage_types::netmessage_type_from_int;
use crate::netmessage_types::NetmessageType;
use crate::parser_thread_settings::ParserThread;
use crate::read_bits::Bitreader;
use bitter::BitReader;
use csgoproto::demo::*;
use csgoproto::netmessages::*;
use csgoproto::networkbasetypes::CNETMsg_Tick;
use netmessage_types::NetmessageType::*;
use protobuf::Message;
use snap::raw::Decoder as SnapDecoder;
use EDemoCommands::*;

impl ParserThread {
    pub fn start(&mut self) -> Result<(), DemoParserError> {
        let started_at = self.ptr;
        loop {
            let cmd = self.read_varint()?;
            let tick = self.read_varint()?;
            let size = self.read_varint()?;

            self.tick = tick as i32;
            self.packets_parsed += 1;

            if self.ptr + size as usize >= self.bytes.get_len() {
                break;
            }

            let msg_type = cmd & !64;
            let is_compressed = (cmd & 64) == 64;
            let demo_cmd = demo_cmd_type_from_int(msg_type as i32).unwrap();

            if demo_cmd == DEM_AnimationData || demo_cmd == DEM_SendTables || demo_cmd == DEM_StringTables {
                self.ptr += size as usize;
                continue;
            }
            let bytes = match is_compressed {
                true => match SnapDecoder::new().decompress_vec(self.read_n_bytes(size)?) {
                    Ok(b) => b,
                    Err(e) => return Err(DemoParserError::DecompressionFailure(format!("{}", e))),
                },
                false => self.read_n_bytes(size)?.to_vec(),
            };

            let ok = match demo_cmd {
                DEM_SignonPacket => self.parse_packet(&bytes),
                DEM_Packet => self.parse_packet(&bytes),
                DEM_FullPacket => {
                    match self.parse_all_packets {
                        true => {}
                        false => {
                            if self.fullpackets_parsed == 0 && started_at != 16 {
                                self.parse_full_packet(&bytes)?;
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
        println!("started_at: {} ended_at: {}", started_at, self.ptr);
        Ok(())
    }
    fn packet_orderer(&self, packet: &NetmessageType) -> i32 {
        match packet {
            svc_CreateStringTable => 1001,
            svc_UpdateStringTable => 1000,
            GE_Source1LegacyGameEvent => 999,
            svc_PacketEntities => 90,
            _ => 0,
        }
    }
    pub fn parse_packet(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let packet: CDemoPacket = match Message::parse_from_bytes(bytes) {
            Err(_e) => return Err(DemoParserError::MalformedMessage),
            Ok(p) => p,
        };
        let packet_data = packet.data.unwrap();
        let mut bitreader = Bitreader::new(&packet_data);
        // Inner loop
        let mut msgs = vec![];

        while bitreader.reader.has_bits_remaining(8) {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            let msg_bytes = bitreader.read_n_bytes(size as usize)?;
            msgs.push((msg_bytes, netmessage_type_from_int(msg_type as i32)));
        }

        msgs.sort_by_key(|x| self.packet_orderer(&x.1));

        for (msg_bytes, msg_type) in msgs {
            let ok = match msg_type {
                svc_PacketEntities => self.parse_packet_ents(&msg_bytes),
                svc_CreateStringTable => self.parse_create_stringtable(&msg_bytes),
                svc_UpdateStringTable => self.update_string_table(&msg_bytes),
                svc_ServerInfo => self.parse_server_info(&msg_bytes),
                GE_Source1LegacyGameEvent => self.parse_event(&msg_bytes),
                CS_UM_SendPlayerItemDrops => self.parse_item_drops(&msg_bytes),
                CS_UM_EndOfMatchAllPlayersData => self.parse_player_end_msg(&msg_bytes),
                UM_SayText2 => self.parse_chat_messages(&msg_bytes),
                net_SetConVar => self.create_custom_event_parse_convars(&msg_bytes),
                CS_UM_PlayerStatsUpdate => self.parse_player_stats_update(&msg_bytes),
                CS_UM_ServerRankUpdate => self.create_custom_event_rank_update(&msg_bytes),
                net_Tick => self.parse_net_tick(&msg_bytes),
                svc_ClearAllStringTables => self.clear_stringtables(),
                _ => Ok(()),
            };
            ok?;
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
    pub fn parse_full_packet(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let full_packet: CDemoFullPacket = match Message::parse_from_bytes(bytes) {
            Err(_e) => return Err(DemoParserError::MalformedMessage),
            Ok(p) => p,
        };
        for item in &full_packet.string_table.tables {
            if item.table_name == Some("instancebaseline".to_string()) {
                for i in &item.items {
                    let k = i.str().parse::<u32>().unwrap_or(999999);
                    self.baselines.insert(k, i.data.as_ref().unwrap().clone());
                }
            }
        }
        let p = full_packet.packet.0.unwrap();
        let mut bitreader = Bitreader::new(p.data());
        // Inner loop
        while bitreader.reader.bits_remaining().unwrap() > 8 {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            let msg_bytes = bitreader.read_n_bytes(size as usize)?;

            let ok = match netmessage_type_from_int(msg_type as i32) {
                svc_PacketEntities => self.parse_packet_ents(&msg_bytes),
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
pub fn demo_cmd_type_from_int(value: i32) -> ::std::option::Option<EDemoCommands> {
    match value {
        -1 => ::std::option::Option::Some(EDemoCommands::DEM_Error),
        0 => ::std::option::Option::Some(EDemoCommands::DEM_Stop),
        1 => ::std::option::Option::Some(EDemoCommands::DEM_FileHeader),
        2 => ::std::option::Option::Some(EDemoCommands::DEM_FileInfo),
        3 => ::std::option::Option::Some(EDemoCommands::DEM_SyncTick),
        4 => ::std::option::Option::Some(EDemoCommands::DEM_SendTables),
        5 => ::std::option::Option::Some(EDemoCommands::DEM_ClassInfo),
        6 => ::std::option::Option::Some(EDemoCommands::DEM_StringTables),
        7 => ::std::option::Option::Some(EDemoCommands::DEM_Packet),
        8 => ::std::option::Option::Some(EDemoCommands::DEM_SignonPacket),
        9 => ::std::option::Option::Some(EDemoCommands::DEM_ConsoleCmd),
        10 => ::std::option::Option::Some(EDemoCommands::DEM_CustomData),
        11 => ::std::option::Option::Some(EDemoCommands::DEM_CustomDataCallbacks),
        12 => ::std::option::Option::Some(EDemoCommands::DEM_UserCmd),
        13 => ::std::option::Option::Some(EDemoCommands::DEM_FullPacket),
        14 => ::std::option::Option::Some(EDemoCommands::DEM_SaveGame),
        15 => ::std::option::Option::Some(EDemoCommands::DEM_SpawnGroups),
        16 => ::std::option::Option::Some(EDemoCommands::DEM_AnimationData),
        17 => ::std::option::Option::Some(EDemoCommands::DEM_Max),
        64 => ::std::option::Option::Some(EDemoCommands::DEM_IsCompressed),
        _ => ::std::option::Option::None,
    }
}
