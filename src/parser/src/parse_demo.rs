use crate::first_pass::frameparser::{FrameParser, StartEndOffset, StartEndType};
use crate::first_pass::parser::FirstPassOutput;
use crate::first_pass::parser_settings::check_multithreadability;
use crate::first_pass::parser_settings::{FirstPassParser, ParserInputs};
use crate::first_pass::prop_controller::{PropController, NAME_ID, STEAMID_ID, TICK_ID};
use crate::first_pass::read_bits::DemoParserError;
use crate::second_pass::collect_data::ProjectileRecord;
use crate::second_pass::game_events::{EventField, GameEvent};
use crate::second_pass::parser::SecondPassOutput;
use crate::second_pass::parser_settings::*;
use crate::second_pass::variants::VarVec;
use crate::second_pass::variants::{PropColumn, Variant};
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::CsvcMsgVoiceData;
use itertools::Itertools;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

pub const HEADER_ENDS_AT_BYTE: usize = 16;

#[derive(Debug)]
pub struct DemoOutput {
    pub df: AHashMap<u32, PropColumn>,
    pub game_events: Vec<GameEvent>,
    pub skins: Vec<EconItem>,
    pub item_drops: Vec<EconItem>,
    pub chat_messages: Vec<ChatMessageRecord>,
    pub convars: AHashMap<String, String>,
    pub header: Option<AHashMap<String, String>>,
    pub player_md: Vec<PlayerEndMetaData>,
    pub game_events_counter: AHashSet<String>,
    pub uniq_prop_names: Vec<String>,
    pub projectiles: Vec<ProjectileRecord>,
    pub voice_data: Vec<CsvcMsgVoiceData>,
    pub prop_controller: PropController,
    pub df_per_player: AHashMap<u64, AHashMap<u32, PropColumn>>,
}

pub struct Parser<'a> {
    input: ParserInputs<'a>,
    pub parsing_mode: ParsingMode,
}
#[derive(PartialEq)]
pub enum ParsingMode {
    ForceSingleThreaded,
    ForceMultiThreaded,
    Normal,
}

impl<'a> Parser<'a> {
    pub fn new(input: ParserInputs<'a>, parsing_mode: ParsingMode) -> Self {
        Parser {
            input: input,
            parsing_mode: parsing_mode,
        }
    }
    pub fn parse_demo(&mut self, demo_bytes: &[u8]) -> Result<DemoOutput, DemoParserError> {
        let mut first_pass_parser = FirstPassParser::new(&self.input);
        let first_pass_output = first_pass_parser.parse_demo(&demo_bytes, false)?;
        if self.parsing_mode == ParsingMode::Normal
            && check_multithreadability(&self.input.wanted_player_props)
            && !(self.parsing_mode == ParsingMode::ForceSingleThreaded)
            || self.parsing_mode == ParsingMode::ForceMultiThreaded
        {
            return self.second_pass_multi_threaded(demo_bytes, first_pass_output);
        } else {
            self.second_pass_single_threaded(demo_bytes, first_pass_output)
        }
    }

    fn second_pass_multi_threaded(&self, outer_bytes: &[u8], first_pass_output: FirstPassOutput) -> Result<DemoOutput, DemoParserError> {
        let second_pass_outputs: Vec<Result<SecondPassOutput, DemoParserError>> = first_pass_output
            .fullpacket_offsets
            .par_iter()
            .map(|offset| {
                let mut parser = SecondPassParser::new(first_pass_output.clone(), *offset, false, None)?;
                parser.start(outer_bytes)?;
                Ok(parser.create_output())
            })
            .collect();
        // check for errors
        let mut ok = vec![];
        for result in second_pass_outputs {
            match result {
                Err(e) => return Err(e),
                Ok(r) => ok.push(r),
            };
        }
        let mut outputs = self.combine_outputs(&mut ok, first_pass_output);
        if let Some(new_df) = self.rm_unwanted_ticks(&mut outputs.df) {
            outputs.df = new_df;
        }
        Parser::add_item_purchase_sell_column(&mut outputs.game_events);
        Parser::remove_item_sold_events(&mut outputs.game_events);
        Ok(outputs)
    }

    fn second_pass_single_threaded(&self, outer_bytes: &[u8], first_pass_output: FirstPassOutput) -> Result<DemoOutput, DemoParserError> {
        let mut parser = SecondPassParser::new(first_pass_output.clone(), 16, true, None)?;
        parser.start(outer_bytes)?;
        let second_pass_output = parser.create_output();
        let mut outputs = self.combine_outputs(&mut vec![second_pass_output], first_pass_output);
        if let Some(new_df) = self.rm_unwanted_ticks(&mut outputs.df) {
            outputs.df = new_df;
        }
        Parser::add_item_purchase_sell_column(&mut outputs.game_events);
        Parser::remove_item_sold_events(&mut outputs.game_events);
        Ok(outputs)
    }
    fn second_pass_threaded_with_channels(
        &self,
        outer_bytes: &[u8],
        first_pass_output: FirstPassOutput,
        reciever: Receiver<StartEndOffset>,
    ) -> Result<DemoOutput, DemoParserError> {
        thread::scope(|s| {
            let mut handles = vec![];
            let mut channel_threading_was_ok = true;
            loop {
                if let Ok(start_end_offset) = reciever.recv_timeout(Duration::from_secs(3)) {
                    match start_end_offset.msg_type {
                        StartEndType::EndOfMessages => break,
                        StartEndType::OK => {}
                        StartEndType::MultithreadingWasNotOk => {
                            channel_threading_was_ok = false;
                            break;
                        }
                    }
                    let my_first_out = first_pass_output.clone();
                    handles.push(s.spawn(move || {
                        let mut parser = SecondPassParser::new(my_first_out, start_end_offset.start, false, Some(start_end_offset))?;
                        parser.start(outer_bytes)?;
                        Ok(parser.create_output())
                    }));
                } else {
                    channel_threading_was_ok = false;
                    break;
                }
            }
            // Fallback if channels failed to find all fullpackets. Should be rare.
            if !channel_threading_was_ok {
                let mut first_pass_parser = FirstPassParser::new(&self.input);
                let first_pass_output = first_pass_parser.parse_demo(outer_bytes, false)?;
                return self.second_pass_multi_threaded_no_channels(outer_bytes, first_pass_output);
            }
            // check for errors
            let mut ok = vec![];
            for result in handles {
                match result.join() {
                    Err(_e) => return Err(DemoParserError::MalformedMessage),
                    Ok(r) => {
                        ok.push(r?);
                    }
                };
            }
            let mut outputs = self.combine_outputs(&mut ok, first_pass_output);
            if let Some(new_df) = self.rm_unwanted_ticks(&mut outputs.df) {
                outputs.df = new_df;
            }
            Parser::add_item_purchase_sell_column(&mut outputs.game_events);
            Parser::remove_item_sold_events(&mut outputs.game_events);
            return Ok(outputs);
        })
    }
    fn second_pass_multi_threaded_no_channels(&self, outer_bytes: &[u8], first_pass_output: FirstPassOutput) -> Result<DemoOutput, DemoParserError> {
        let second_pass_outputs: Vec<Result<SecondPassOutput, DemoParserError>> = first_pass_output
            .fullpacket_offsets
            .par_iter()
            .map(|offset| {
                let mut parser = SecondPassParser::new(first_pass_output.clone(), *offset, false, None)?;
                parser.start(outer_bytes)?;
                Ok(parser.create_output())
            })
            .collect();
        // check for errors
        let mut ok = vec![];
        for result in second_pass_outputs {
            match result {
                Err(e) => return Err(e),
                Ok(r) => ok.push(r),
            };
        }
        let mut outputs = self.combine_outputs(&mut ok, first_pass_output);
        if let Some(new_df) = self.rm_unwanted_ticks(&mut outputs.df) {
            outputs.df = new_df;
        }
        Parser::add_item_purchase_sell_column(&mut outputs.game_events);
        Parser::remove_item_sold_events(&mut outputs.game_events);
        Ok(outputs)
    }
    fn remove_item_sold_events(events: &mut Vec<GameEvent>) {
        events.retain(|x| x.name != "item_sold")
    }
    fn add_item_purchase_sell_column(events: &mut Vec<GameEvent>) {
        // Checks each item_purchase event for if the item was eventually sold

        let purchases = events.iter().filter(|x| x.name == "item_purchase").collect_vec();
        let sells = events.iter().filter(|x| x.name == "item_sold").collect_vec();

        let purchases = purchases.iter().filter_map(|event| SellBackHelper::from_event(event)).collect_vec();
        let sells = sells.iter().filter_map(|event| SellBackHelper::from_event(event)).collect_vec();

        let mut was_sold = vec![];
        for purchase in &purchases {
            let wanted_sells = sells
                .iter()
                .filter(|sell| sell.tick > purchase.tick && sell.steamid == purchase.steamid && sell.inventory_slot == purchase.inventory_slot);
            let wanted_buys = purchases
                .iter()
                .filter(|buy| buy.tick > purchase.tick && buy.steamid == purchase.steamid && buy.inventory_slot == purchase.inventory_slot);
            let min_tick_sells = wanted_sells.min_by_key(|x| x.tick);
            let min_tick_buys = wanted_buys.min_by_key(|x| x.tick);
            if let (Some(sell_tick), Some(buy_tick)) = (min_tick_sells, min_tick_buys) {
                if sell_tick.tick < buy_tick.tick {
                    was_sold.push(true);
                } else {
                    was_sold.push(false);
                }
            } else {
                was_sold.push(false);
            }
        }
        let mut idx = 0;
        for event in events {
            if event.name == "item_purchase" {
                event.fields.push(EventField {
                    name: "was_sold".to_string(),
                    data: Some(Variant::Bool(was_sold[idx])),
                });
                idx += 1;
            }
        }
    }
    fn rm_unwanted_ticks(&self, hm: &mut AHashMap<u32, PropColumn>) -> Option<AHashMap<u32, PropColumn>> {
        // Used for removing ticks when velocity is needed
        if self.input.wanted_ticks.is_empty() {
            return None;
        }
        let mut wanted_indicies = vec![];
        if let Some(ticks) = hm.get(&TICK_ID) {
            if let Some(VarVec::I32(t)) = &ticks.data {
                for (idx, val) in t.iter().enumerate() {
                    if let Some(tick) = val {
                        if self.input.wanted_ticks.contains(tick) {
                            wanted_indicies.push(idx);
                        }
                    }
                }
            }
        }
        let mut new_df = AHashMap::default();
        for (k, v) in hm {
            if let Some(new) = v.slice_to_new(&wanted_indicies) {
                new_df.insert(*k, new);
            }
        }
        Some(new_df)
    }

    fn combine_outputs(&self, second_pass_outputs: &mut Vec<SecondPassOutput>, first_pass_output: FirstPassOutput) -> DemoOutput {
        // Combines all inner DemoOutputs into one big output
        second_pass_outputs.sort_by_key(|x| x.ptr);

        let mut dfs = second_pass_outputs.iter().map(|x| x.df.clone()).collect();
        let all_dfs_combined = self.combine_dfs(&mut dfs, false);
        let all_game_events: AHashSet<String> = AHashSet::from_iter(second_pass_outputs.iter().flat_map(|x| x.game_events_counter.iter().cloned()));
        let mut all_prop_names: Vec<String> = Vec::from_iter(second_pass_outputs.iter().flat_map(|x| x.uniq_prop_names.iter().cloned()));
        all_prop_names.sort();
        all_prop_names.dedup();
        // Remove temp props
        let mut prop_controller = first_pass_output.prop_controller.clone();
        for prop in first_pass_output.added_temp_props {
            prop_controller.wanted_player_props.retain(|x| x != &prop);
            prop_controller.prop_infos.retain(|x| &x.prop_name != &prop);
        }
        let per_players: Vec<AHashMap<u64, AHashMap<u32, PropColumn>>> = second_pass_outputs.iter().map(|x| x.df_per_player.clone()).collect();
        let mut all_steamids = AHashSet::default();
        for entry in &per_players {
            for (k, _) in entry {
                all_steamids.insert(k);
            }
        }
        let mut pp = AHashMap::default();
        for steamid in all_steamids {
            let mut v = vec![];
            for output in &per_players {
                if let Some(df) = output.get(&steamid) {
                    v.push(df.clone());
                }
            }
            let combined = self.combine_dfs(&mut v, true);
            pp.insert(*steamid, combined);
        }

        DemoOutput {
            prop_controller: prop_controller,
            chat_messages: second_pass_outputs.iter().flat_map(|x| x.chat_messages.clone()).collect(),
            item_drops: second_pass_outputs.iter().flat_map(|x| x.item_drops.clone()).collect(),
            player_md: second_pass_outputs.iter().flat_map(|x| x.player_md.clone()).collect(),
            game_events: second_pass_outputs.iter().flat_map(|x| x.game_events.clone()).collect(),
            skins: second_pass_outputs.iter().flat_map(|x| x.skins.clone()).collect(),
            convars: second_pass_outputs.iter().flat_map(|x| x.convars.clone()).collect(),
            df: all_dfs_combined,
            header: Some(first_pass_output.header),
            game_events_counter: all_game_events,
            projectiles: second_pass_outputs.iter().flat_map(|x| x.projectiles.clone()).collect(),
            voice_data: second_pass_outputs.iter().flat_map(|x| x.voice_data.clone()).collect_vec(),
            df_per_player: pp,
            uniq_prop_names: all_prop_names,
        }
    }

    fn combine_dfs(&self, v: &mut Vec<AHashMap<u32, PropColumn>>, remove_name_and_steamid: bool) -> AHashMap<u32, PropColumn> {
        let mut big: AHashMap<u32, PropColumn> = AHashMap::default();
        if v.len() == 1 {
            let mut result = v.remove(0);
            if remove_name_and_steamid {
                result.remove(&STEAMID_ID);
                result.remove(&NAME_ID);
            }
            return result;
        }

        for part_df in v {
            for (k, v) in part_df {
                if remove_name_and_steamid {
                    if k == &STEAMID_ID || k == &NAME_ID {
                        continue;
                    }
                }

                if big.contains_key(k) {
                    if let Some(inner) = big.get_mut(k) {
                        inner.extend_from(v)
                    }
                } else {
                    big.insert(*k, v.clone());
                }
            }
        }
        big
    }
}

#[derive(Debug)]
pub struct SellBackHelper {
    pub tick: i32,
    pub steamid: u64,
    pub inventory_slot: u32,
}
impl SellBackHelper {
    pub fn from_event(event: &GameEvent) -> Option<Self> {
        if let Some(Variant::I32(tick)) = SellBackHelper::extract_field("tick", &event.fields) {
            if let Some(Variant::U64(steamid)) = SellBackHelper::extract_field("steamid", &event.fields) {
                if let Some(Variant::U32(slot)) = SellBackHelper::extract_field("inventory_slot", &event.fields) {
                    return Some(SellBackHelper {
                        tick: tick,
                        steamid: steamid,
                        inventory_slot: slot,
                    });
                }
            }
        }
        None
    }
    fn extract_field(name: &str, fields: &[EventField]) -> Option<Variant> {
        for field in fields {
            if field.name == name {
                return field.data.clone();
            }
        }
        None
    }
}
