use ahash::AHashMap;
use ahash::AHashSet;
use memmap2::MmapOptions;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::parser_thread_settings::create_huffman_lookup_table;
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
    let wanted_props = vec!["CCSPlayerPawn.m_iHealth".to_owned()];
    let demo_path = "/home/laiho/Documents/demos/cs2/test/66.dem";
    // let bytes = fs::read(demo_path).unwrap();
    // let file = File::open(demo_path).unwrap();
    // let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let before = Instant::now();
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/test2/").unwrap();
    let mut c = 0;

    for path in dir {
        c += 1;
        let huf = create_huffman_lookup_table();
        let arc_huf = Arc::new(huf);
        let before = Instant::now();

        if c > 1 {
            break;
        }

        let before1 = Instant::now();

        //let file = File::open(path.unwrap().path()).unwrap();
        let file = File::open("/home/laiho/Documents/demos/cs2/driv/nuke.dem").unwrap();

        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        mmap.advise(memmap2::Advice::HugePage).unwrap();
        // mmap.advise(memmap2::Advice::Sequential).unwrap();
        // mmap.advise(memmap2::Advice::).unwrap();

        let arc_bytes = Arc::new(mmap);
        let mc = arc_bytes.clone();
        let demo_path = "/home/laiho/Documents/demos/cs2/test2/66.dem";

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
        let d = ds.parse_demo().unwrap();
        // println!("{:?}", d.df);
        println!("{:?}", before.elapsed());

        // println!("{:?}", ds.handles);
        //println!("{:#?}", ds.state.cls_by_id);
    }
    println!("TOTAL {:?}", before.elapsed());
}
