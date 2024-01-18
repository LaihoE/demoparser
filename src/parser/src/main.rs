use ahash::AHashMap;
use memmap2::MmapOptions;
use parser::e2e_test;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::parser_thread_settings::create_huffman_lookup_table;
use std::fs;
use std::fs::File;
use std::time::Instant;
/*
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
*/
/*
extern crate stats_alloc;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;
*/

fn main() {
    e2e_test::create_tests();
}

/*
fn main() {
    let wanted_props = vec!["yaw".to_string()];
    let before = Instant::now();
    let dir = fs::read_dir("/home/laiho/Documents/demos/cs2/pov/").unwrap();
    let mut c = 0;
    let huf = create_huffman_lookup_table();
    let mut total = 0;

    for path in dir {
        c += 1;

        let before = Instant::now();

        if c > 100 {
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

        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_player_props: wanted_props.clone(),
            wanted_player_props_og_names: wanted_props.clone(),
            //wanted_events: vec!["player_death".to_string()],
            wanted_events: vec![],
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
        ds.is_multithreadable = false;

        let file = File::open("/home/laiho/Documents/programming/rust/csnew2/demoparser/test_demo.dem").unwrap();
        // let file = File::open(path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

        total += mmap.len();

        mmap.advise(memmap2::Advice::HugePage).unwrap();
        let d = ds.parse_demo(&mmap).unwrap();
        let mut s: Vec<&String> = d.game_events_counter.iter().collect();
        s.sort();

        for x in s {
            println!("{:?}", x);
        }
        println!("{:?}", d.df[&2]);
        println!("TOTAL {:?}", before.elapsed());
    }
    println!("TOTAL {:?}", before.elapsed().as_millis());
    let x = total as f32 / before.elapsed().as_millis() as f32;

    println!("{:?} GB/S", x * 1000.0 / 1_000_000_000.0);

    // println!("GB/S {:?}", x / 1_000_000_000.0);
}
*/
