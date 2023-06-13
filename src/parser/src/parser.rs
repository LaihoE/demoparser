use super::netmessage_types;
use super::read_bits::DemoParserError;
use crate::netmessage_types::netmessage_type_from_int;
use crate::parser_settings::Parser;
use crate::read_bits::Bitreader;

use bitter::BitReader;
use csgoproto::demo::*;
use csgoproto::netmessages::*;
use netmessage_types::NetmessageType::*;
use protobuf::Message;
use snap::raw::Decoder as SnapDecoder;
use EDemoCommands::*;

// The parser struct is defined in parser_settings.rs
impl<'a> Parser<'a> {
    pub fn start(&mut self) -> Result<(), DemoParserError> {
        let file_length = self.bytes.len();
        println!("{}", self.ptr);
        // Header (there is a longer header as a DEM_FileHeader msg below)
        //let header = self.read_n_bytes(16)?;
        //Parser::handle_short_header(file_length, header)?;
        // Outer loop that continues trough the file, until "DEM_Stop" msg
        loop {
            println!("A");

            let cmd = self.read_varint()?;
            let tick = self.read_varint()?;
            let size = self.read_varint()?;
            self.tick = tick as i32;

            println!("{} {} {} ", cmd, tick, size);

            let msg_type = cmd & !64;
            let is_compressed = (cmd & 64) == 64;
            let bytes = match is_compressed {
                true => SnapDecoder::new()
                    .decompress_vec(self.read_n_bytes(size)?)
                    .unwrap(),
                false => self.read_n_bytes(size)?.to_vec(),
            };

            let ok = match demo_cmd_type_from_int(msg_type as i32).unwrap() {
                DEM_Packet => self.parse_packet(&bytes),
                DEM_FileHeader => self.parse_header(&bytes),
                DEM_FileInfo => self.parse_file_info(&bytes),
                DEM_SendTables => self.parse_classes(&bytes),
                // DEM_ClassInfo => self.parse_class_info(&bytes),
                DEM_SignonPacket => self.parse_packet(&bytes),
                DEM_UserCmd => self.parse_user_command_cmd(&bytes),
                DEM_StringTables => self.parse_stringtable_cmd(&bytes),
                DEM_Stop => break,
                _ => Ok(()),
            };
            ok?;
            //self.collect_entities();
        }
        Ok(())
    }

    pub fn parse_packet(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let packet: CDemoPacket = Message::parse_from_bytes(bytes).unwrap();
        let packet_data = packet.data.unwrap();
        let mut bitreader = Bitreader::new(&packet_data);
        // Inner loop
        while bitreader.reader.has_bits_remaining(8) {
            let msg_type = bitreader.read_u_bit_var()?;
            let size = bitreader.read_varint()?;
            let msg_bytes = bitreader.read_n_bytes(size as usize)?;

            let ok = match netmessage_type_from_int(msg_type as i32) {
                svc_PacketEntities => self.parse_packet_ents(&msg_bytes),
                svc_ServerInfo => self.parse_server_info(&msg_bytes),
                svc_CreateStringTable => self.parse_create_stringtable(&msg_bytes),
                svc_UpdateStringTable => self.update_string_table(&msg_bytes),
                //GE_Source1LegacyGameEventList => self.parse_game_event_list(&msg_bytes),
                //GE_Source1LegacyGameEvent => self.parse_event(&msg_bytes),
                CS_UM_SendPlayerItemDrops => self.parse_item_drops(&msg_bytes),
                CS_UM_EndOfMatchAllPlayersData => self.parse_player_end_msg(&msg_bytes),
                UM_SayText2 => self.parse_chat_messages(&msg_bytes),
                net_SetConVar => self.parse_convars(&msg_bytes),
                CS_UM_PlayerStatsUpdate => self.parse_player_stats_update(&msg_bytes),
                _ => Ok(()),
            };
            ok?
        }
        Ok(())
    }
    pub fn parse_stringtable_cmd(&mut self, data: &[u8]) -> Result<(), DemoParserError> {
        // Why do we use this and not just create/update stringtables??
        let tables: CDemoStringTables = Message::parse_from_bytes(data).unwrap();
        for item in &tables.tables {
            if item.table_name.as_ref().unwrap() == "instancebaseline" {
                for i in &item.items {
                    let k = i.str().parse::<u32>().unwrap_or(999999);
                    self.baselines.insert(k, i.data.as_ref().unwrap().clone());
                }
            }
        }
        Ok(())
    }
    pub fn parse_server_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let server_info: CSVCMsg_ServerInfo = Message::parse_from_bytes(bytes).unwrap();
        let class_count = server_info.max_classes();
        self.cls_bits = Some((class_count as f32 + 1.).log2().ceil() as u32);
        Ok(())
    }
    pub fn parse_classes(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if self.parse_entities {
            let tables: CDemoSendTables = Message::parse_from_bytes(bytes).unwrap();
            //self.parse_sendtable(tables)?;
        }
        Ok(())
    }
    fn handle_short_header(file_len: usize, bytes: &[u8]) -> Result<(), DemoParserError> {
        match std::str::from_utf8(&bytes[..8]) {
            Ok(magic) => match magic {
                "PBDEMS2\0" => {}
                "HL2DEMO\0" => {
                    return Err(DemoParserError::Source1DemoError);
                }
                _ => {
                    return Err(DemoParserError::UnknownFile);
                }
            },
            Err(_) => {}
        };
        // hmmmm not sure where the 18 comes from if the header is only 16?
        // can be used to check that file ends early
        let file_length_expected = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) + 18;
        let missing_bytes = file_length_expected - file_len as u32;
        if missing_bytes != 0 {
            return Err(DemoParserError::DemoEndsEarly(format!(
                "demo ends early. Expected legth: {}, file lenght: {}. Missing: {:.2}%",
                file_length_expected,
                file_len,
                100.0 - (file_len as f32 / file_length_expected as f32 * 100.0),
            )));
        }
        // seems to be byte offset to where DEM_END command happens. After that comes Spawngroups and fileinfo. odd...
        let _no_clue_what_this_is = i32::from_le_bytes(bytes[12..].try_into().unwrap());
        Ok(())
    }
    pub fn parse_user_command_cmd(&mut self, _data: &[u8]) -> Result<(), DemoParserError> {
        // Only in pov demos. Maybe implement sometime. Includes buttons etc.
        Ok(())
    }
    pub fn parse_header(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let header: CDemoFileHeader = Message::parse_from_bytes(bytes).unwrap();
        self.header.insert(
            "demo_file_stamp".to_string(),
            header.demo_file_stamp().to_string(),
        );
        self.header.insert(
            "demo_version_guid".to_string(),
            header.demo_version_guid().to_string(),
        );
        self.header.insert(
            "network_protocol".to_string(),
            header.network_protocol().to_string(),
        );
        self.header
            .insert("server_name".to_string(), header.server_name().to_string());
        self.header
            .insert("client_name".to_string(), header.client_name().to_string());
        self.header
            .insert("map_name".to_string(), header.map_name().to_string());
        self.header.insert(
            "game_directory".to_string(),
            header.game_directory().to_string(),
        );
        self.header.insert(
            "fullpackets_version".to_string(),
            header.fullpackets_version().to_string(),
        );
        self.header.insert(
            "allow_clientside_entities".to_string(),
            header.allow_clientside_entities().to_string(),
        );
        self.header.insert(
            "allow_clientside_particles".to_string(),
            header.allow_clientside_particles().to_string(),
        );
        self.header.insert(
            "allow_clientside_particles".to_string(),
            header.allow_clientside_particles().to_string(),
        );
        self.header
            .insert("addons".to_string(), header.addons().to_string());
        self.header.insert(
            "demo_version_name".to_string(),
            header.demo_version_name().to_string(),
        );
        self.header
            .insert("addons".to_string(), header.addons().to_string());
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
        16 => ::std::option::Option::Some(EDemoCommands::DEM_Max),
        64 => ::std::option::Option::Some(EDemoCommands::DEM_IsCompressed),
        _ => ::std::option::Option::None,
    }
}
