use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::Serializer;
use crate::second_pass::parser_settings::EconItem;
use crate::second_pass::parser_settings::PlayerEndMetaData;
use crate::second_pass::parser_settings::SecondPassParser;
use csgoproto::maps::PAINTKITS;
use csgoproto::maps::WEAPINDICIES;
use csgoproto::CcsUsrMsgEndOfMatchAllPlayersData;
use csgoproto::CcsUsrMsgSendPlayerItemDrops;
use prost::Message;

#[derive(Debug, Clone)]
pub struct Class {
    pub class_id: i32,
    pub name: String,
    pub serializer: Serializer,
}

impl<'a> SecondPassParser<'a> {
    pub fn parse_item_drops(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let drops = match CcsUsrMsgSendPlayerItemDrops::decode(bytes) {
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
                item_name,
                skin_name,
            });
        }
        Ok(())
    }

    pub fn parse_player_end_msg(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        let end_data = match CcsUsrMsgEndOfMatchAllPlayersData::decode(bytes) {
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
                        item_name,
                        skin_name,
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
