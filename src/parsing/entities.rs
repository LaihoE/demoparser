use crate::parsing::class::Class;
use crate::parsing::entities_utils::*;
use crate::parsing::parser::Parser;
use crate::parsing::read_bits::Bitreader;
use crate::parsing::sendtables::Decoder;
use crate::parsing::u128arr::U128Arr;
use crate::parsing::variants::PropData;
use ahash::HashMap;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use smallvec::{smallvec, SmallVec};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
//use std::collections::HashMap as shitmap;
//use fxhash::FxHashMap as shitmap;

const NSERIALBITS: u32 = 17;

pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: HashMap<String, PropData>,
}
impl Entity {
    pub fn get_prop(&self, name: &str) -> Option<&PropData> {
        self.props.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerMetaData {
    pub player_entity_id: i32,
    pub steamid: u64,
    pub controller_entid: i32,
    pub name: String,
    pub team_num: i32,
}

// struct PlayerController {}

impl Parser {
    pub fn get_prop_for_ent(&self, prop: &str, entity_id: &i32) -> Option<&PropData> {
        if let Some(ent) = self.entities.get(&entity_id) {
            if let Some(prop) = ent.props.get(prop) {
                return Some(prop);
            }
        }
        None
    }

    pub fn parse_packet_ents(&mut self, packet_ents: CSVCMsg_PacketEntities) {
        // MIGHT BE ABLE TO SEE WHAT ENTS ARE UPDATED IN THIS PACKET

        let mut b = Bitreader::new(packet_ents.serialized_entities());

        let n_updates = packet_ents.updated_entries();
        let entity_data = packet_ents.entity_data.clone().unwrap();
        let mut bitreader = Bitreader::new(&entity_data);
        let mut entity_id: i32 = -1;

        for _ in 0..n_updates {
            entity_id += 1 + (bitreader.read_u_bit_var().unwrap() as i32);

            // If the enitity should just be deleted
            if bitreader.read_boolie().unwrap() {
                // Entity should be "deleted"
                self.projectiles.remove(&entity_id);
                self.entities.remove(&entity_id);
                bitreader.read_boolie();
                continue;
            }
            let is_new_entity = bitreader.read_boolie().unwrap();

            // Should we create the entity, or refer to an old one
            if is_new_entity {
                let cls_id = bitreader.read_nbits(self.cls_bits).unwrap();
                // Both of these are not used. Don't think they are interesting for the parser
                let _serial = bitreader.read_nbits(NSERIALBITS).unwrap();
                let _unknown = bitreader.read_varint();

                let entity = Entity {
                    entity_id: entity_id,
                    cls_id: cls_id,
                    props: HashMap::default(),
                };
                self.entities.insert(entity_id, entity);

                //println!("{:?}", self.cls_by_name.keys());

                // CInfoMapRegion
                // CCSGO_TeamIntroTerroristPosition
                // CItemDogtags
                // CCSPlayerController
                // CCSGameRulesProxy

                // "CCSGO_TeamIntroCounterTerroristPosition",
                // "CCSGO_TeamPreviewCharacterPosition",
                // "CPointEntity",
                // "CMapVetoPickController",
                // "CCSGO_TeamSelectTerroristPosition",
                // CCSGO_TeamIntroCharacterPosition
                /*
                println!("{:#?}", self.cls_by_name.keys());

                let pp = &self.cls_by_name["CCSGameRulesProxy"];
                for x in &pp.serializer.fields {
                    println!("{:#?}", x);
                }
                panic!("s");
                */
                let pp = &self.cls_by_name["CCSTeam"];
                for x in &pp.serializer.fields {
                    //println!("{:#?}", x);
                }
                // println!("{:?}", self.cls_by_name.keys());
                //panic!("s");
                //println!("{:?}", self.baselines.get(&cls_id));
                /*

                */
                let baseline = match &self.baselines.get(&cls_id) {
                    Some(bl) => Some(bl.clone()),
                    None => None,
                };

                match &self.baselines.get(&cls_id) {
                    Some(bl) => {
                        //println!("BASELINE");
                        let b = &baseline.unwrap().clone();
                        //println!("BBBBB {:?}", b.len());
                        let mut br = Bitreader::new(&b);
                        //br.read_nbits(2);
                        self.decode_paths(&mut br, entity_id, true);
                    }
                    None => {}
                };

                if self.cls_by_id[&cls_id].name.contains("Projectile") {
                    self.projectiles.insert(entity_id);
                }
                //println!("**********************************************'");
                self.decode_paths(&mut bitreader, entity_id, false);
            } else {
                // Entity already exists, don't create it
                self.decode_paths(&mut bitreader, entity_id, false);
            }
        }
    }

    pub fn decode_paths(&mut self, bitreader: &mut Bitreader, entity_id: i32, f: bool) {
        let paths = self.parse_paths(bitreader);
        let entity = self.entities.get_mut(&entity_id).unwrap();
        let cls = &self.cls_by_id[&(entity.cls_id)];
        if cls.name == "CCSPlayerController" {
            // hacky solution for now
            let player_md = Parser::fill_player_data(&self.paths[..paths], bitreader, cls, entity);
            if player_md.player_entity_id != -1 {
                self.players.insert(player_md.player_entity_id, player_md);
            }
        } else {
            let mut cls = &mut self.cls_by_id.get_mut(&entity.cls_id).unwrap();

            println!("{:?}", &self.paths[..paths]);

            for path in &self.paths[..paths] {
                let (name, f, decoder) = cls.serializer.find_decoder(&path, 0);
                let result = bitreader.decode(&decoder);
                println!("{:?}", path);
                println!("{} {:?}", name, result);

                if cls.name == "CCSTeam" && name == "m_iTeamNum" {
                    if let PropData::U32(t) = result {
                        match t {
                            1 => self.teams.team1_entid = Some(entity_id),
                            2 => self.teams.team2_entid = Some(entity_id),
                            3 => self.teams.team3_entid = Some(entity_id),
                            _ => panic!("unknown team: {:?}", result),
                        }
                    }
                }

                if (name == "m_vecX" && f.var_name != "CBodyComponent")
                    || (name == "m_vecY" && f.var_name != "CBodyComponent")
                {
                } else {
                    entity.props.insert(name, result);
                }
            }
        }
    }
    pub fn fill_player_data(
        paths: &[FieldPath],
        bitreader: &mut Bitreader,
        cls: &Class,
        entity: &mut Entity,
    ) -> PlayerMetaData {
        let mut player = PlayerMetaData {
            player_entity_id: -1,
            controller_entid: entity.entity_id,
            team_num: -1,
            name: "".to_string(),
            steamid: 0,
        };
        for path in paths {
            let (var_name, field, decoder) = cls.serializer.find_decoder(&path, 0);
            let result = bitreader.decode(&decoder);
            entity.props.insert(var_name.clone(), result.clone());

            for (k, v) in &entity.props {
                //println!("Q {} {:?}", k, v);
            }

            if var_name.contains("score") || var_name.contains("Score") {
                // println!("SCP {}  {:?}", var_name, result);
            }

            match var_name.as_str() {
                "m_iTeamNum" => {}
                "m_iszPlayerName" => {
                    player.name = match result {
                        PropData::String(n) => n,
                        _ => "Broken name!".to_owned(),
                    };
                }
                "m_steamID" => {
                    player.steamid = match result {
                        PropData::U64(xuid) => xuid,
                        _ => 99999999,
                    };
                }
                "m_hPlayerPawn" => {
                    player.player_entity_id = match result {
                        PropData::U32(handle) => {
                            // create helper value
                            entity.props.insert(
                                "player_entid".to_string(),
                                PropData::I32((handle & 0x7FF) as i32),
                            );
                            (handle & 0x7FF) as i32
                        }
                        _ => -1,
                    };
                }
                _ => {}
            }
        }
        player
    }

    pub fn parse_paths(&mut self, bitreader: &mut Bitreader) -> usize {
        let mut fp = FieldPath {
            path: U128Arr::new(),
            last: 0,
        };
        // The trees are static and created one time in parser constructor.
        // see generate_huffman_tree()
        let mut cur_node = &self.huffman_tree;
        let mut next_node = &self.huffman_tree;
        // Read bits one at a time while traversing a tree (1 = go right, 0 = go left)
        // until you reach a leaf node. When we reach a leaf node we do the operation
        // that that leaf point to. if the operation was not "FieldPathEncodeFinish" then
        // start again from top of tree.
        // let mut paths = Vec::with_capacity(256);
        // println!("{}", paths.capacity());
        let mut idx = 0;
        //use bit_vec::BitVec;
        // let mut bv = BitVec::new();

        loop {
            match bitreader.read_boolie().unwrap() {
                true => {
                    //bv.push(true);
                    next_node = &mut cur_node.right.as_ref().unwrap();
                }
                false => {
                    //bv.push(false);
                    next_node = &mut cur_node.left.as_ref().unwrap();
                }
            }
            if next_node.is_leaf() {
                // println!("{:?}", bv.storage()[0] as i32);
                // Reset back to top of tree
                // println!("{}", next_node.value);
                // bv = BitVec::new();
                cur_node = &self.huffman_tree;
                if do_op(next_node.value, bitreader, &mut fp) {
                    break;
                } else {
                    self.paths[idx] = fp.clone();
                    idx += 1;
                    //paths.push(fp.clone());
                }
            } else {
                cur_node = next_node
            }
        }
        idx
    }
}
