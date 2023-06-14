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
    //let bytes = fs::read(demo_path).unwrap();
    let file = File::open(demo_path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

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
    use parser::parser_settings::create_huffman_lookup_table;
    use std::sync::Arc;

    let state = State {
        cls_by_id: Arc::new(DashMap::default()),
        serializers: Arc::new(DashMap::default()),
    };
    let huf = create_huffman_lookup_table();
    let wanted_props = vec![
        "CCSPlayerController.m_iPawnHealth".to_owned(),
        "m_iClip1".to_owned(),
    ];
    let demo_path = "/home/laiho/Documents/demos/cs2/test/66.dem";

    let arc_bytes = Arc::new(mmap);
    let arc_huf = Arc::new(huf);

    let settings = ParserInputs {
        bytes: arc_bytes.clone(),
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
        huffman_lookup_table: arc_huf.clone(),
    };

    let mut ds = DemoSearcher {
        bytes: arc_bytes.clone(),
        fullpacket_offsets: vec![],
        ptr: 0,
        tick: 0,
        state: state,
        huf: arc_huf.clone(),
        settings: settings,
        handles: vec![],
    };
    ds.front_demo_metadata().unwrap();
    for handle in ds.handles {
        handle.join().unwrap();
    }
    // println!("{:?}", ds.handles);
    //println!("{:#?}", ds.state.cls_by_id);
}
