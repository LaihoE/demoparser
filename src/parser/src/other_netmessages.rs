use super::{read_bits::DemoParserError, sendtables::Serializer};
use crate::parser::Parser;
use crate::parser_thread_settings::ChatMessageRecord;
use crate::parser_thread_settings::EconItem;
use crate::parser_thread_settings::ParserThread;
use crate::parser_thread_settings::PlayerEndData;
use csgoproto::cstrike15_usermessages::CCSUsrMsg_EndOfMatchAllPlayersData;
use csgoproto::cstrike15_usermessages::CCSUsrMsg_SendPlayerItemDrops;
use csgoproto::demo::CDemoFileInfo;
use csgoproto::networkbasetypes::CNETMsg_SetConVar;
use csgoproto::usermessages::CUserMessageSayText2;
use protobuf::Message;

// This file has functions for the simpler netmessages.
// Don't want to create a new file for each of these.

#[derive(Debug, Clone)]
pub struct Class {
    pub class_id: i32,
    pub name: String,
    pub serializer: Serializer,
}

impl<'a> ParserThread<'a> {
    pub fn parse_item_drops(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let drops: CCSUsrMsg_SendPlayerItemDrops = Message::parse_from_bytes(&bytes).unwrap();
        for item in &drops.entity_updates {
            self.item_drops.push(EconItem {
                account_id: item.accountid,
                item_id: item.itemid,
                def_index: item.defindex,
                paint_index: item.paintindex,
                rarity: item.rarity,
                quality: item.quality,
                paint_seed: item.paintseed,
                paint_wear: item.paintwear,
                quest_id: item.questid,
                dropreason: item.dropreason,
                custom_name: item.customname.clone(),
                inventory: item.inventory,
                ent_idx: item.entindex,
                steamid: None,
            });
        }
        Ok(())
    }
    pub fn parse_chat_messages(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let chat_msg: CUserMessageSayText2 = Message::parse_from_bytes(bytes).unwrap();
        self.chat_messages.push(ChatMessageRecord {
            entity_idx: chat_msg.entityindex,
            param1: chat_msg.param1,
            param2: chat_msg.param2,
            param3: chat_msg.param3,
            param4: chat_msg.param4,
        });
        Ok(())
    }
    pub fn parse_convars(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let convar: CNETMsg_SetConVar = Message::parse_from_bytes(bytes).unwrap();
        for cv in &convar.convars {
            for var in &cv.cvars {
                self.convars
                    .insert(var.name().to_owned(), var.value().to_owned());
            }
        }
        Ok(())
    }

    pub fn parse_player_end_msg(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let end_data: CCSUsrMsg_EndOfMatchAllPlayersData =
            Message::parse_from_bytes(&bytes).unwrap();
        /*
        Todo parse "accolade", seems to be the awards at the end like "most mvps in game"
        But seems to only have integers so need to figure out what they mean
        example:

        Accolade {
            eaccolade: Some(
                21,
            ),
            value: Some(
                5100.0,
            ),
            position: Some(
                1,
            ),
        }
        */
        for player in &end_data.allplayerdata {
            self.player_end_data.push(PlayerEndData {
                name: player.name.clone(),
                steamid: player.xuid,
                team_number: player.teamnumber,
            });
            for item in &player.items {
                if item.itemid() != 0 {
                    self.skins.push(EconItem {
                        account_id: item.accountid,
                        item_id: item.itemid,
                        def_index: item.defindex,
                        paint_index: item.paintindex,
                        rarity: item.rarity,
                        quality: item.quality,
                        paint_seed: item.paintseed,
                        paint_wear: item.paintwear,
                        quest_id: item.questid,
                        dropreason: item.dropreason,
                        custom_name: item.customname.clone(),
                        inventory: item.inventory,
                        ent_idx: item.entindex,
                        steamid: player.xuid,
                    });
                }
            }
        }
        Ok(())
    }
    pub fn parse_player_stats_update(&mut self, _bytes: &[u8]) -> Result<(), DemoParserError> {
        // Only in pov demos
        // let upd: CCSUsrMsg_PlayerStatsUpdate = Message::parse_from_bytes(bytes).unwrap();
        Ok(())
    }
    pub fn parse_file_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let _info: CDemoFileInfo = Message::parse_from_bytes(bytes).unwrap();
        Ok(())
    }
}
