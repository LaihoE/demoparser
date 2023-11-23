use ahash::AHashMap;
use memmap2::MmapOptions;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::parser_thread_settings::create_huffman_lookup_table;
use parser::variants::BytesVariant;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    let wanted_props = vec!["inventory".to_string()];
    let before = Instant::now();
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/mm/").unwrap();
    let mut c = 0;
    let huf = create_huffman_lookup_table();

    for path in dir {
        c += 1;

        let before = Instant::now();

        if c > 1 {
            break;
        }

        /*
        "CDecoyProjectile"
        "CSmokeGrenadeProjectile"
        "CMolotovProjectile"
        "CBaseCSGrenadeProjectile"
        "CFlashbang"
        "CFlashbangProjectile"
        */
        let file = File::open("/home/laiho/Documents/programming/python/map/1.dem").unwrap();
        // let file = File::open(path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        mmap.advise(memmap2::Advice::HugePage).unwrap();

        // let s: u32 = mmap.par_iter().map(|x| *x as u32).sum();

        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_player_props: wanted_props.clone(),
            wanted_player_props_og_names: wanted_props.clone(),
            wanted_events: vec!["player_blind".to_string()],
            //wanted_events: vec![],
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
            parse_projectiles: true,
            only_header: false,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &huf,
        };

        let mut ds = Parser::new(&settings);
        ds.is_multithreadable = true;
        let file = File::open("/home/laiho/Documents/programming/python/map/1.dem").unwrap();
        // let file = File::open(path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        mmap.advise(memmap2::Advice::HugePage).unwrap();
        let d = ds.parse_demo(&mmap).unwrap();
        println!("TOTAL {:?}", before.elapsed());
    }
    println!("TOTAL {:?}", before.elapsed());
}
