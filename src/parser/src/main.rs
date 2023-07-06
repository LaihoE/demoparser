use ahash::AHashMap;
use memmap2::MmapOptions;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::parser_thread_settings::create_huffman_lookup_table;
use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

//#[global_allocator]
//static GLOBAL: MiMalloc = MiMalloc;

use std::env;
fn x() {
    // this method needs to be inside main() method
    env::set_var("RUST_BACKTRACE", "1");
}

fn main() {
    x();
    let wanted_props = vec!["m_iClip1".to_string()];
    // let bytes = fs::read(demo_path).unwrap();
    // let file = File::open(demo_path).unwrap();
    // let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let before = Instant::now();
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/test2/").unwrap();
    let mut c = 0;
    let huf = create_huffman_lookup_table();
    let arc_huf = Arc::new(huf);

    for path in dir {
        c += 1;

        let before = Instant::now();

        if c > 3000 {
            break;
        }

        let file = File::open(path.unwrap().path()).unwrap();
        // let file = File::open("/home/laiho/Documents/demos/cs2/driv/lpk.dem").unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        mmap.advise(memmap2::Advice::HugePage).unwrap();

        let arc_bytes = Arc::new(mmap);
        // let demo_path = "/home/laiho/Documents/demos/cs2/driv/mirage.dem";

        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            bytes: arc_bytes.clone(),
            wanted_player_props: wanted_props.clone(),
            wanted_player_props_og_names: wanted_props.clone(),
            wanted_event: Some("player_death".to_string()),
            // wanted_event: None,
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
        let _d = ds.parse_demo().unwrap();
        // println!("{:?}", d.df);
        println!("{:?}", before.elapsed());

        // println!("{:?}", ds.handles);
        // println!("{:#?}", ds.state.cls_by_id);
    }
    println!("TOTAL {:?}", before.elapsed());
}
