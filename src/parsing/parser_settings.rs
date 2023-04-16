use super::class::Class;
use super::entities::PathVariant;
use super::entities_utils::FieldPath;
use super::entities_utils::HuffmanNode;
use super::game_events::GameEvent;
use super::sendtables::Serializer;
use super::stringtables::StringTable;
use super::variants::PropColumn;
use crate::parsing::collect_data::ProjectileRecordVec;
use crate::parsing::entities::Entity;
use crate::parsing::entities::PlayerMetaData;
use crate::parsing::entities_utils::generate_huffman_tree;
use crate::parsing::sendtables::Decoder;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use nohash_hasher::*;
use smallvec::smallvec;
use soa_derive::StructOfArray;
use std::fs;

pub struct Parser {
    // todo split into smaller parts
    pub ptr: usize,
    pub bytes: Vec<u8>,
    pub ge_list: Option<AHashMap<i32, Descriptor_t>>,
    pub serializers: AHashMap<String, Serializer, RandomState>,
    pub cls_by_id: AHashMap<u32, Class, RandomState>,
    pub cls_by_name: AHashMap<String, Class, RandomState>,
    pub cls_bits: Option<u32>,
    pub entities: AHashMap<i32, Entity, RandomState>,
    pub tick: i32,
    pub wanted_ticks: AHashSet<i32, RandomState>,
    pub wanted_props: Vec<String>,
    pub wanted_event: Option<String>,
    pub players: AHashMap<i32, PlayerMetaData, RandomState>,
    pub output: AHashMap<String, PropColumn, RandomState>,
    pub game_events: Vec<GameEvent>,
    pub parse_entities: bool,
    pub projectiles: AHashSet<i32, RandomState>,
    pub projectile_records: ProjectileRecordVec,

    pub paths: Vec<FieldPath>,

    pub pattern_cache: AHashMap<u64, Decoder, RandomState>,
    pub baselines: AHashMap<u32, Vec<u8>, RandomState>,
    pub string_tables: Vec<StringTable>,
    pub cache: AHashMap<u128, (String, Decoder)>,
    pub teams: Teams,
    pub game_events_counter: AHashMap<String, i32>,
    pub props_counter: AHashMap<String, i32>,
    pub parse_projectiles: bool,
    pub count_props: bool,
    pub rules_entity_id: Option<i32>,
    pub uniq_message_ids: AHashSet<u32>,
    pub convars: AHashMap<String, String>,
    pub only_convars: bool,
    pub chat_messages: ChatMessageRecordVec,
    pub item_drops: EconItemVec,
    pub player_end_data: PlayerEndDataVec,
    pub skins: EconItemVec,

    pub history: AHashMap<u64, (u64, Decoder), RandomState>,

    pub huffman_codes: [u32; 300_000],
    pub huffman_codes2: [u32; 300_000],
    pub symbol_bits: [u8; 1024],
}
#[derive(Debug, Clone)]
pub struct Teams {
    pub team1_entid: Option<i32>,
    pub team2_entid: Option<i32>,
    pub team3_entid: Option<i32>,
}
impl Teams {
    pub fn new() -> Self {
        Teams {
            team1_entid: None,
            team2_entid: None,
            team3_entid: None,
        }
    }
}

#[derive(Debug, StructOfArray)]
pub struct ChatMessageRecord {
    pub entity_idx: Option<i32>,
    pub param1: Option<String>,
    pub param2: Option<String>,
    pub param3: Option<String>,
    pub param4: Option<String>,
}
#[derive(Debug, StructOfArray)]
pub struct EconItem {
    pub account_id: Option<u32>,
    pub item_id: Option<u64>,
    pub def_index: Option<u32>,
    pub paint_index: Option<u32>,
    pub rarity: Option<u32>,
    pub quality: Option<u32>,
    pub paint_wear: Option<u32>,
    pub paint_seed: Option<u32>,
    pub quest_id: Option<u32>,
    pub dropreason: Option<u32>,
    pub custom_name: Option<String>,
    pub inventory: Option<u32>,
    pub ent_idx: Option<i32>,
    pub steamid: Option<u64>,
}
#[derive(Debug, StructOfArray)]
pub struct PlayerEndData {
    pub steamid: Option<u64>,
    pub name: Option<String>,
    pub team_number: Option<i32>,
}

#[derive(Debug)]
pub struct ParserInputs {
    pub path: String,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: Vec<i32>,
    pub wanted_event: Option<String>,
    pub parse_ents: bool,
    pub parse_projectiles: bool,
    pub only_header: bool,
    pub count_props: bool,
    pub only_convars: bool,
}

impl Parser {
    pub fn new(mut settings: ParserInputs) -> Self {
        let bytes = fs::read(settings.path).unwrap();
        // let tree = generate_huffman_tree().unwrap();
        let fp_filler = FieldPath {
            last: 0,
            path: [-1, 0, 0, 0, 0, 0, 0],
        };
        settings.wanted_props.extend(vec![
            "tick".to_owned(),
            "steamid".to_owned(),
            "name".to_owned(),
        ]);
        let mut a: [u32; 300_000] = [999999; 300_000];
        a[0] = 0;
        a[2] = 39;
        a[24] = 8;
        a[50] = 2;
        a[51] = 29;
        a[100] = 2;
        a[101] = 29;
        a[26] = 4;
        a[432] = 30;
        a[866] = 38;
        a[55488] = 35;
        a[55489] = 34;
        a[27745] = 27;
        a[55492] = 25;
        a[55493] = 24;
        a[55494] = 33;
        a[55495] = 28;
        a[55496] = 13;
        a[110994] = 15;
        a[110995] = 14;
        a[27749] = 6;
        a[111000] = 21;
        a[111001] = 20;
        a[111002] = 23;
        a[111003] = 22;
        a[111004] = 17;
        a[111005] = 16;
        a[111006] = 19;
        a[111007] = 18;
        a[3469] = 5;
        a[1735] = 36;
        a[217] = 10;
        a[218] = 7;
        a[438] = 12;
        a[439] = 37;
        a[220] = 9;
        a[442] = 31;
        a[443] = 26;
        a[222] = 32;
        a[223] = 3;
        a[14] = 1;
        a[15] = 11;

        let mut b = a.clone();
        b[0] = 999999;

        let mut v: Vec<u32> = vec![];
        for (idx, x) in b.iter().enumerate() {
            if x != &999999 {
                v.push(idx as u32);
            }
        }

        let mut ans = [0_u8; 1024];

        for x in v {
            let shifta = MSB(x);
            let sym = b[x as usize];
            ans[sym as usize] = shifta as u8;
            for i in 0..300_000 {
                let shiftb = MSB(i);

                if x == i >> shiftb - shifta {
                    b[i as usize] = b[x as usize];
                    // println!("{:#032b} {:#032b} {} {}", x, i, x, i);
                    // println!("{:#032b} {:#032b} {} ", x, i >> shifta, shifta);
                }
            }
        }
        ans[0] = 1;
        println!("{}", ans[0]);
        /*
        for x in v {
            let n = 32 - x.leading_zeros();
            for i in 0..140_000 {
                if (i << (32 - n)) == x << (32 - n) {
                    println!("{}", a[i as usize]);
                    a[i as usize] = a[x as usize];
                    // println!("{:#032b} {:#032b} {} {}", x, i, x, i);
                }
            }
        }
        */

        Parser {
            serializers: AHashMap::default(),
            ptr: 0,
            ge_list: None,
            bytes: bytes,
            cls_by_id: AHashMap::default(),
            cls_by_name: AHashMap::default(),
            entities: AHashMap::default(),
            cls_bits: None,
            tick: -99999,
            wanted_props: settings.wanted_props,
            players: AHashMap::default(),
            output: AHashMap::default(),
            wanted_ticks: AHashSet::from_iter(settings.wanted_ticks),
            game_events: vec![],
            wanted_event: settings.wanted_event,
            parse_entities: settings.parse_ents,
            projectiles: AHashSet::default(),
            projectile_records: ProjectileRecordVec::new(),
            pattern_cache: AHashMap::default(),
            baselines: AHashMap::default(),
            string_tables: vec![],
            cache: AHashMap::default(),
            paths: vec![fp_filler; 10000],
            teams: Teams::new(),
            game_events_counter: AHashMap::default(),
            props_counter: AHashMap::default(),
            parse_projectiles: settings.parse_projectiles,
            count_props: settings.count_props,
            rules_entity_id: None,
            uniq_message_ids: AHashSet::default(),
            convars: AHashMap::default(),
            only_convars: settings.only_convars,
            chat_messages: ChatMessageRecordVec::new(),
            item_drops: EconItemVec::new(),
            skins: EconItemVec::new(),
            player_end_data: PlayerEndDataVec::new(),
            huffman_codes: a,
            history: AHashMap::default(),
            huffman_codes2: b,
            symbol_bits: ans,
        }
    }
}

fn MSB(mut val: u32) -> u32 {
    let mut cnt = 0;
    while val > 0 {
        cnt = cnt + 1;
        val = val >> 1;
    }
    cnt
}
