use std::time::Instant;

use parsing::parser::Parser;
mod parsing;

fn main() {
    //let mut parser = Parser::new("/home/laiho/Documents/demos/cs2/s2-gotv.dem");
    let mut wanted_props = vec!["m_vecX".to_owned()];
    let mut wanted_ticks: Vec<i32> = (0..300000).collect();
    let wanted_event = Some("player_death".to_string());
    let demo_path = "/home/laiho/Documents/demos/cs2/003606754679372906816_1689787990.dem";

    let before = Instant::now();
    let mut parser = Parser::new(demo_path, wanted_props, wanted_ticks, None, true);
    parser.start();
    println!("{:2?}", before.elapsed());
    //println!("{:#?}", parser.game_events)
}
