use crate::parsing::parser::Parser;
use crate::parsing::read_bits::Bitreader;
use crate::parsing::sendtables::Decoder;
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
    pub entity_id: i32,
    pub steamid: u64,
    pub name: String,
}

struct FieldPaths<'a> {
    v: Vec<i32>,
    paths: Vec<Path<'a>>,
}
struct Path<'a> {
    path: &'a [i32],
    last: i32,
}

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
        //println!("{:?}", packet_ents.alternate_baselines);
        //println!("{:?}", packet_ents.baseline());
        //println!("{:?}", packet_ents.update_baseline);

        //println!("Q INFO {:?}", packet_ents.command_queue_info);
        //println!("DELTA FROM {:?}", packet_ents.delta_from());
        //println!("LAST CMD {:?}", packet_ents.last_cmd_number());

        //println!("SER {:?}", packet_ents.serialized_entities);
        //println!("UPD {:?}", packet_ents.updated_entries());

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

                let pp = &self.cls_by_name["CCSGameRulesProxy"];
                for x in &pp.serializer.fields {
                    //println!("{:#?}", x);
                }
                //panic!("s");

                //println!("{:?}", self.baselines.get(&cls_id));
                /*
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
                */
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
            let player_md = Parser::fill_player_data(&self.paths[..paths], bitreader, cls);
            if player_md.entity_id != -1 {
                self.players.insert(player_md.entity_id, player_md);
            }
        } else {
            let mut cls = &mut self.cls_by_id.get_mut(&entity.cls_id).unwrap();

            for path in &self.paths[..paths] {
                // println!("{:?}", path.path);
                /*
                let key = path_to_key(&path, cls.class_id as u32);

                let decoder = match self.pattern_cache.get(&key) {
                    Some(d) => d.clone(),
                    None => {
                        let (_, f, decoder) = cls.serializer.find_decoder(&path, 0);

                        //self.pattern_cache.insert(key, decoder.clone());
                        decoder
                    }
                };
                */
                let (name, f, decoder) = cls.serializer.find_decoder(&path, 0);
                let result = bitreader.decode(&decoder);
                if cls.name == "CCSGO_TeamIntroTerroristPosition" && name != "m_flNextRespawnWave" {
                    // println!("{:?} {}", result, name);
                }
                if (name == "m_vecX" && f.var_name != "CBodyComponent")
                    || (name == "m_vecY" && f.var_name != "CBodyComponent")
                {
                    println!("{} {}", f.var_name, f.send_node);
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
    ) -> PlayerMetaData {
        let mut player = PlayerMetaData {
            entity_id: -1,
            name: "".to_string(),
            steamid: 0,
        };
        for path in paths {
            let (var_name, field, decoder) = cls.serializer.find_decoder(&path, 0);
            let result = bitreader.decode(&decoder);
            if var_name != "m_nTickBase" {
                //println!("{:?} {}", result, var_name);
            }

            match var_name.as_str() {
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
                    player.entity_id = match result {
                        PropData::U32(handle) => (handle & 0x7FF) as i32,
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
        //let mut paths = Vec::with_capacity(256);
        // println!("{}", paths.capacity());
        let mut idx = 0;
        loop {
            match bitreader.read_boolie().unwrap() {
                true => {
                    next_node = &mut cur_node.right.as_ref().unwrap();
                }
                false => {
                    next_node = &mut cur_node.left.as_ref().unwrap();
                }
            }
            if next_node.is_leaf() {
                // Reset back to top of tree
                // println!("{}", next_node.value);

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

#[inline(always)]
pub fn path_to_key(field_path: &FieldPath, cls_id: u32) -> u64 {
    let mut key: u64 = 0;
    for idx in 0..field_path.last + 1 {
        key |= field_path.path.get(idx) as u64;
        key <<= 14;
    }
    key | cls_id as u64
}

pub fn do_op(opcode: i32, bitreader: &mut Bitreader, field_path: &mut FieldPath) -> bool {
    match opcode {
        0 => PlusOne(bitreader, field_path),
        1 => PlusTwo(bitreader, field_path),
        2 => PlusThree(bitreader, field_path),
        3 => PlusFour(bitreader, field_path),
        4 => PlusN(bitreader, field_path),
        5 => PushOneLeftDeltaZeroRightZero(bitreader, field_path),
        6 => PushOneLeftDeltaZeroRightNonZero(bitreader, field_path),
        7 => PushOneLeftDeltaOneRightZero(bitreader, field_path),
        8 => PushOneLeftDeltaOneRightNonZero(bitreader, field_path),
        9 => PushOneLeftDeltaNRightZero(bitreader, field_path),
        10 => PushOneLeftDeltaNRightNonZero(bitreader, field_path),
        11 => PushOneLeftDeltaNRightNonZeroPack6Bits(bitreader, field_path),
        12 => PushOneLeftDeltaNRightNonZeroPack8Bits(bitreader, field_path),
        13 => PushTwoLeftDeltaZero(bitreader, field_path),
        14 => PushTwoPack5LeftDeltaZero(bitreader, field_path),
        15 => PushThreeLeftDeltaZero(bitreader, field_path),
        16 => PushThreePack5LeftDeltaZero(bitreader, field_path),
        17 => PushTwoLeftDeltaOne(bitreader, field_path),
        18 => PushTwoPack5LeftDeltaOne(bitreader, field_path),
        19 => PushThreeLeftDeltaOne(bitreader, field_path),
        20 => PushThreePack5LeftDeltaOne(bitreader, field_path),
        21 => PushTwoLeftDeltaN(bitreader, field_path),
        22 => PushTwoPack5LeftDeltaN(bitreader, field_path),
        23 => PushThreeLeftDeltaN(bitreader, field_path),
        24 => PushThreePack5LeftDeltaN(bitreader, field_path),
        25 => PushN(bitreader, field_path),
        26 => PushNAndNonTopological(bitreader, field_path),
        27 => PopOnePlusOne(bitreader, field_path),
        28 => PopOnePlusN(bitreader, field_path),
        29 => PopAllButOnePlusOne(bitreader, field_path),
        30 => PopAllButOnePlusN(bitreader, field_path),
        31 => PopAllButOnePlusNPack3Bits(bitreader, field_path),
        32 => PopAllButOnePlusNPack6Bits(bitreader, field_path),
        33 => PopNPlusOne(bitreader, field_path),
        34 => PopNPlusN(bitreader, field_path),
        35 => PopNAndNonTopographical(bitreader, field_path),
        36 => NonTopoComplex(bitreader, field_path),
        37 => NonTopoPenultimatePlusOne(bitreader, field_path),
        38 => NonTopoComplexPack4Bits(bitreader, field_path),
        39 => {
            // FieldPathEncodeFinish(bitreader, field_path);
            return true;
        }
        _ => {}
    }
    false
}

#[derive(Eq, Debug)]
pub struct HuffmanNode {
    pub weight: i32,
    pub value: i32,
    pub left: Option<Box<HuffmanNode>>,
    pub right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

use super::class::Class;
use super::u128arr::U128Arr;
use std::cmp::Ordering;

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.weight == other.weight {
            if self.value >= other.value {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            self.weight.cmp(&other.weight)
        }
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

#[derive(Clone, Debug)]
pub struct FieldPath {
    pub path: U128Arr,
    pub last: usize,
}
impl FieldPath {
    pub fn pop_special(&mut self, n: usize) {
        for i in 0..n {
            self.path.set(self.last, 0);
            //self.path[self.last] = 0;
            self.last -= 1;
        }
    }
}

fn PlusOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
}
fn PlusTwo(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 2);
}
fn PlusThree(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 3);
}
fn PlusFour(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 4);
}
fn PlusN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32 + 5);
}
fn PushOneLeftDeltaZeroRightZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.last += 1;
    field_path.path.set(field_path.last, 0);
}
fn PushOneLeftDeltaZeroRightNonZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
}
fn PushOneLeftDeltaOneRightZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
    field_path.last += 1;
    field_path.path.set(field_path.last, 0);
}
fn PushOneLeftDeltaOneRightNonZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
    field_path.last += 1;
    field_path
        .path
        .set(field_path.last, (bitreader.read_ubit_var_fp() as i32));
}
fn PushOneLeftDeltaNRightZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path.path.set(field_path.last, 0);
}
fn PushOneLeftDeltaNRightNonZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32 + 2);
    field_path.last += 1;
    field_path
        .path
        .set(field_path.last, bitreader.read_ubit_var_fp() as i32 + 1);
}
fn PushOneLeftDeltaNRightNonZeroPack6Bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_nbits(3).unwrap() + 2) as i32,
    );
    field_path.last += 1;
    field_path.path.set(
        field_path.last,
        (bitreader.read_nbits(3).unwrap() + 1) as i32,
    );
}
fn PushOneLeftDeltaNRightNonZeroPack8Bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_nbits(4).unwrap() + 2) as i32,
    );
    field_path.last += 1;
    field_path.path.set(
        field_path.last,
        (bitreader.read_nbits(4).unwrap() + 1) as i32,
    );
}
fn PushTwoLeftDeltaZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
}
fn PushTwoPack5LeftDeltaZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.last += 1;

    field_path
        .path
        .set(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
    field_path.last += 1;

    field_path
        .path
        .set(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
}
fn PushThreeLeftDeltaZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
}
fn PushThreePack5LeftDeltaZero(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.last += 1;
    field_path
        .path
        .set(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
    field_path.last += 1;
    field_path
        .path
        .set(field_path.last, bitreader.read_nbits(5).unwrap() as i32);

    field_path.last += 1;
    field_path
        .path
        .set(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
}
fn PushTwoLeftDeltaOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
}
fn PushTwoPack5LeftDeltaOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
}
fn PushThreeLeftDeltaOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
}
fn PushThreePack5LeftDeltaOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last, 1);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_nbits(5).unwrap() as i32);
}
fn PushTwoLeftDeltaN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_u_bit_var().unwrap() + 2) as i32,
    );
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_ubit_var_fp() as i32));
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_ubit_var_fp() as i32));
}
fn PushTwoPack5LeftDeltaN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_u_bit_var().unwrap() + 2) as i32,
    );
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_nbits(5).unwrap() as i32));
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_nbits(5).unwrap() as i32));
}
fn PushThreeLeftDeltaN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_u_bit_var().unwrap() + 2) as i32,
    );
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_ubit_var_fp() as i32));

    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_ubit_var_fp() as i32));
    field_path.last += 1;
    field_path
        .path
        .incr_idx(field_path.last, (bitreader.read_ubit_var_fp() as i32));
}
fn PushThreePack5LeftDeltaN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_u_bit_var().unwrap() + 2) as i32,
    );
    field_path.last += 1;
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_nbits(5).unwrap() as i32) as i32,
    );
    field_path.last += 1;
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_nbits(5).unwrap() as i32) as i32,
    );
    field_path.last += 1;
    field_path.path.incr_idx(
        field_path.last,
        (bitreader.read_nbits(5).unwrap() as i32) as i32,
    );
    field_path.last += 1;
}
fn PushN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    let n = bitreader.read_u_bit_var().unwrap() as i32;
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_u_bit_var().unwrap() as i32);
    for i in 0..n {
        field_path.last += 1;
        field_path
            .path
            .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32);
    }
}
fn PushNAndNonTopological(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie().unwrap() {
            field_path
                .path
                .incr_idx(i, bitreader.read_varint32().unwrap() + 1);
        }
    }
    let count = bitreader.read_u_bit_var().unwrap();
    for i in 0..count {
        field_path.last += 1;
        field_path
            .path
            .set(field_path.last, bitreader.read_ubit_var_fp() as i32);
    }
}
fn PopOnePlusOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(1);
    field_path.path.incr_idx(field_path.last, 1);
}
fn PopOnePlusN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(1);
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_ubit_var_fp() as i32 + 1);
}
fn PopAllButOnePlusOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(field_path.last);
    field_path.path.incr_idx(0, 1);
}
fn PopAllButOnePlusN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(field_path.last);
    field_path
        .path
        .incr_idx(0, bitreader.read_ubit_var_fp() as i32 + 1);
}
fn PopAllButOnePlusNPack3Bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(field_path.last);
    field_path
        .path
        .incr_idx(0, bitreader.read_nbits(3).unwrap() as i32 + 1);
}
fn PopAllButOnePlusNPack6Bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(field_path.last);
    field_path
        .path
        .incr_idx(0, bitreader.read_nbits(6).unwrap() as i32 + 1);
}
fn PopNPlusOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(bitreader.read_ubit_var_fp() as usize);
    field_path.path.incr_idx(field_path.last, 1);
}
fn PopNPlusN(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(bitreader.read_ubit_var_fp() as usize);
    field_path
        .path
        .incr_idx(field_path.last, bitreader.read_varint32().unwrap());
}
fn PopNAndNonTopographical(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.pop_special(bitreader.read_ubit_var_fp() as usize);
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie().unwrap() {
            field_path
                .path
                .incr_idx(i, bitreader.read_varint32().unwrap());
        }
    }
}
fn NonTopoComplex(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie().unwrap() {
            field_path
                .path
                .incr_idx(i, bitreader.read_varint32().unwrap());
        }
    }
}
fn NonTopoPenultimatePlusOne(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    field_path.path.incr_idx(field_path.last - 1, 1);
}
fn NonTopoComplexPack4Bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie().unwrap() {
            field_path
                .path
                .incr_idx(i, bitreader.read_nbits(4).unwrap() as i32 - 7);
        }
    }
}

const PAIRS: [(&str, i32); 40] = [
    ("PlusOne", 36271),
    ("PlusTwo", 10334),
    ("PlusThree", 1375),
    ("PlusFour", 646),
    ("PlusN", 4128),
    ("PushOneLeftDeltaZeroRightZero", 35),
    ("PushOneLeftDeltaZeroRightNonZero", 3),
    ("PushOneLeftDeltaOneRightZero", 521),
    ("PushOneLeftDeltaOneRightNonZero", 2942),
    ("PushOneLeftDeltaNRightZero", 560),
    ("PushOneLeftDeltaNRightNonZero", 471),
    ("PushOneLeftDeltaNRightNonZeroPack6Bits", 10530),
    ("PushOneLeftDeltaNRightNonZeroPack8Bits", 251),
    ("PushTwoLeftDeltaZero", 0),
    ("PushTwoPack5LeftDeltaZero", 0),
    ("PushThreeLeftDeltaZero", 0),
    ("PushThreePack5LeftDeltaZero", 0),
    ("PushTwoLeftDeltaOne", 0),
    ("PushTwoPack5LeftDeltaOne", 0),
    ("PushThreeLeftDeltaOne", 0),
    ("PushThreePack5LeftDeltaOne", 0),
    ("PushTwoLeftDeltaN", 0),
    ("PushTwoPack5LeftDeltaN", 0),
    ("PushThreeLeftDeltaN", 0),
    ("PushThreePack5LeftDeltaN", 0),
    ("PushN", 0),
    ("PushNAndNonTopological", 310),
    ("PopOnePlusOne", 2),
    ("PopOnePlusN", 0),
    ("PopAllButOnePlusOne", 1837),
    ("PopAllButOnePlusN", 149),
    ("PopAllButOnePlusNPack3Bits", 300),
    ("PopAllButOnePlusNPack6Bits", 634),
    ("PopNPlusOne", 0),
    ("PopNPlusN", 0),
    ("PopNAndNonTopographical", 1),
    ("NonTopoComplex", 76),
    ("NonTopoPenultimatePlusOne", 271),
    ("NonTopoComplexPack4Bits", 99),
    ("FieldPathEncodeFinish", 25474),
];

pub fn generate_huffman_tree() -> Option<HuffmanNode> {
    /*
    Should result in this tree (same as Dotabuffs tree):

    value, weight, len(prefix), prefix
    0	36271	2	0
    39	25474	3	10
    8	2942	6	11000
    2	1375	7	110010
    29	1837	7	110011
    4	4128	6	11010
    30	149	    10	110110000
    38	99	    11	1101100010
    35	1	    17	1101100011000000
    34	1	    17	1101100011000001
    27	2	    16	110110001100001
    25	1	    17	1101100011000100
    24	1	    17	1101100011000101
    33	1	    17	1101100011000110
    28	1	    17	1101100011000111
    13	1	    17	1101100011001000
    15	1	    18	11011000110010010
    14	1	    18	11011000110010011
    6	3	    16	110110001100101
    21	1	    18	11011000110011000
    20	1	    18	11011000110011001
    23	1	    18	11011000110011010
    22	1	    18	11011000110011011
    17	1	    18	11011000110011100
    16	1	    18	11011000110011101
    19	1	    18	11011000110011110
    18	1	    18	11011000110011111
    5	35	    13	110110001101
    36	76	    12	11011000111
    10	471	    9	11011001
    7	521	    9	11011010
    12	251	    10	110110110
    37	271	    10	110110111
    9	560	    9	11011100
    31	300	    10	110111010
    26	310	    10	110111011
    32	634	    9	11011110
    3	646	    9	11011111
    1	10334	5	1110
    11	10530	5	1111
    */

    let mut trees = vec![];
    for (idx, (_, weight)) in PAIRS.iter().enumerate() {
        let node = if *weight == 0 {
            HuffmanNode {
                weight: 1,
                value: idx as i32,
                left: None,
                right: None,
            }
        } else {
            HuffmanNode {
                weight: *weight,
                value: idx as i32,
                left: None,
                right: None,
            }
        };
        trees.push(node);
    }

    let mut heap = BinaryHeap::new();
    for tree in trees {
        heap.push(Reverse(tree));
    }

    for idx in 0..heap.len() - 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        heap.push(Reverse(HuffmanNode {
            weight: a.0.weight + b.0.weight,
            value: (idx + 40) as i32,
            left: Some(Box::new(a.0)),
            right: Some(Box::new(b.0)),
        }))
    }
    Some(heap.pop().unwrap().0)
}

impl Parser {
    pub fn print_tree(&self, tree: Option<&HuffmanNode>, prefix: Vec<i32>) {
        match tree {
            None => return,
            Some(t) => {
                if t.is_leaf() {
                    println!(" ");
                    print!("{} {} ", t.value, t.weight);
                    for i in prefix {
                        print!("{}", i);
                    }
                    //println!(" ")
                } else {
                    let mut l = prefix.clone();
                    let mut r = prefix.clone();
                    l.push(0);
                    r.push(1);
                    self.print_tree(Some(&t.left.as_ref().unwrap()), l);
                    self.print_tree(Some(&t.right.as_ref().unwrap()), r);
                }
            }
        }
    }
}
