use super::class::Class;
use super::entities_utils::FieldPath;
use super::game_events::GameEvent;
use super::read_bits::DemoParserError;
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
use bit_reverse::LookupReverse;
use cached::instant::Instant;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use memmap2::Mmap;
use memmap2::MmapOptions;
use soa_derive::StructOfArray;

// Wont fit in L1, evaluate if worth to use pointer method
const HUF_LOOKUPTABLE_MAXVALUE: u32 = (1 << 19) - 1;

pub struct Parser {
    // todo split into smaller parts
    pub ptr: usize,
    pub bytes: Mmap,
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
    pub huffman_lookup_table: Vec<(u32, u8)>,

    pub prop_name_to_path: AHashMap<String, [i32; 7]>,
    pub path_to_prop_name: AHashMap<[i32; 7], String>,

    pub wanted_prop_paths: AHashSet<[i32; 7]>,
    pub header: HashMap<String, String>,
    pub counter: HashMap<String, i32>,
    pub wanted_prop_og_names: Vec<String>,
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
    pub wanted_prop_og_names: Vec<String>,
    pub wanted_ticks: Vec<i32>,
    pub wanted_event: Option<String>,
    pub parse_ents: bool,
    pub parse_projectiles: bool,
    pub only_header: bool,
    pub count_props: bool,
    pub only_convars: bool,
}

impl Parser {
    pub fn new(mut settings: ParserInputs) -> Result<Self, DemoParserError> {
        let file = match std::fs::File::open(&settings.path) {
            Ok(f) => f,
            Err(e) => return Err(DemoParserError::FileError(e)),
        };
        let bytes = match unsafe { MmapOptions::new().map(&file) } {
            Ok(f) => f,
            Err(e) => return Err(DemoParserError::FileError(e)),
        };
        let fp_filler = FieldPath {
            last: 0,
            path: [-1, 0, 0, 0, 0, 0, 0],
        };
        settings.wanted_props.extend(vec![
            "tick".to_owned(),
            "steamid".to_owned(),
            "name".to_owned(),
        ]);
        let before = Instant::now();
        let huffman_table = create_huffman_lookup_table();
        println!("{:2?}", before.elapsed());
        Ok(Parser {
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
            paths: vec![fp_filler; 4096],
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
            huffman_lookup_table: huffman_table,
            prop_name_to_path: AHashMap::default(),
            wanted_prop_paths: AHashSet::default(),
            path_to_prop_name: AHashMap::default(),
            header: HashMap::default(),
            counter: HashMap::default(),
            wanted_prop_og_names: settings.wanted_prop_og_names,
        })
    }
}

fn msb(mut val: u32) -> u32 {
    let mut cnt = 0;
    while val > 0 {
        cnt = cnt + 1;
        val = val >> 1;
    }
    cnt
}

fn create_huffman_lookup_table() -> Vec<(u32, u8)> {
    let mut huffman_table = vec![(999999, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];
    let mut huffman_rev_table = vec![(999999, 255); HUF_LOOKUPTABLE_MAXVALUE as usize];

    huffman_table[0] = (0, 1);
    huffman_table[2] = (39, 2);
    huffman_table[24] = (8, 5);
    huffman_table[50] = (2, 6);
    huffman_table[51] = (29, 6);
    huffman_table[100] = (2, 6);
    huffman_table[101] = (29, 6);
    huffman_table[26] = (4, 5);
    huffman_table[432] = (30, 9);
    huffman_table[866] = (38, 10);
    huffman_table[55488] = (35, 16);
    huffman_table[55489] = (34, 16);
    huffman_table[27745] = (27, 15);
    huffman_table[55492] = (25, 16);
    huffman_table[55493] = (24, 16);
    huffman_table[55494] = (33, 16);
    huffman_table[55495] = (28, 16);
    huffman_table[55496] = (13, 16);
    huffman_table[110994] = (15, 17);
    huffman_table[110995] = (14, 17);
    huffman_table[27749] = (6, 15);
    huffman_table[111000] = (21, 17);
    huffman_table[111001] = (20, 17);
    huffman_table[111002] = (23, 17);
    huffman_table[111003] = (22, 17);
    huffman_table[111004] = (17, 17);
    huffman_table[111005] = (16, 17);
    huffman_table[111006] = (19, 17);
    huffman_table[111007] = (18, 17);
    huffman_table[3469] = (5, 12);
    huffman_table[1735] = (36, 11);
    huffman_table[217] = (10, 8);
    huffman_table[218] = (7, 8);
    huffman_table[438] = (12, 9);
    huffman_table[439] = (37, 9);
    huffman_table[220] = (9, 8);
    huffman_table[442] = (31, 9);
    huffman_table[443] = (26, 9);
    huffman_table[222] = (32, 8);
    huffman_table[223] = (3, 8);
    huffman_table[14] = (1, 4);
    huffman_table[15] = (11, 4);
    huffman_table[0] = (999999, 255);

    huffman_rev_table[0.swap_bits() >> RIGHTSHIFT_BITORDER] = (0, 1);
    huffman_rev_table[2.swap_bits() >> RIGHTSHIFT_BITORDER] = (39, 2);
    huffman_rev_table[24.swap_bits() >> RIGHTSHIFT_BITORDER] = (8, 5);
    huffman_rev_table[50.swap_bits() >> RIGHTSHIFT_BITORDER] = (2, 6);
    huffman_rev_table[51.swap_bits() >> RIGHTSHIFT_BITORDER] = (29, 6);
    huffman_rev_table[100.swap_bits() >> RIGHTSHIFT_BITORDER] = (2, 6);
    huffman_rev_table[101.swap_bits() >> RIGHTSHIFT_BITORDER] = (29, 6);
    huffman_rev_table[26.swap_bits() >> RIGHTSHIFT_BITORDER] = (4, 5);
    huffman_rev_table[432.swap_bits() >> RIGHTSHIFT_BITORDER] = (30, 9);
    huffman_rev_table[866.swap_bits() >> RIGHTSHIFT_BITORDER] = (38, 10);
    huffman_rev_table[55488.swap_bits() >> RIGHTSHIFT_BITORDER] = (35, 16);
    huffman_rev_table[55489.swap_bits() >> RIGHTSHIFT_BITORDER] = (34, 16);
    huffman_rev_table[27745.swap_bits() >> RIGHTSHIFT_BITORDER] = (27, 15);
    huffman_rev_table[55492.swap_bits() >> RIGHTSHIFT_BITORDER] = (25, 16);
    huffman_rev_table[55493.swap_bits() >> RIGHTSHIFT_BITORDER] = (24, 16);
    huffman_rev_table[55494.swap_bits() >> RIGHTSHIFT_BITORDER] = (33, 16);
    huffman_rev_table[55495.swap_bits() >> RIGHTSHIFT_BITORDER] = (28, 16);
    huffman_rev_table[55496.swap_bits() >> RIGHTSHIFT_BITORDER] = (13, 16);
    huffman_rev_table[110994.swap_bits() >> RIGHTSHIFT_BITORDER] = (15, 17);
    huffman_rev_table[110995.swap_bits() >> RIGHTSHIFT_BITORDER] = (14, 17);
    huffman_rev_table[27749.swap_bits() >> RIGHTSHIFT_BITORDER] = (6, 15);
    huffman_rev_table[111000.swap_bits() >> RIGHTSHIFT_BITORDER] = (21, 17);
    huffman_rev_table[111001.swap_bits() >> RIGHTSHIFT_BITORDER] = (20, 17);
    huffman_rev_table[111002.swap_bits() >> RIGHTSHIFT_BITORDER] = (23, 17);
    huffman_rev_table[111003.swap_bits() >> RIGHTSHIFT_BITORDER] = (22, 17);
    huffman_rev_table[111004.swap_bits() >> RIGHTSHIFT_BITORDER] = (17, 17);
    huffman_rev_table[111005.swap_bits() >> RIGHTSHIFT_BITORDER] = (16, 17);
    huffman_rev_table[111006.swap_bits() >> RIGHTSHIFT_BITORDER] = (19, 17);
    huffman_rev_table[111007.swap_bits() >> RIGHTSHIFT_BITORDER] = (18, 17);
    huffman_rev_table[3469.swap_bits() >> RIGHTSHIFT_BITORDER] = (5, 12);
    huffman_rev_table[1735.swap_bits() >> RIGHTSHIFT_BITORDER] = (36, 11);
    huffman_rev_table[217.swap_bits() >> RIGHTSHIFT_BITORDER] = (10, 8);
    huffman_rev_table[218.swap_bits() >> RIGHTSHIFT_BITORDER] = (7, 8);
    huffman_rev_table[438.swap_bits() >> RIGHTSHIFT_BITORDER] = (12, 9);
    huffman_rev_table[439.swap_bits() >> RIGHTSHIFT_BITORDER] = (37, 9);
    huffman_rev_table[220.swap_bits() >> RIGHTSHIFT_BITORDER] = (9, 8);
    huffman_rev_table[442.swap_bits() >> RIGHTSHIFT_BITORDER] = (31, 9);
    huffman_rev_table[443.swap_bits() >> RIGHTSHIFT_BITORDER] = (26, 9);
    huffman_rev_table[222.swap_bits() >> RIGHTSHIFT_BITORDER] = (32, 8);
    huffman_rev_table[223.swap_bits() >> RIGHTSHIFT_BITORDER] = (3, 8);
    huffman_rev_table[14.swap_bits() >> RIGHTSHIFT_BITORDER] = (1, 4);
    huffman_rev_table[15.swap_bits() >> RIGHTSHIFT_BITORDER] = (11, 4);
    huffman_rev_table[0.swap_bits() >> RIGHTSHIFT_BITORDER] = (999999, 255);

    const RIGHTSHIFT_BITORDER: u32 = 45;

    let mut v: Vec<u32> = vec![];
    for (idx, x) in huffman_table.iter().enumerate() {
        if x.0 != 999999 {
            v.push(idx as u32);
        }
    }

    let mut idx_msb_map = Vec::with_capacity(HUF_LOOKUPTABLE_MAXVALUE as usize);
    for i in 0..HUF_LOOKUPTABLE_MAXVALUE {
        idx_msb_map.push(msb(i));
    }
    for x in v {
        let shifta = msb(x);
        for (idx, pair) in idx_msb_map.iter().enumerate() {
            if x == idx as u32 >> pair - shifta {
                let peekbits = (idx as u64).swap_bits() >> RIGHTSHIFT_BITORDER;
                huffman_table[idx as usize] = huffman_table[x as usize];
                huffman_rev_table[peekbits as usize] = huffman_table[x as usize];
            }
        }
    }
    /*
    for x in v {
        let shifta = msb(x);
        for i in 0..HUF_LOOKUPTABLE_MAXVALUE {
            let shiftb = msb(i);
            if x == i >> shiftb - shifta {
                let peekbits = (i as u64).swap_bits() >> RIGHTSHIFT_BITORDER;
                huffman_table[i as usize] = huffman_table[x as usize];
                huffman_rev_table[peekbits as usize] = huffman_table[x as usize];
            }
        }
    }
    */
    for v in 0..HUF_LOOKUPTABLE_MAXVALUE {
        let p: u64 = (v as u64).swap_bits() >> RIGHTSHIFT_BITORDER;
        if p & 1 == 0 {
            huffman_rev_table[p as usize] = (0, 1);
        }
    }
    huffman_rev_table
}
