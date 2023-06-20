use ahash::AHashMap;
use ahash::AHashSet;
use dashmap::DashMap;
use memmap2::MmapOptions;
use mimalloc::MiMalloc;
use parser::parser_settings::Parser;
use parser::parser_thread_settings::create_huffman_lookup_table;
use parser::parser_thread_settings::ParserInputs;
use parser::parser_thread_settings::SpecialIDs;
use parser::read_bits::Bitreader;
use parser::sendtables::PropInfo;
use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

//#[global_allocator]
//static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(24)
        .build_global()
        .unwrap();

    let wanted_props = vec!["X".to_owned(), "Y".to_owned()];
    let demo_path = "/home/laiho/Documents/demos/cs2/test/66.dem";
    //let bytes = fs::read(demo_path).unwrap();
    //let file = File::open(demo_path).unwrap();
    //let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let huf = create_huffman_lookup_table();
    let before = Instant::now();
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/test2/").unwrap();
    //let huf = vec![];

    let arc_huf = Arc::new(huf);
    let mut c = 0;
    for path in dir {
        if c > 1000 {
            break;
        }
        c += 1;
        println!("{:?}", path.as_ref().unwrap().path());
        let before1 = Instant::now();

        let file = File::open(path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let arc_bytes = Arc::new(mmap);
        let mc = arc_bytes.clone();

        let demo_path = "/home/laiho/Documents/demos/cs2/test/66.dem";

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

        let mut ds = Parser::new(settings);

        let d = ds.front_demo_metadata().unwrap();
        println!("{:?}", d.game_events);
    }
    println!("{:?}", before.elapsed());
    // println!("{:?}", ds.handles);
    //println!("{:#?}", ds.state.cls_by_id);
}
