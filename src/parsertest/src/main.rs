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

    pub fn gt_lt_test(input: Vec<MinMaxTestEntry>, demo_path: &str) -> bool {
        let bytes = fs::read(demo_path).unwrap();
        let wanted_props: Vec<String> = input.iter().map(|x| x.prop_name.to_string()).collect();

        let mut parser = get_default_tick_parser(wanted_props.clone(), &bytes);
        parser.start().unwrap();

        for entry in &input {
            match parser.output.get(&entry.prop_name) {
                Some(propcol) => match &propcol.data {
                    Some(parser::variants::VarVec::F32(v)) => {
                        if entry.variant_type != "F32" {
                            panic!("INCORRECT VARIANT TYPE: {:?}", input);
                        }
                        for p in v {
                            if let Some(val) = p {
                                assert!(
                                    val <= &(entry.max as f32),
                                    "{} {} <= {}",
                                    entry.prop_name,
                                    val,
                                    entry.max
                                );
                                assert!(
                                    val >= &(entry.min as f32),
                                    "{} {} >= {}",
                                    entry.prop_name,
                                    val,
                                    entry.min
                                );
                            }
                        }
                    }
                    Some(parser::variants::VarVec::I32(v)) => {
                        if entry.variant_type != "I32" {
                            panic!("INCORRECT VARIANT TYPE: {:?}", input);
                        }
                        for p in v {
                            if let Some(val) = p {
                                assert!(
                                    val <= &(entry.max as i32),
                                    "{} {} <= {}",
                                    entry.prop_name,
                                    val,
                                    entry.max
                                );
                                assert!(
                                    val >= &(entry.min as i32),
                                    "{} {} >= {}",
                                    entry.prop_name,
                                    val,
                                    entry.min
                                );
                            }
                        }
                    }
                    Some(parser::variants::VarVec::U32(v)) => {
                        if entry.variant_type != "U32" {
                            panic!("INCORRECT VARIANT TYPE: {:?}", input);
                        }
                        for p in v {
                            if let Some(val) = p {
                                assert!(
                                    val <= &(entry.max as u32),
                                    "{} {} <= {}",
                                    entry.prop_name,
                                    val,
                                    entry.max
                                );
                                assert!(
                                    val >= &(entry.min as u32),
                                    "{} {} >= {}",
                                    entry.prop_name,
                                    val,
                                    entry.min
                                );
                            }
                        }
                    }
                    Some(parser::variants::VarVec::U64(v)) => {
                        if entry.variant_type != "U64" {
                            panic!("INCORRECT VARIANT TYPE: {:?}", input);
                        }
                        for p in v {
                            if let Some(val) = p {
                                assert!(
                                    val <= &(entry.max as u64),
                                    "{} {} <= {}",
                                    entry.prop_name,
                                    val,
                                    entry.max
                                );
                                assert!(
                                    val >= &(entry.min as u64),
                                    "{} {} >= {}",
                                    entry.prop_name,
                                    val,
                                    entry.min
                                );
                            }
                        }
                    }
                    Some(parser::variants::VarVec::String(v)) => {
                        panic!("String type: {:?}", entry);
                    }
                    Some(parser::variants::VarVec::Bool(v)) => {
                        panic!("Bool type: {:?}", entry);
                    }
                    None => {
                        println!("{:?} NONE propcol", entry.prop_name);
                    }
                },
                None => {
                    panic!("{:?} not found in output", entry.prop_name);
                }
            }
        }
        true
    }
    #[test]
    fn minmaxtest() {
        let demo_path = "/home/laiho/Documents/demos/cs2/s2.dem";

        let v = vec![
            MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_flSimulationTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_flCreateTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 3,
                prop_name: "CCSPlayerController.m_iTeamNum".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 77000000000000000,
                prop_name: "CCSPlayerController.m_steamID".to_string(),
                variant_type: "U64".to_string(),
            },
            // CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount
            MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.m_pInGameMoneyServices.m_iStartAccount".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerController.m_pInGameMoneyServices.m_iTotalCashSpent".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerController.m_pInGameMoneyServices.m_iCashSpentThisRound"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 65535,
                prop_name: "CCSPlayerController.m_pInventoryServices.m_unMusicID".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name: "CCSPlayerController.m_pInventoryServices.m_nPersonaDataPublicLevel"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name:
                    "CCSPlayerController.m_pInventoryServices.m_nPersonaDataPublicCommendsLeader"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name:
                    "CCSPlayerController.m_pInventoryServices.m_nPersonaDataPublicCommendsTeacher"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name:
                    "CCSPlayerController.m_pInventoryServices.m_nPersonaDataPublicCommendsFriendly"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 64,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iKills"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iDeaths"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 64,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iAssists"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iLiveTime"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 64,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iHeadShotKills"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iDamage"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 64,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iObjective"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iUtilityDamage"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iEnemiesFlashed"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iEquipmentValue"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iMoneySaved"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iKillReward"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name:
                    "CCSPlayerController.m_pActionTrackingServices.m_perRoundStats.m_iCashEarned"
                        .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 300,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iKills".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 300,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iDeaths".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 300,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iAssists".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iLiveTime".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iHeadShotKills".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iEnemy5Ks".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iEnemy4Ks".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iEnemy3Ks".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 64,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iNumRoundKills".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 64,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iNumRoundKillsHeadshots"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 999,
                prop_name: "CCSPlayerController.m_iPing".to_string(),
                variant_type: "U32".to_string(),
            },
            /*
            MinMaxTestEntry {
            min: 0,
            max: 1000,
            prop_name: "CCSPlayerController.m_szCrosshairCodes".to_string(),
            variant_type: "String".to_string(),
            },
            */
            MinMaxTestEntry {
                min: 0,
                max: 3,
                prop_name: "CCSPlayerController.m_iPendingTeamNum".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: -10000,
                max: 10000,
                prop_name: "CCSPlayerController.m_flForceTeamTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: -1,
                max: 10,
                prop_name: "CCSPlayerController.m_iCompTeammateColor".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_iCoachingTeam".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_nPlayerDominated".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_nPlayerDominatingMe".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 30,
                prop_name: "CCSPlayerController.m_iCompetitiveRanking".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name: "CCSPlayerController.m_iCompetitiveWins".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: -1,
                max: 1000,
                prop_name: "CCSPlayerController.m_iCompetitiveRankType".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: -1,
                max: 1000,
                prop_name: "CCSPlayerController.m_nEndMatchNextMapVote".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_unActiveQuestId".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_nQuestProgressReason".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: u32::MAX as i64,
                prop_name: "CCSPlayerController.m_unPlayerTvControlFlags".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: -100000,
                max: 1000000,
                prop_name: "CCSPlayerController.m_nDisconnectionTick".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.m_iPawnHealth".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.m_iPawnArmor".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 65535,
                prop_name: "CCSPlayerController.m_nPawnCharacterDefIndex".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: -1000,
                max: 100000,
                prop_name: "CCSPlayerController.m_iPawnLifetimeStart".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: -1000,
                max: 100000,
                prop_name: "CCSPlayerController.m_iPawnLifetimeEnd".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_iPawnGunGameLevel".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: -1,
                max: 1000,
                prop_name: "CCSPlayerController.m_iPawnBotDifficulty".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 300,
                prop_name: "CCSPlayerController.m_iScore".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_vecKills".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1,
                prop_name: "CCSPlayerController.m_iMVPs".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerController.m_nTickBase".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: -10000,
                max: 1000000,
                prop_name: "CCSPlayerController.m_nNextThinkTick".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: -10000,
                max: 10000,
                prop_name: "CCSPlayerController.m_vecX".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: -10000,
                max: 10000,
                prop_name: "CCSPlayerController.m_vecY".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: -10000,
                max: 10000,
                prop_name: "CCSPlayerController.m_vecZ".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_flFriction".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_flGravityScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_flTimeScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 100000,
                max: 100000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iDamage".to_string(),
                variant_type: "I32".to_string(),
            },
            /*
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iObjective".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iUtilityDamage".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_pActionTrackingServices.m_iEnemiesFlashed"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flSimulationTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hOwnerEntity".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_thirdPersonHeading".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flSlopeDropOffset".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flSlopeDropHeight".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vHeadConstraintOffset".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_angEyeAngles".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iHealth".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_lifeState".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_fFlags".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hGroundEntity".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_cellX".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_cellY".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_cellZ".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_vecX".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_vecY".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_vecZ".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_hParent".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_angRotation".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_flScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nNewSequenceParity".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nResetEventsParity".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_name".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_hierarchyAttachName".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_hModel".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_bClientClothCreationSuppressed".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_MeshGroupMask".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nIdealMotionType".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_bIsAnimationEnabled".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_bUseParentRenderBounds".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_materialGroup".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nHitboxSet".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_flWeight".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nBoolVariablesCount".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nOwnerOnlyBoolVariablesCount".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nRandomSeedOffset".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_bClientSideAnimation".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nAnimLoopMode".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredBoolVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredByteVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredUInt16Variables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredIntVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredUInt32Variables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredFloatVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredVectorVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_PredQuaternionVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_flLastTeleportTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_nOutsideWorld".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pEntity".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pEntity.m_nameStringableIndex".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iMaxHealth".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_MoveCollide".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_MoveType".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flCreateTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bClientSideRagdoll".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_ubInterpolationFrame".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iTeamNum".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hEffectEntity".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_fEffects".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flElasticity".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bSimulatedEveryTick".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bAnimatedEveryTick".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flNavIgnoreUntilTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nRenderMode".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nRenderFX".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_clrRender".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecRenderAttributes".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_LightGroup".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bRenderToCubemaps".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nInteractsAs".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nInteractsWith".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nInteractsExclude".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nEntityId".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nOwnerId".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nHierarchyId".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nCollisionGroup".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nCollisionFunctionMask".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecMins".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecMaxs".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_usSolidFlags".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nSolidType".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_triggerBloat".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nSurroundType".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_CollisionGroup".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nEnablePhysics".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecSpecifiedSurroundingMins".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecSpecifiedSurroundingMaxs".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vCapsuleCenter1".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vCapsuleCenter2".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flCapsuleRadius".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iGlowType".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iGlowTeam".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nGlowRange".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nGlowRangeMin".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_glowColorOverride".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bFlashing".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flGlowTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flGlowStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bEligibleForScreenHighlight".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flGlowBackfaceMult".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_fadeMinDist".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_fadeMaxDist".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flFadeScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flShadowStrength".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nObjectCulling".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nAddDecal".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vDecalPosition".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vDecalForwardAxis".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flDecalHealBloodRate".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flDecalHealHeightRate".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_ConfigEntitiesToPropagateMaterialDecalsTo".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pRagdollPose".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bClientRagdoll".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecForce".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nForceBone".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bShouldAnimateDuringGameplayPause".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bAnimGraphUpdateEnabled".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bInitiallyPopulateInterpHistory".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vLookTargetPosition".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_blinktoggle".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hMyWearables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flFieldOfView".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_hMyWeapons".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_hActiveWeapon".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_iAmmo".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_bIsLookingAtWeapon".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_bIsHoldingLookAtWeapon".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pItemServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pItemServices.m_bHasDefuser".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pItemServices.m_bHasHelmet".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pItemServices.m_bHasHeavyArmor".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pObserverServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pObserverServices.m_iObserverMode".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pObserverServices.m_hObserverTarget".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWaterServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pUseServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_vecPunchAngle".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_iFOV".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_iFOVStart".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_flFOVTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_vecPunchAngleVel".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_nPunchAngleJoltTick".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_hZoomOwner".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_hColorCorrectionCtrl".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_hViewEntity".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_hCtrl".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flMaxspeed".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flForceSubtickMoveWhen".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flMaxFallVelocity".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_vecLadderNormal".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_nLadderSurfacePropIndex".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flDuckAmount".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flDuckSpeed".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bDuckOverride".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bOldJumpPressed".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flJumpUntil".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flJumpVel".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_fStashGrenadeParameterWhen".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flDeathTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hController".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pActionTrackingServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pActionTrackingServices.m_bIsRescuing".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pActionTrackingServices.m_iWeaponPurchasesThisMatch"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pActionTrackingServices.m_iWeaponPurchasesThisRound"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pViewModelServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pViewModelServices.m_hViewModel".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hOriginalController".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bSpotted".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bSpottedByMask".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flTimeOfLastInjury".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nRelativeDirectionOfLastInjury".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iPlayerState".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_passiveItems".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsScoped".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsWalking".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bResumeZoom".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsDefusing".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsGrabbingHostage".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iBlockingUseActionInProgress".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_fImmuneToGunGameDamageTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bGunGameImmunity".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bMadeFinalGunGameProgressiveKill".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iGunGameProgressiveWeaponIndex".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iNumGunGameTRKillPoints".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iNumGunGameKillsWithCurrentWeapon".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_unTotalRoundDamageDealt".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_fMolotovDamageTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bHasMovedSinceSpawn".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bCanMoveDuringFreezePeriod".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_isCurrentGunGameLeader".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_isCurrentGunGameTeamLeader".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flGuardianTooFarDistFrac".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flDetectedByEnemySensorTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsSpawnRappelling".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecSpawnRappellingRopeOrigin".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nSurvivalTeam".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_hSurvivalAssassinationTarget".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flHealthShotBoostExpirationTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nHeavyAssaultSuitCooldownRemaining".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flLastExoJumpTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bHasNightVision".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bNightVisionOn".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bInBombZone".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bInBuyZone".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bInNoDefuseArea".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bKilledByTaser".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iMoveState".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nWhichBombZone".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bInHostageRescueZone".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flStamina".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iDirection".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iShotsFired".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_ArmorValue".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flVelocityModifier".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flGroundAccelLinearFracLastTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flFlashDuration".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flFlashMaxAlpha".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flProgressBarStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iProgressBarDuration".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bWaitForNoAttack".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsRespawningForDMBonus".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flLowerBodyYawTarget".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bStrafing".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bHideTargetID".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bHud_MiniScoreHidden".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bHud_RadarHidden".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nLastKillerIndex".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nLastConcurrentKilled".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nDeathCamMusic".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iAddonBits".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iPrimaryAddon".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iSecondaryAddon".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecPlayerPatchEconIndices".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_unCurrentEquipmentValue".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_unRoundStartEquipmentValue".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_unFreezetimeEndEquipmentValue".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_szLastPlaceName".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nSurvivalTeamNumber".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_aimPunchAngle".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_aimPunchAngleVel".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_aimPunchTickBase".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_aimPunchTickFraction".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bKilledByHeadshot".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pBulletServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pBulletServices.m_totalHitsOnServer".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pHostageServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pHostageServices.m_hCarriedHostage".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pHostageServices.m_hCarriedHostageProp".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pPingServices".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pPingServices.m_hPlayerPing".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iRetakesOffering".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iRetakesOfferingCard".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bRetakesHasDefuseKit".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bRetakesMVPLastRound".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iRetakesMVPBoostItem".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_RetakesMVPBoostExtraUtility".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bIsBuyMenuOpen".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nRagdollDamageBone".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vRagdollDamageForce".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vRagdollDamagePosition".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_szRagdollDamageWeaponName".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bRagdollDamageHeadshot".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iItemDefinitionIndex".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iEntityQuality".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iEntityLevel".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iItemIDHigh".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iItemIDLow".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iAccountID".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iInventoryPosition".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bInitialized".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_Attributes".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_szCustomName".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_qDeathEyeAngles".to_string(),
                variant_type: "VecXYZ".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bvDisabledHitGroups".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CRenderComponent".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecX".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecY".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecZ".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nMinCPULevel".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nMaxCPULevel".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nMinGPULevel".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nMaxGPULevel".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flWaterLevel".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetBoolVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetByteVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetUInt16Variables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetIntVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetUInt32Variables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetFloatVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetVectorVariables".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponent.m_OwnerOnlyPredNetQuaternionVariables"
                    .to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_hLastWeapon".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pWeaponServices.m_flNextAttack".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_hTonemapController".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.localSound".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.soundscapeIndex".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.localBits".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.soundscapeEntityListIndex".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.soundEventHash".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_PostProcessingVolumes".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pCameraServices.m_flFOVRate".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_nDuckTimeMsecs".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_nToggleButtonDownMask".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flFallVelocity".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bInCrouch".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_nCrouchState".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flCrouchTransitionStartTime"
                    .to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bDucked".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bDucking".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bInDuckJump".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_bAllowAutoMovement".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_nDuckJumpTimeMsecs".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_nJumpTimeMsecs".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_pMovementServices.m_flLastDuckTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.colorPrimaryLerpTo".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.colorSecondaryLerpTo".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.farz".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.skyboxFogFactor".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.skyboxFogFactorLerpTo".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.startLerpTo".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.endLerpTo".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.maxdensityLerpTo".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.lerptime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.duration".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.blendtobackground".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.scattering".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.locallightscale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nNextThinkTick".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vecBaseVelocity".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flFriction".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flGravityScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flTimeScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iHideHUD".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.scale".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.origin".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.bClip3DSkyBoxNearToWorldFar".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.flClip3DSkyBoxNearToWorldFarOffset".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.dirPrimary".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.colorPrimary".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.colorSecondary".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.start".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.end".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.maxdensity".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.exponent".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.HDRColorScale".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.enable".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.blend".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_bNoReflectionFog".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_nWorldGroupID".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_flNextSprayDecalTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_iTeamNum".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_aPlayers".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_aPawns".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_iScore".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_szTeamname".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_bSurrendered".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_szTeamMatchStat".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_numMapVictories".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_scoreFirstHalf".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_scoreSecondHalf".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_scoreOvertime".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_szClanTeamname".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_iClanID".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_szTeamFlagImage".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_szTeamLogoImage".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_nGGLeaderSlot_CT".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSTeam.m_nGGLeaderSlot_T".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_bHostageAlive".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_isHostageFollowingSomeone".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_iHostageEntityIDs".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_bombsiteCenterA".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_bombsiteCenterB".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_hostageRescueX".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_hostageRescueY".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_hostageRescueZ".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerResource.m_bEndMatchNextMapAllVoted".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bFreezePeriod".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bWarmupPeriod".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_fWarmupPeriodEnd".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_fWarmupPeriodStart".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bTerroristTimeOutActive".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bCTTimeOutActive".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flTerroristTimeOutRemaining".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flCTTimeOutRemaining".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nTerroristTimeOuts".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nCTTimeOuts".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bTechnicalTimeOut".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bMatchWaitingForResume".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iRoundTime".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_fMatchStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_fRoundStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flRestartRoundTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bGameRestart".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flGameStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_timeUntilNextPhaseStarts".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_gamePhase".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_totalRoundsPlayed".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nRoundsPlayedThisPhase".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nOvertimePlaying".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iHostagesRemaining".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bAnyHostageReached".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bMapHasBombTarget".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bMapHasRescueZone".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bMapHasBuyZone".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bIsQueuedMatchmaking".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nQueuedMatchmakingMode".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bIsValveDS".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bLogoMap".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bPlayAllStepSoundsOnServer".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iNumGunGameProgressiveWeaponsCT"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iNumGunGameProgressiveWeaponsT"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iSpectatorSlotCount".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_GGProgressiveWeaponOrderCT".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_GGProgressiveWeaponOrderT".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_GGProgressiveWeaponKillUpgradeOrderCT"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_GGProgressiveWeaponKillUpgradeOrderT"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_MatchDevice".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bHasMatchStarted".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flDMBonusStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flDMBonusTimeLength".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_unDMBonusWeaponLoadoutSlot".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bDMBonusActive".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nNextMapInMapgroup".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_szTournamentEventName".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_szTournamentEventStage".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_szMatchStatTxt".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_szTournamentPredictionsTxt".to_string(),
                variant_type: "String".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nTournamentPredictionsPct".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flCMMItemDropRevealStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flCMMItemDropRevealEndTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bIsDroppingItems".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bIsQuestEligible".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nGuardianModeWaveNumber".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nGuardianModeSpecialKillsRemaining"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nGuardianModeSpecialWeaponNeeded"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_numGlobalGiftsGiven".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_numGlobalGifters".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_numGlobalGiftsPeriodSeconds".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_arrFeaturedGiftersAccounts".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_arrFeaturedGiftersGifts".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_arrProhibitedItemIndices".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_arrTournamentActiveCasterAccounts"
                    .to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_numBestOfMaps".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nHalloweenMaskListSeed".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bBombDropped".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bBombPlanted".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iRoundWinStatus".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_eRoundWinReason".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bTCantBuy".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bCTCantBuy".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flGuardianBuyUntilTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iMatchStats_RoundResults".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iMatchStats_PlayersAlive_CT".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iMatchStats_PlayersAlive_T".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_TeamRespawnWaveTimes".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flNextRespawnWave".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nServerQuestID".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nEndMatchMapGroupVoteTypes".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nEndMatchMapGroupVoteOptions".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nEndMatchMapVoteWinner".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iNumConsecutiveCTLoses".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iNumConsecutiveTerroristLoses".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_vecPlayAreaMins".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_vecPlayAreaMaxs".to_string(),
                variant_type: "FloatVec32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iPlayerSpawnHexIndices".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_SpawnTileState".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flSpawnSelectionTimeStartCurrentStage"
                    .to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flSpawnSelectionTimeEndCurrentStage"
                    .to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flSpawnSelectionTimeEndLastStage"
                    .to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_spawnStage".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flTabletHexOriginX".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flTabletHexOriginY".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flTabletHexSize".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_roundData_playerXuids".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_roundData_playerPositions".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_roundData_playerTeams".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_SurvivalGameRuleDecisionTypes".to_string(),
                variant_type: "U64".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_SurvivalGameRuleDecisionValues"
                    .to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_flSurvivalStartTime".to_string(),
                variant_type: "F32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nMatchSeed".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bBlockersPresent".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bRoundInProgress".to_string(),
                variant_type: "Bool".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iFirstSecondHalfRound".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_iBombSite".to_string(),
                variant_type: "I32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_nMatchEndCount".to_string(),
                variant_type: "U32".to_string(),
            },
            MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSGameRulesProxy.m_pGameRules.m_bTeamIntroPeriod".to_string(),
                variant_type: "Bool".to_string(),
            },
            */
        ];
        gt_lt_test(v, demo_path);
    }
}
fn get_default_tick_parser(wanted_props: Vec<String>, bytes: &Vec<u8>) -> Parser {
    let settings = ParserInputs {
        bytes: &bytes,
        wanted_player_props: wanted_props.clone(),
        wanted_player_props_og_names: wanted_props.clone(),
        wanted_event: None,
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
