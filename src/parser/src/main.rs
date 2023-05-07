use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use std::fs;
use std::time::Instant;

fn main() {
    let wanted_props = vec!["CCSPlayerPawn.m_szLastPlaceName".to_owned()];
    let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";

    let bytes = fs::read(demo_path).unwrap();

    let settings = ParserInputs {
        bytes: &bytes,
        wanted_props: wanted_props.clone(),
        wanted_prop_og_names: wanted_props.clone(),
        wanted_event: Some("bomb_planted".to_string()),
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

    println!("{:?}", parser.game_events);

    for event in &parser.game_events {
        for f in &event.fields {
            println!("{} {:?}", f.name, f.data);
        }
    }
}
