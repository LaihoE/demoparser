use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Serializer;
use crate::maps::PAINTKITS;
use crate::maps::WEAPINDICIES;
use crate::second_pass::parser_settings::ChatMessageRecord;
use crate::second_pass::parser_settings::EconItem;
use crate::second_pass::parser_settings::PlayerEndMetaData;
use crate::second_pass::parser_settings::SecondPassParser;
use csgoproto::cstrike15_usermessages::CCSUsrMsg_EndOfMatchAllPlayersData;
use csgoproto::cstrike15_usermessages::CCSUsrMsg_SendPlayerItemDrops;
use csgoproto::networkbasetypes::CNETMsg_SetConVar;
use csgoproto::usermessages::CUserMessageSayText2;
use protobuf::Message;

#[derive(Debug, Clone)]
pub struct Class {
    pub class_id: i32,
    pub name: String,
    pub serializer: Serializer,
}

impl<'a> SecondPassParser<'a> {
    pub fn parse_item_drops(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let drops: CCSUsrMsg_SendPlayerItemDrops = match Message::parse_from_bytes(&bytes) {
            Ok(msg) => msg,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        for item in &drops.entity_updates {
            let item_name = match WEAPINDICIES.get(&item.defindex.unwrap_or(u32::MAX)) {
                Some(name) => Some(name.to_string()),
                None => None,
            };
            let skin_name = match PAINTKITS.get(&item.paintindex.unwrap_or(u32::MAX)) {
                Some(name) => Some(name.to_string()),
                None => None,
            };
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
                item_name: item_name,
                skin_name: skin_name,
            });
        }
        Ok(())
    }
    pub fn parse_chat_messages(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let chat_msg: CUserMessageSayText2 = match Message::parse_from_bytes(&bytes) {
            Ok(msg) => msg,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
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
        let convar: CNETMsg_SetConVar = match Message::parse_from_bytes(&bytes) {
            Ok(msg) => msg,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
        for cv in &convar.convars {
            for var in &cv.cvars {
                self.convars.insert(var.name().to_owned(), var.value().to_owned());
            }
        }
        Ok(())
    }

    pub fn parse_player_end_msg(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let end_data: CCSUsrMsg_EndOfMatchAllPlayersData = match Message::parse_from_bytes(&bytes) {
            Ok(msg) => msg,
            Err(_) => return Err(DemoParserError::MalformedMessage),
        };
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
            self.player_end_data.push(PlayerEndMetaData {
                name: player.name.clone(),
                steamid: player.xuid,
                team_number: player.teamnumber,
            });
            for item in &player.items {
                if item.itemid() != 0 {
                    let item_name = match WEAPINDICIES.get(&item.defindex.unwrap_or(u32::MAX)) {
                        Some(name) => Some(name.to_string()),
                        None => None,
                    };
                    let skin_name = match PAINTKITS.get(&item.paintindex.unwrap_or(u32::MAX)) {
                        Some(name) => Some(name.to_string()),
                        None => None,
                    };
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
                        item_name: item_name,
                        skin_name: skin_name,
                    });
                }
            }
        }
        Ok(())
    }
    pub fn parse_player_stats_update(&mut self, _bytes: &[u8]) -> Result<(), DemoParserError> {
        // let upd: CCSUsrMsg_PlayerStatsUpdate = Message::parse_from_bytes(bytes);
        Ok(())
    }
    pub fn parse_file_info(&mut self, _bytes: &[u8]) -> Result<(), DemoParserError> {
        // let _info: CDemoFileInfo = Message::parse_from_bytes(bytes);
        Ok(())
    }
}
