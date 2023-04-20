use super::class::Class;
use super::entities_utils::FieldPath;
use super::game_events::GameEvent;
use super::sendtables::Serializer;
use super::stringtables::StringTable;
use super::variants::PropColumn;
use crate::parsing::collect_data::ProjectileRecordVec;
use crate::parsing::entities::Entity;
use crate::parsing::entities::PlayerMetaData;
use crate::parsing::sendtables::Decoder;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::HashMap;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use soa_derive::StructOfArray;
use std::fs;

const HUF_LOOKUPTABLE_MAXVALUE: u32 = 1 << 19 - 1;
const MAX_HUF_SYMBOL: usize = 40;

pub struct Parser {
    // todo split into smaller parts
    pub ptr: usize,
    pub bytes: Vec<u8>,
    pub ge_list: Option<AHashMap<i32, Descriptor_t>>,
    pub serializers: AHashMap<String, Serializer, RandomState>,
    pub cls_by_id: [Option<Class>; 560],
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
    pub huffman_lookup_table: [(u32, u8); HUF_LOOKUPTABLE_MAXVALUE as usize],

    pub prop_name_to_path: AHashMap<String, [i32; 7]>,
    pub path_to_prop_name: AHashMap<[i32; 7], String>,

    pub wanted_prop_paths: AHashSet<[i32; 7]>,
    pub header: HashMap<String, String>,
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
        let mut a: [(u32, u8); HUF_LOOKUPTABLE_MAXVALUE as usize] =
            [(999999, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];
        a[0] = (0, 1);
        a[2] = (39, 2);
        a[24] = (8, 5);
        a[50] = (2, 6);
        a[51] = (29, 6);
        a[100] = (2, 6);
        a[101] = (29, 6);
        a[26] = (4, 5);
        a[432] = (30, 9);
        a[866] = (38, 10);
        a[55488] = (35, 16);
        a[55489] = (34, 16);
        a[27745] = (27, 15);
        a[55492] = (25, 16);
        a[55493] = (24, 16);
        a[55494] = (33, 16);
        a[55495] = (28, 16);
        a[55496] = (13, 16);
        a[110994] = (15, 17);
        a[110995] = (14, 17);
        a[27749] = (6, 15);
        a[111000] = (21, 17);
        a[111001] = (20, 17);
        a[111002] = (23, 17);
        a[111003] = (22, 17);
        a[111004] = (17, 17);
        a[111005] = (16, 17);
        a[111006] = (19, 17);
        a[111007] = (18, 17);
        a[3469] = (5, 12);
        a[1735] = (36, 11);
        a[217] = (10, 8);
        a[218] = (7, 8);
        a[438] = (12, 9);
        a[439] = (37, 9);
        a[220] = (9, 8);
        a[442] = (31, 9);
        a[443] = (26, 9);
        a[222] = (32, 8);
        a[223] = (3, 8);
        a[14] = (1, 4);
        a[15] = (11, 4);

        /*
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

        a[0] = (999999, 255);
        let mut v: Vec<u32> = vec![];
        for (idx, x) in a.iter().enumerate() {
            if x.0 != 999999 {
                v.push(idx as u32);
            }
        }
        let mut found = vec![];
        for x in v {
            let shifta = MSB(x);
            let (sym, bitlen) = a[x as usize];
            for i in 0..HUF_LOOKUPTABLE_MAXVALUE {
                let shiftb = MSB(i);
                if x == i >> shiftb - shifta {
                    found.push(x);
                    a[i as usize] = a[x as usize];
                }
            }
        }

        Parser {
            serializers: AHashMap::default(),
            ptr: 0,
            ge_list: None,
            bytes: bytes,
            // JUST LOL
            cls_by_id: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            ],
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
            history: AHashMap::default(),
            huffman_lookup_table: a,
            prop_name_to_path: AHashMap::default(),
            wanted_prop_paths: AHashSet::default(),
            path_to_prop_name: AHashMap::default(),
            header: HashMap::default(),
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
