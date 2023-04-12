use parsing::parser_settings::Parser;
use std::time::Instant;

use crate::parsing::parser_settings::ParserInputs;
mod parsing;

fn main() {
    //let mut parser = Parser::new("/home/laiho/Documents/demos/cs2/s2-gotv.dem");
    let wanted_props = vec!["round".to_owned()];
    let wanted_ticks: Vec<i32> = (0..300000).collect();
    //let demo_path = "/home/laiho/Documents/demos/cs2/003606754679372906816_1689787990.dem";
    let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";

    let before = Instant::now();

    let settings = ParserInputs {
        path: demo_path.to_string(),
        wanted_props: wanted_props.clone(),
        wanted_event: None,
        parse_ents: true,
        wanted_ticks: wanted_ticks,
        parse_projectiles: false,
    };
    let mut parser = Parser::new(settings);
    parser.start();
    println!("{:2?}", before.elapsed());
}
