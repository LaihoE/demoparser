use ahash::AHashMap;
use ahash::AHashSet;
use dashmap::DashMap;
use memmap2::MmapOptions;
use parser::demo_searcher::DemoSearcher;
use parser::demo_searcher::State;
use parser::parser_settings::create_huffman_lookup_table;
use parser::parser_settings::ControllerIDS;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::read_bits::Bitreader;
use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .unwrap();

    let wanted_props = vec![
        "CCSPlayerController.m_iPawnHealth".to_owned(),
        "m_iClip1".to_owned(),
    ];
    let demo_path = "/home/laiho/Documents/demos/cs2/test/66.dem";
    //let bytes = fs::read(demo_path).unwrap();
    //let file = File::open(demo_path).unwrap();
    //let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let huf = create_huffman_lookup_table();
    let wanted_props = vec![
        "CCSPlayerController.m_iPawnHealth".to_owned(),
        "m_iClip1".to_owned(),
    ];
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/test/").unwrap();
    let arc_huf = Arc::new(huf);

    for path in dir {
        println!("{:?}", path.as_ref().unwrap().path());
        let before = Instant::now();
        let state = State {
            cls_by_id: Arc::new(DashMap::default()),
            serializers: Arc::new(DashMap::default()),
        };
        let file = File::open(path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let arc_bytes = Arc::new(mmap);

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

        let mut ds = DemoSearcher {
            bytes: arc_bytes.clone(),
            fullpacket_offsets: vec![],
            ptr: 0,
            tick: 0,
            state: state,
            huf: arc_huf.clone(),
            settings: settings,
            handles: vec![],
            parse_entities: true,
            serializers: AHashMap::default(),
            parse_projectiles: false,
            wanted_player_props: vec!["CCSPlayerController.m_iPawnHealth".to_owned()],
            wanted_ticks: AHashSet::default(),
            wanted_prop_paths: AHashSet::default(),
            path_to_prop_name: AHashMap::default(),
            prop_name_to_path: AHashMap::default(),
            wanted_event: None,
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            cls_by_id: AHashMap::default(),
            controller_ids: ControllerIDS {
                teamnum: None,
                player_name: None,
                steamid: None,
                player_pawn: None,
            },
            id_to_path: AHashMap::default(),
            id: 0,
            player_output_ids: vec![],
            wanted_prop_ids: vec![],
            prop_out_id: 0,
        };
        ds.front_demo_metadata().unwrap();
        for handle in ds.handles {
            handle.join().unwrap();
        }
        println!("{:?}", before.elapsed());
        // break;
    }
    // println!("{:?}", ds.handles);
    //println!("{:#?}", ds.state.cls_by_id);
}
