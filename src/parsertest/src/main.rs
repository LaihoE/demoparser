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
                            panic!(
                                "INCORRECT VARIANT TYPE: {:?} F32 != {:?}",
                                entry.prop_name, entry.variant_type
                            );
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
                            panic!(
                                "INCORRECT VARIANT TYPE: {:?} I32 != {:?}",
                                entry.prop_name, entry.variant_type
                            );
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
                            panic!(
                                "INCORRECT VARIANT TYPE: {:?} U32 != {:?}",
                                entry.prop_name, entry.variant_type
                            );
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
                            panic!(
                                "INCORRECT VARIANT TYPE: {:?} U64 != {:?}",
                                entry.prop_name, entry.variant_type
                            );
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
                max: 1000000,
                prop_name: "CCSPlayerController.m_flSimulationTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
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
                max: 1000000,
                prop_name: "CCSPlayerController.m_iConnected".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 77000000000000000,
                prop_name: "CCSPlayerController.m_steamID".to_string(),
                variant_type: "U64".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_iDesiredFOV".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iStartAccount".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iTotalCashSpent".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iCashSpentThisRound".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 65535,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.m_unMusicID".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.m_rank".to_string(),
                variant_type: "U64".to_string(),
                },
                MinMaxTestEntry {
                min: -1,
                max: 10000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicLevel".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: -1,
                max: 10000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsLeader".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: -1,
                max: 10000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsTeacher".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: -1,
                max: 10000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsFriendly".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKills".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDeaths".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iAssists".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iLiveTime".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iHeadShotKills".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDamage".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iObjective".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iUtilityDamage".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEnemiesFlashed".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEquipmentValue".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iMoneySaved".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKillReward".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iCashEarned".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKills".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDeaths".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iAssists".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iLiveTime".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iHeadShotKills".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 10,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy5Ks".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy4Ks".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy3Ks".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iNumRoundKills".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iNumRoundKillsHeadshots".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.m_iPing".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 4,
                prop_name: "CCSPlayerController.m_iPendingTeamNum".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: -10000,
                max: 100000,
                prop_name: "CCSPlayerController.m_flForceTeamTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: -1,
                max: 100,
                prop_name: "CCSPlayerController.m_iCompTeammateColor".to_string(),
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
                max: 1,
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
                max: 1000,
                prop_name: "CCSPlayerController.m_unPlayerTvControlFlags".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
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
                min: -1000,
                max: 1000000,
                prop_name: "CCSPlayerController.m_iPawnLifetimeStart".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: -1000,
                max: 1000000,
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
                max: 1000,
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
                max: 1000,
                prop_name: "CCSPlayerController.m_iMVPs".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.CEconItemView.m_iItemDefinitionIndex".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.CEconItemView.m_iEntityLevel".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.CCSPlayerController_InventoryServices.CEconItemView.m_iInventoryPosition".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerController.m_nTickBase".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerController.m_nNextThinkTick".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_vecX".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_vecY".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_vecZ".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_vecBaseVelocity".to_string(),
                variant_type: "FloatVec32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_flFriction".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_flGravityScale".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.m_flTimeScale".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDamage".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iObjective".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iUtilityDamage".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemiesFlashed".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_flSimulationTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_flSlopeDropOffset".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_flSlopeDropHeight".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_vHeadConstraintOffset".to_string(),
                variant_type: "FloatVec32".to_string(),
                },
                /*
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_angEyeAngles".to_string(),
                variant_type: "VecXYZ".to_string(),
                },
                */
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerPawn.m_iHealth".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 10,
                prop_name: "CCSPlayerPawn.m_lifeState".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: -100000,
                max: 100000,
                prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellX".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: -100000,
                max: 100000,
                prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellY".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: -100000,
                max: 100000,
                prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellZ".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                    min: -100000,
                    max: 100000,
                    prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecX".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                    min: -100000,
                    max: 100000,
                    prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecY".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                    min: -100000,
                    max: 100000,
                    prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecZ".to_string(),
                variant_type: "F32".to_string(),
                },
                /*
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_angRotation".to_string(),
                variant_type: "VecXYZ".to_string(),
                },
                */
                MinMaxTestEntry {
                min: -100000,
                max: 100000,
                prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_flScale".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_flLastTeleportTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerPawn.m_iMaxHealth".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_flCreateTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 3,
                prop_name: "CCSPlayerPawn.m_iTeamNum".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name: "CCSPlayerPawn.m_flFieldOfView".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1,
                prop_name: "CCSPlayerPawn.CCSPlayer_WeaponServices.m_iAmmo".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 500,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxspeed".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 0,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxFallVelocity".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckAmount".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckSpeed".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpUntil".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpVel".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_flDeathTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_iWeaponPurchasesThisMatch".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_iWeaponPurchasesThisRound".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_flTimeOfLastInjury".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 4,
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
                max: 10000,
                prop_name: "CCSPlayerPawn.m_unTotalRoundDamageDealt".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerPawn.m_fMolotovDamageTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iMoveState".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 2,
                prop_name: "CCSPlayerPawn.m_nWhichBombZone".to_string(),
                variant_type: "I32".to_string(),
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
                max: 100,
                prop_name: "CCSPlayerPawn.m_iShotsFired".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
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
                max: 10000,
                prop_name: "CCSPlayerPawn.m_flGroundAccelLinearFracLastTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 10,
                prop_name: "CCSPlayerPawn.m_flFlashDuration".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
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
                prop_name: "CCSPlayerPawn.m_flLowerBodyYawTarget".to_string(),
                variant_type: "F32".to_string(),
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
                max: 32000,
                prop_name: "CCSPlayerPawn.m_unCurrentEquipmentValue".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerPawn.m_unRoundStartEquipmentValue".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 32000,
                prop_name: "CCSPlayerPawn.m_unFreezetimeEndEquipmentValue".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 3,
                prop_name: "CCSPlayerPawn.m_nSurvivalTeamNumber".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerPawn.m_aimPunchTickBase".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerPawn.m_aimPunchTickFraction".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100,
                prop_name: "CCSPlayerPawn.CCSPlayer_BulletServices.m_totalHitsOnServer".to_string(),
                variant_type: "I32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_iItemDefinitionIndex".to_string(),
                variant_type: "U32".to_string(),
                },
                /*
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.m_qDeathEyeAngles".to_string(),
                variant_type: "VecXYZ".to_string(),
                },
                */
                MinMaxTestEntry {
                min: -100000,
                max: 100000,
                prop_name: "CCSPlayerPawn.m_vecX".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                    min: -100000,
                    max: 100000,
                prop_name: "CCSPlayerPawn.m_vecY".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                    min: -100000,
                    max: 100000,
                prop_name: "CCSPlayerPawn.m_vecZ".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CCSPlayer_WeaponServices.m_flNextAttack".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000,
                prop_name: "CCSPlayerPawn.CCSPlayer_CameraServices.m_flFOVRate".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flCrouchTransitionStartTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_nDuckJumpTimeMsecs".to_string(),
                variant_type: "U32".to_string(),
                },

                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_nJumpTimeMsecs".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 100000,
                prop_name: "CCSPlayerPawn.CCSPlayer_MovementServices.m_flLastDuckTime".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerPawn.m_nNextThinkTick".to_string(),
                variant_type: "U32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 10000,
                prop_name: "CCSPlayerPawn.m_flFriction".to_string(),
                variant_type: "F32".to_string(),
                },
                MinMaxTestEntry {
                min: 0,
                max: 1000000,
                prop_name: "CCSPlayerPawn.m_flTimeScale".to_string(),
                variant_type: "F32".to_string(),
                },       
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
    let parser = Parser::new(settings).unwrap();
    parser
}
