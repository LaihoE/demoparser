use ahash::AHashMap;
use parsing::parser_settings::Parser;
use std::time::Instant;

use crate::parsing::parser_settings::ParserInputs;
mod parsing;
use crate::parsing::sendtables::Decoder;

fn main() {
    let wanted_props = vec!["m_iMatchStats_PlayersAlive_T".to_owned()];
    let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";

    let before = Instant::now();

    for i in 0..5 {
        let settings = ParserInputs {
            path: demo_path.to_string(),
            wanted_props: wanted_props.clone(),
            wanted_event: None,
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: false,
            count_props: false,
            only_convars: false,
        };
        let mut parser = Parser::new(settings);

        parser.start().unwrap();

        println!(
            "{} {} {:.2}% into the demo file.",
            parser.ptr,
            parser.bytes.len(),
            parser.ptr as f32 / parser.bytes.len() as f32 * 100.0
        );

        /*
        let mut t: Vec<(u64, u64, Decoder)> = parser
            .history
            .iter()
            .map(|(k, (v, d))| (*k, *v, d.clone()))
            .collect();
        t.sort_by_key(|x| x.1);

        for (k, v, d) in t {
            println!("{} => {:?}", k, d);
        }
        break;
        */
        break;
    }

    println!("{:2?}", before.elapsed());
}
