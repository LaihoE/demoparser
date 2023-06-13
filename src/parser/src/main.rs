use ahash::AHashMap;
use dashmap::DashMap;
use memmap2::MmapOptions;
use parser::demo_searcher::DemoSearcher;
use parser::demo_searcher::State;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::read_bits::Bitreader;
use std::fs;
use std::fs::File;
use std::time::Instant;

fn main() {
    let wanted_props = vec![
        "CCSPlayerController.m_iPawnHealth".to_owned(),
        "m_iClip1".to_owned(),
    ];
    let demo_path = "/home/laiho/Documents/demos/cs2/test/66.dem";
    let bytes = fs::read(demo_path).unwrap();
    //let file = File::open(demo_path).unwrap();
    //let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    /*
    let settings = ParserInputs {
        bytes: &bytes,
        wanted_player_props: wanted_props.clone(),
        wanted_player_props_og_names: wanted_props.clone(),
        wanted_event: Some("bomb_planted".to_string()),
        wanted_other_props: vec![
            "CCSTeam.m_iScore".to_string(),
            "CCSTeam.m_szTeamname".to_string(),
            "CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed".to_string(),
        ],
        wanted_other_props_og_names: vec![
            "score".to_string(),
            "name".to_string(),
            "CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed".to_string(),
        ],
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
    };
    let mut parser = Parser::new(settings).unwrap();
    parser.start().unwrap();

    let x = &parser.output["CCSPlayerController.m_iPawnHealth"];
    */
    use std::sync::Arc;

    let state = State {
        cls_by_id: Arc::new(DashMap::default()),
        serializers: Arc::new(DashMap::default()),
    };

    let mut ds = DemoSearcher {
        bytes: bytes,
        fullpacket_offsets: vec![],
        ptr: 0,
        tick: 0,
        cls_by_id: Arc::new(AHashMap::default()),
        state: state,
    };
    ds.front_demo_metadata().unwrap();
    //println!("{:#?}", ds.state.cls_by_id);
}
