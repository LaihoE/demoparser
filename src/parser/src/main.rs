use ahash::AHashMap;
use ahash::AHashSet;
use dashmap::DashMap;
use memmap2::MmapOptions;
use parser::demo_searcher::DemoSearcher;
use parser::parser_settings::create_huffman_lookup_table;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::parser_settings::SpecialIDs;
use parser::read_bits::Bitreader;
use parser::sendtables::PropInfo;
use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

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
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/test/").unwrap();
    //let huf = vec![];

    let arc_huf = Arc::new(huf);
    let mut c = 0;
    for path in dir {
        if c > 1000 {
            break;
        }
        c += 1;
        println!("{:?}", path.as_ref().unwrap().path());
        let before = Instant::now();

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
            name_to_id: AHashMap::default(),
            bytes: arc_bytes.clone(),
            fullpacket_offsets: vec![],
            ptr: 0,
            tick: 0,
            huf: arc_huf.clone(),
            settings: settings,
            handles: vec![],
            parse_entities: true,
            serializers: AHashMap::default(),
            parse_projectiles: false,
            wanted_player_props: wanted_props.clone(),
            wanted_ticks: AHashSet::default(),
            wanted_prop_paths: AHashSet::default(),
            path_to_prop_name: AHashMap::default(),
            prop_name_to_path: AHashMap::default(),
            wanted_event: None,
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            cls_by_id: AHashMap::default(),
            controller_ids: SpecialIDs {
                teamnum: None,
                player_name: None,
                steamid: None,
                player_pawn: None,
                player_team_pointer: None,
                weapon_owner_pointer: None,
                cell_x_offset_player: None,
                cell_x_player: None,
                cell_y_offset_player: None,
                cell_y_player: None,
                cell_z_offset_player: None,
                cell_z_player: None,
                team_team_num: None,
                active_weapon: None,
            },
            id_to_path: AHashMap::default(),
            id: 0,
            player_output_ids: vec![],
            wanted_prop_ids: vec![],
            prop_out_id: 0,
            prop_infos: vec![],
            header: AHashMap::default(),
        };
        let d = ds.front_demo_metadata().unwrap();
        println!("{:?}", d.keys());
    }
    println!("{:?}", before.elapsed());
    // println!("{:?}", ds.handles);
    //println!("{:#?}", ds.state.cls_by_id);
}
