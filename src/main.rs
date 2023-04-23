use crate::parsing::parser_settings::ParserInputs;
use parsing::parser_settings::Parser;
use polars::{prelude::NamedFrom, series::Series};
use std::time::Instant;
mod parsing;
use arrow_array::{Array, Float32Array};

fn main() {
    let wanted_props = vec![
        "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecX".to_owned(),
        "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecY".to_owned(),
    ];
    let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";

    let settings = ParserInputs {
        path: demo_path.to_string(),
        wanted_props: wanted_props.clone(),
        wanted_event: Some("player_death".to_string()),
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
    };
    let mut parser = Parser::new(settings).unwrap();
    let before = Instant::now();
    parser.start().unwrap();
    println!("{:2?}", before.elapsed());
}
