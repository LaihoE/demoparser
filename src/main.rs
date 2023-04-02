use std::time::Instant;

use parsing::parser::Parser;
mod parsing;

fn main() {
    let wanted_props = vec!["m_vecX".to_owned()];
    let wanted_ticks: Vec<i32> = (0..300000).collect();
    let wanted_event = Some("player_death".to_string());
    let demo_path = "path_to_my_demo";

    let mut parser = Parser::new(demo_path, wanted_props, wanted_ticks, None, true);
    parser.start();
}
