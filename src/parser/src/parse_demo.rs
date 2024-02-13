use crate::first_pass::parser::FirstPassOutput;
use crate::first_pass::parser_settings::check_multithreadability;
use crate::first_pass::parser_settings::{FirstPassParser, ParserInputs};
use crate::first_pass::prop_controller::{PropController, TICK_ID};
use crate::first_pass::read_bits::DemoParserError;
use crate::second_pass::collect_data::ProjectileRecord;
use crate::second_pass::game_events::GameEvent;
use crate::second_pass::parser::SecondPassOutput;
use crate::second_pass::parser_settings::*;
use crate::second_pass::variants::PropColumn;
use crate::second_pass::variants::VarVec;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::netmessages::CSVCMsg_VoiceData;
use itertools::Itertools;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;

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
    pub projectiles: Vec<ProjectileRecord>,
    pub voice_data: Vec<CSVCMsg_VoiceData>,
    pub prop_controller: PropController,
}

pub struct Parser<'a> {
    input: ParserInputs<'a>,
    pub force_singlethread: bool,
}

impl<'a> Parser<'a> {
    pub fn new(input: ParserInputs<'a>, force_singlethread: bool) -> Self {
        Parser {
            input: input,
            force_singlethread: force_singlethread,
        }
    }
    pub fn parse_demo(&mut self, demo_bytes: &[u8]) -> Result<DemoOutput, DemoParserError> {
        let mut first_pass_parser = FirstPassParser::new(&self.input);
        let first_pass_output = first_pass_parser.parse_demo(&demo_bytes)?;

        if check_multithreadability(&self.input.wanted_player_props) && !self.force_singlethread {
            self.second_pass_multi_threaded(demo_bytes, first_pass_output)
        } else {
            self.second_pass_single_threaded(demo_bytes, first_pass_output)
        }
    }

    fn second_pass_single_threaded(
        &self,
        outer_bytes: &[u8],
        first_pass_output: FirstPassOutput,
    ) -> Result<DemoOutput, DemoParserError> {
        let mut parser = SecondPassParser::new(first_pass_output.clone(), 16, true)?;
        parser.start(outer_bytes)?;
        let second_pass_output = parser.create_output();
        let mut outputs = self.combine_outputs(&mut vec![second_pass_output], first_pass_output);
        if let Some(new_df) = self.rm_unwanted_ticks(&mut outputs.df) {
            outputs.df = new_df;
        }
        Ok(outputs)
    }

    fn second_pass_multi_threaded(
        &self,
        outer_bytes: &[u8],
        first_pass_output: FirstPassOutput,
    ) -> Result<DemoOutput, DemoParserError> {
        let second_pass_outputs: Vec<Result<SecondPassOutput, DemoParserError>> = first_pass_output
            .fullpacket_offsets
            .par_iter()
            .map(|offset| {
                let mut parser = SecondPassParser::new(first_pass_output.clone(), *offset, false)?;
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
        Ok(outputs)
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
        let all_dfs_combined = self.combine_dfs(&mut dfs);
        let all_game_events: AHashSet<String> =
            AHashSet::from_iter(second_pass_outputs.iter().flat_map(|x| x.game_events_counter.iter().cloned()));
        // Remove temp props
        let mut prop_controller = first_pass_output.prop_controller.clone();
        for prop in first_pass_output.added_temp_props {
            prop_controller.wanted_player_props.retain(|x| x != &prop);
            prop_controller.prop_infos.retain(|x| &x.prop_name != &prop);
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
        }
    }
    fn combine_dfs(&self, v: &mut Vec<AHashMap<u32, PropColumn>>) -> AHashMap<u32, PropColumn> {
        let mut big: AHashMap<u32, PropColumn> = AHashMap::default();
        if v.len() == 1 {
            return v.remove(0);
        }
        for part_df in v {
            for (k, v) in part_df {
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
