use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::variants::Variant;
use std::fs;

#[derive(Debug)]
pub struct MinMaxTestEntry {
    pub min: i64,
    pub max: i64,
    pub prop_name: String,
    pub variant_type: String,
}

fn main() {
    let m = MinMaxTestEntry {
        max: 100,
        min: 0,
        prop_name: "CCSPlayerController.m_iPawnHealth".to_owned(),
        variant_type: "u32".to_owned(),
    };

    let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";
    let bytes = fs::read(demo_path).unwrap();
    let wanted_props = vec![
        "CCSPlayerController.m_iPawnHealth".to_owned(),
        "m_iClip1".to_owned(),
    ];
    let mut parser = get_default_tick_parser(wanted_props.clone(), &bytes);
    parser.start().unwrap();

    let x = &parser.output["CCSPlayerController.m_iPawnHealth"];

    for v in &x.data {
        match v {
            parser::variants::VarVec::U32(h) => {
                for p in h {
                    if let Some(val) = p {
                        assert_eq!(true, val <= &(m.max as u32));
                        assert_eq!(true, val >= &(m.min as u32));
                    }
                }
            }
            _ => panic!("Wrong varvec from health"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::get_default_tick_parser;
    use crate::MinMaxTestEntry;
    use parser::parser_settings::Parser;
    use parser::parser_settings::ParserInputs;
    use std::fs;

    pub fn gt_lt_test(input: MinMaxTestEntry, demo_path: &str) -> bool {
        let bytes = fs::read(demo_path).unwrap();
        let wanted_props = vec![input.prop_name.clone()];
        let mut parser = get_default_tick_parser(wanted_props.clone(), &bytes);
        parser.start().unwrap();

        println!("{:?}", parser.output.keys());
        let x = &parser.output[&input.prop_name];

        for v in &x.data {
            match v {
                parser::variants::VarVec::U32(h) => {
                    if input.variant_type != "u32" {
                        panic!("INCORRECT VARIANT TYPE: {:?}", input);
                    }
                    for p in h {
                        if let Some(val) = p {
                            assert_eq!(true, val <= &(input.max as u32));
                            assert_eq!(true, val >= &(input.min as u32));
                        }
                    }
                }
                parser::variants::VarVec::F32(h) => {
                    if input.variant_type != "f32" {
                        panic!("INCORRECT VARIANT TYPE: {:?}", input);
                    }
                    for p in h {
                        if let Some(val) = p {
                            assert_eq!(true, val <= &(input.max as f32));
                            assert_eq!(true, val >= &(input.min as f32));
                        }
                    }
                }
                _ => panic!("Wrong varvec"),
            }
        }
        true
    }

    #[test]
    fn health() {
        let m = MinMaxTestEntry {
            max: 100,
            min: 0,
            prop_name: "CCSPlayerController.m_iPawnHealth".to_owned(),
            variant_type: "u32".to_owned(),
        };
        let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";

        assert_eq!(gt_lt_test(m, demo_path), true)
    }
    #[test]
    fn x() {
        let m = MinMaxTestEntry {
            max: 100_000,
            min: -100_000,
            prop_name: "X".to_owned(),
            variant_type: "f32".to_owned(),
        };
        let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";
        assert_eq!(gt_lt_test(m, demo_path), true)
    }
    #[test]
    fn y() {
        let m = MinMaxTestEntry {
            max: 100_000,
            min: -100_000,
            prop_name: "Y".to_owned(),
            variant_type: "f32".to_owned(),
        };
        let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";
        assert_eq!(gt_lt_test(m, demo_path), true)
    }
    #[test]
    fn z() {
        let m = MinMaxTestEntry {
            max: 100_000,
            min: -100_000,
            prop_name: "Z".to_owned(),
            variant_type: "f32".to_owned(),
        };
        let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";
        assert_eq!(gt_lt_test(m, demo_path), true)
    }
}
fn get_default_tick_parser(wanted_props: Vec<String>, bytes: &Vec<u8>) -> Parser {
    let settings = ParserInputs {
        bytes: &bytes,
        wanted_player_props: wanted_props.clone(),
        wanted_player_props_og_names: wanted_props.clone(),
        wanted_event: Some("bomb_planted".to_string()),
        wanted_other_props: vec![],
        wanted_other_props_og_names: vec![],
        parse_ents: true,
        wanted_ticks: vec![],
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
    };
    let mut parser = Parser::new(settings).unwrap();
    parser
}
