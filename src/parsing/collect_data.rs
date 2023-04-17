use super::entities::PlayerMetaData;
use super::variants::PropData;
use crate::parsing::parser_settings::Parser;
use crate::parsing::variants::PropColumn;
use phf_macros::phf_set;
use soa_derive::StructOfArray;

#[derive(Debug, StructOfArray)]
pub struct ProjectileRecord {
    pub steamid: Option<u64>,
    pub name: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub tick: Option<i32>,
    pub grenade_type: Option<String>,
}

impl Parser {
    pub fn collect_entities(&mut self) {
        if !self.wanted_ticks.contains(&self.tick) && self.wanted_ticks.len() != 0 {
            return;
        }
        if self.parse_projectiles {
            self.collect_projectiles();
        }
        // iterate every player and every wanted prop name
        // if either one is missing then push None to output
        for (entity_id, player) in &self.players {
            for prop_name in &self.wanted_props {
                // returns none if missing
                let prop = self.find_prop(prop_name, entity_id, player);
                self.output
                    .entry(prop_name.to_string())
                    .or_insert_with(|| PropColumn::new())
                    .push(prop);
            }
        }
    }
    pub fn find_prop(
        &self,
        prop_name: &str,
        entity_id: &i32,
        player: &PlayerMetaData,
    ) -> Option<PropData> {
        // Early exit these metadata props
        match prop_name {
            "tick" => return Some(PropData::I32(self.tick)),
            "steamid" => return Some(PropData::U64(player.steamid)),
            "name" => return Some(PropData::String(player.name.to_string())),
            _ => {}
        }
        if CONTROLLERPROPS.contains(prop_name) {
            return self.get_prop_for_ent(prop_name, &player.controller_entid);
        }
        if CUSTOMPROPS.contains(&prop_name) {
            return self.create_custom_prop(&prop_name, entity_id);
        }
        if TEAMPROPS.contains(prop_name) {
            return self.find_team_prop(entity_id, prop_name);
        }
        if RULESPROPS.contains(prop_name) {
            match self.rules_entity_id {
                Some(entity_id) => return self.get_prop_for_ent(prop_name, &entity_id),
                None => return None,
            }
        }
        return self.get_prop_for_ent(&prop_name, &entity_id);
    }

    pub fn collect_cell_coordinate(&self, axis: &str, entity_id: &i32) -> Option<PropData> {
        let offset = self.get_prop_for_ent(&("m_vec".to_owned() + axis), entity_id);
        let cell = self.get_prop_for_ent(&("m_cell".to_owned() + axis), entity_id);
        if let Some(coord) = coord_from_cell(cell, offset) {
            return Some(PropData::F32(coord));
        }
        None
    }
    pub fn find_thrower_steamid(&self, entity_id: &i32) -> Option<u64> {
        let owner_entid = match self.get_prop_for_ent("m_nOwnerId", entity_id) {
            Some(PropData::U32(prop)) => Some(prop & 0x7FF),
            _ => None,
        };
        let steamid = match owner_entid {
            Some(entid) => match self.players.get(&(entid as i32)) {
                Some(metadata) => Some(metadata.steamid as u64),
                None => None,
            },
            None => None,
        };
        steamid
    }
    pub fn find_thrower_name(&self, entity_id: &i32) -> Option<String> {
        let owner_entid = match self.get_prop_for_ent("m_nOwnerId", entity_id) {
            Some(PropData::U32(prop)) => Some(prop & 0x7FF),
            _ => None,
        };
        let name = match owner_entid {
            Some(entid) => match self.players.get(&(entid as i32)) {
                Some(metadata) => Some(metadata.name.clone()),
                None => None,
            },
            None => None,
        };
        name
    }
    fn find_grenade_type(&self, entity_id: &i32) -> Option<String> {
        if let Some(ent) = self.entities.get(&entity_id) {
            if let Some(cls) = self.cls_by_id[ent.cls_id as usize].as_ref() {
                // remove extra from name: CSmokeGrenadeProjectile --> SmokeGrenade
                // Todo maybe make names like this: smoke_grenade or just "smoke"
                let mut clean_name = cls.name[1..].split_at(cls.name.len() - 11).0;
                // Seems like the only exception
                if clean_name == "BaseCSGrenade" {
                    clean_name = "HeGrenade"
                }
                return Some(clean_name.to_owned());
            }
        }
        None
    }

    pub fn collect_projectiles(&mut self) {
        for projectile_entid in &self.projectiles {
            let grenade_type = self.find_grenade_type(projectile_entid);
            let steamid = self.find_thrower_steamid(projectile_entid);
            let name = self.find_thrower_name(projectile_entid);
            let x = self.collect_cell_coordinate("X", projectile_entid);
            let y = self.collect_cell_coordinate("Y", projectile_entid);
            let z = self.collect_cell_coordinate("Z", projectile_entid);

            let float_x = match x {
                Some(PropData::F32(p)) => Some(p),
                _ => None,
            };
            let float_y = match y {
                Some(PropData::F32(p)) => Some(p),
                _ => None,
            };
            let float_z = match z {
                Some(PropData::F32(p)) => Some(p),
                _ => None,
            };

            self.projectile_records.push(ProjectileRecord {
                steamid: steamid,
                name: name,
                x: float_x,
                y: float_y,
                z: float_z,
                tick: Some(self.tick),
                grenade_type: grenade_type,
            });
        }
    }

    pub fn find_team_prop(&self, player_entid: &i32, prop: &str) -> Option<PropData> {
        if let Some(PropData::U32(team_num)) = self.get_prop_for_ent("m_iTeamNum", player_entid) {
            let team_entid = match team_num {
                // 1 should be spectator
                1 => self.teams.team1_entid,
                2 => self.teams.team2_entid,
                3 => self.teams.team3_entid,
                // KNOWN PROBLEM THAT SOMETIMES TEAM IS PARSED INCORRECTLY :(
                _ => None,
            };
            // Get prop from team entity
            if let Some(entid) = team_entid {
                if let Some(p) = self.get_prop_for_ent(prop, &entid) {
                    return Some(p);
                }
            }
        }
        None
    }
    fn find_round(&self) -> Option<PropData> {
        // Rules entity seems to also have: m_totalRoundsPlayed. Maybe that one would be better
        let mut total_rounds = 0;
        if let Some(team) = self.teams.team2_entid {
            if let Some(PropData::I32(ct_rounds)) = self.get_prop_for_ent("m_iScore", &team) {
                total_rounds += ct_rounds;
            }
        }
        if let Some(team) = self.teams.team3_entid {
            if let Some(PropData::I32(t_rounds)) = self.get_prop_for_ent("m_iScore", &team) {
                total_rounds += t_rounds;
            }
        }
        Some(PropData::I32(total_rounds))
    }
    pub fn create_custom_prop(&self, prop_name: &str, entity_id: &i32) -> Option<PropData> {
        match prop_name {
            "round" => self.find_round(),
            "X" => self.collect_cell_coordinate("X", entity_id),
            "Y" => self.collect_cell_coordinate("Y", entity_id),
            "Z" => self.collect_cell_coordinate("Z", entity_id),
            _ => panic!("unknown custom prop: {}", prop_name),
        }
    }
}
fn coord_from_cell(cell: Option<PropData>, offset: Option<PropData>) -> Option<f32> {
    // DONT KNOW IF THESE ARE CORRECT. SEEMS TO GIVE CORRECT VALUES
    let cell_bits = 9;
    let max_coord = (1 << 14) as f32;
    // Both are cell and offset are needed for calculation
    if let Some(PropData::U32(cell)) = cell {
        if let Some(PropData::F32(offset)) = offset {
            let cell_coord = ((cell as f32 * (1 << cell_bits) as f32) - max_coord) as f32;
            return Some(cell_coord + offset);
        }
    }
    None
}

pub static TEAMPROPS: phf::Set<&'static str> = phf_set! {
    "m_szClanTeamname",
    "m_iTeamNum",
    "m_scoreSecondHalf",
    "m_iScore",
    "m_szTeamMatchStat",
    "m_szTeamname",
    "m_bSurrendered",
    "m_numMapVictories",
    "m_iClanID",
    "m_scoreFirstHalf",
    "m_scoreOvertime",
    // WILL PROBABLY BREAK PARSER
    "m_szTeamFlagImage",
};
pub static RULESPROPS: phf::Set<&'static str> = phf_set! {
    "m_iRoundWinStatus",
    "m_eRoundWinReason",
    "m_iMatchStats_PlayersAlive_CT",
    "m_iMatchStats_PlayersAlive_T",
    "m_iNumConsecutiveCTLoses",
    "m_iNumConsecutiveTLoses",
    "m_flRestartRoundTime",
    "m_totalRoundsPlayed",
    "m_nRoundsPlayedThisPhase",
    "m_bBombPlanted",
    "m_bFreezePeriod",
    "m_bWarmupPeriod",
};

// Props that dont really exist (we create them manually)
pub static CUSTOMPROPS: phf::Set<&'static str> = phf_set! {
    "round",
    "X",
    "Y",
    "Z"
};

pub static CONTROLLERPROPS: phf::Set<&'static str> = phf_set! {
    "m_nPersonaDataPublicCommendsTeacher",
    "m_nPersonaDataPublicCommendsLeader",
    "m_nPersonaDataPublicCommendsFriendly",
    "m_szCrosshairCodes",
    "m_iCompetitiveRankType",
    "m_iCompTeammateColor",
    "m_nPersonaDataPublicLevel",
    "m_unMusicID",
    "m_iConnected",
    "m_steamID",
    "m_nNextThinkTick",
    "m_iEnemiesFlashed",
    "m_iTotalCashSpent",
    "m_iPawnArmor",
    "m_rank",
    "m_iAccount",
    "m_iKillReward",
    "m_iUtilityDamage",
    "m_iszPlayerName",
    "m_iDamage",
    "m_iEquipmentValue",
    "m_bPawnIsAlive",
    "m_iPawnHealth",
    "m_nPawnCharacterDefIndex",
    "m_DamageList",
    "m_flGravityScale",
    "m_iStartAccount",
    "m_iPawnLifetimeStart",
    "m_hPlayerPawn",
    "m_iPawnLifetimeEnd",
    "m_flTimeScale",
    "m_iCashEarned",
    "m_nQuestProgressReason",
    "m_nTickBase",
    "m_perRoundStats",
    "m_nSendUpdate",
    "m_iMoneySaved",
    "m_bPawnHasHelmet",
    "m_iCashSpentThisRound",
    "m_iPendingTeamNum",
    "m_iObjective",
    "m_vecBaseVelocity",
    "m_iPing",
};
