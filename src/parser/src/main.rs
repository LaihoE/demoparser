use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::read_bits::Bitreader;
use std::fs;
use std::time::Instant;

fn main() {
    let wanted_props = vec![
        "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon".to_owned(),
        "m_iClip1".to_owned(),
    ];
    let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";
    let bytes = fs::read(demo_path).unwrap();

    let settings = ParserInputs {
        bytes: &bytes,
        wanted_player_props: wanted_props.clone(),
        wanted_player_props_og_names: wanted_props.clone(),
        wanted_event: Some("bomb_planted".to_string()),
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
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
    };
    let mut parser = Parser::new(settings).unwrap();
    parser.start().unwrap();

    for event in &parser.game_events {
        for f in &event.fields {
            println!("{} {:?}", f.name, f.data);
        }
    }
}
