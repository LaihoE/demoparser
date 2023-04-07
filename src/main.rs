use std::time::Instant;

use ahash::HashSet;
use parsing::parser::Parser;
mod parsing;

fn main() {
    //let mut parser = Parser::new("/home/laiho/Documents/demos/cs2/s2-gotv.dem");
    let mut wanted_props = vec!["m_vecX".to_owned()];
    let mut wanted_ticks: Vec<i32> = (0..300000).collect();
    let wanted_event = Some("smokegrenade_detonate".to_string());
    //let demo_path = "/home/laiho/Documents/demos/cs2/003606754679372906816_1689787990.dem";
    let demo_path = "/home/laiho/Documents/demos/cs2/fulls2demo.dem";

    let before = Instant::now();
    let mut parser = Parser::new(demo_path, wanted_props, wanted_ticks, wanted_event, true);
    parser.start();

    //let mut uniq = HashSet::default();
    /*
    for (idx, e) in parser.entities {
        uniq.insert(parser.cls_by_id[&(e.cls_id as i32)].name.clone());
        for p in e.props {
            if parser.cls_by_id[&(e.cls_id as i32)].name == "CSmokeGrenadeProjectile" {
                println!(
                    "{} {} {:?}",
                    parser.cls_by_id[&(e.cls_id as i32)].name,
                    p.0,
                    p.1
                )
            }
        }
    }
    for x in uniq {
        println!("{:?}", x);
    }

    */
    //println!("{:#?}", parser.game_events)
    println!("{:2?}", before.elapsed());
}
