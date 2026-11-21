use crate::first_pass::parser_settings::ParserInputs;
use crate::first_pass::prop_controller::is_grenade_or_weapon;
use crate::first_pass::prop_controller::PropController;
use crate::first_pass::read_bits::Bitreader;
use crate::first_pass::read_bits::DemoParserError;
use crate::first_pass::sendtables::find_field;
use crate::first_pass::sendtables::find_field_with_parent;
use crate::first_pass::sendtables::get_decoder_from_field;
use crate::first_pass::sendtables::get_propinfo;
use crate::first_pass::sendtables::Field;
use crate::first_pass::sendtables::FieldInfo;
use crate::first_pass::sendtables::Serializer;
use crate::second_pass::collect_data::PropType;
use crate::second_pass::game_events::GameEventInfo;
use crate::second_pass::other_netmessages::Class;
use crate::second_pass::parser_settings::SecondPassParser;
use crate::second_pass::path_ops::*;
use crate::second_pass::variants::Variant;
use ahash::AHashMap;
use ahash::AHashSet;
use csgoproto::CsvcMsgPacketEntities;
use prost::Message;

/// path[..last] equal, written out so it doesn't become a memcmp call (last <= 5).
#[inline(always)]
fn same_prefix(a: &FieldPath, b: &FieldPath, last: usize) -> bool {
    for k in 0..last {
        if a.path[k] != b.path[k] {
            return false;
        }
    }
    true
}

// ---------- needed-props mode ----------
/// Prop ids at or above this are fixed constants (weapons, purchases, skins, grenade ammo, ...)
/// that the second pass reads directly; fields with such ids are always decoded.
pub const CONST_PROP_ID_MIN: u32 = 500_000;
/// Normal field prop ids start here. Anything below is unassigned (the prop controller never
/// registered the field, e.g. for chickens or observer pawns), so nothing can look it up.
const NORMAL_PROP_ID_MIN: u32 = 1000;

/// Prop ids the second pass must decode for the requested output: the requested props,
/// prop-state filters, the special ids those props/events read, and only the constant-id
/// ranges (weapons, purchases, skins, gloves, ammo) a requested prop or event uses.
pub struct NeededSet {
    /// Bit per normal prop id (< CONST_PROP_ID_MIN): ids are dense small integers.
    bits: Vec<u64>,
    /// Explicit ids >= CONST_PROP_ID_MIN (sorted).
    large: Vec<u32>,
    /// Half-open [start, end) ranges of constant (flattened) prop ids.
    ranges: Vec<(u32, u32)>,
}

impl NeededSet {
    fn new(ids: AHashSet<u32>, ranges: Vec<(u32, u32)>) -> Self {
        let mut bits = vec![];
        let mut large = vec![];
        for id in ids {
            if id < CONST_PROP_ID_MIN {
                let w = (id >> 6) as usize;
                if bits.len() <= w {
                    bits.resize(w + 1, 0u64);
                }
                bits[w] |= 1 << (id & 63);
            } else {
                large.push(id);
            }
        }
        large.sort_unstable();
        large.dedup();
        NeededSet { bits, large, ranges }
    }
    #[inline(always)]
    fn contains(&self, id: u32) -> bool {
        if id < NORMAL_PROP_ID_MIN {
            return false;
        }
        if id < CONST_PROP_ID_MIN {
            return self.bits.get((id >> 6) as usize).map_or(false, |w| (w >> (id & 63)) & 1 == 1);
        }
        self.large.binary_search(&id).is_ok() || self.ranges.iter().any(|&(a, b)| id >= a && id < b)
    }
}

pub fn compute_needed(pc: &PropController, settings: &ParserInputs) -> NeededSet {
    use crate::first_pass::prop_controller::*;
    let sp = &pc.special_ids;
    let mut ids: AHashSet<u32> = AHashSet::default();
    let mut ranges: Vec<(u32, u32)> = vec![];
    let add = |ids: &mut AHashSet<u32>, v: &[Option<u32>]| ids.extend(v.iter().flatten());

    // Requested props (normal field ids) and prop-state filters.
    ids.extend(pc.prop_infos.iter().map(|p| p.id));
    ids.extend(pc.wanted_prop_state_infos.iter().map(|p| p.base.id));
    // Core bookkeeping, always: player identity/linking on controllers, team entities, round and
    // match counters on the rules entity, weapon/grenade owner handles. None of it is on pawns.
    add(&mut ids, &[sp.teamnum, sp.player_name, sp.steamid, sp.player_pawn, sp.team_team_num]);
    add(
        &mut ids,
        &[
            sp.total_rounds_played,
            sp.round_win_reason,
            sp.round_start_count,
            sp.round_end_count,
            sp.match_end_count,
        ],
    );
    add(&mut ids, &[sp.h_owner_entity, sp.grenade_owner_id]);

    let cells_player = [
        sp.cell_x_player,
        sp.cell_y_player,
        sp.cell_z_player,
        sp.cell_x_offset_player,
        sp.cell_y_offset_player,
        sp.cell_z_offset_player,
    ];
    let weapon = [sp.active_weapon, sp.item_def];
    let my_weapons = (MY_WEAPONS_OFFSET, MY_WEAPONS_OFFSET + FLATTENED_VEC_MAX_LEN);
    let ammo = [(GRENADE_AMMO_ID, GRENADE_AMMO_ID + 1), (FLASHBANG_AMMO_ID, FLASHBANG_AMMO_ID + 1)];
    let weapon_skin = (WEAPON_SKIN_ID, WEAPON_SKIN_ID + 1000);
    let glove = (GLOVE_PAINT_ID, GLOVE_PAINT_ID + 1000);
    let mut everything = false;

    if pc.needs_velocity || pc.event_with_velocity {
        add(&mut ids, &cells_player);
        // VelocitySample::push compares the pawn lifetime start so velocity never spans a respawn.
        add(&mut ids, &[sp.pawn_lifetime_start]);
    }
    for p in pc.prop_infos.iter() {
        match p.prop_type {
            PropType::Team => add(&mut ids, &[sp.player_team_pointer]),
            PropType::Weapon => add(&mut ids, &weapon),
            PropType::Button => add(&mut ids, &[sp.buttons]),
            PropType::Custom => match p.id {
                PLAYER_X_ID | PLAYER_Y_ID | PLAYER_Z_ID | VELOCITY_ID | VELOCITY_X_ID | VELOCITY_Y_ID | VELOCITY_Z_ID => add(&mut ids, &cells_player),
                PITCH_ID | YAW_ID => add(&mut ids, &[sp.eye_angles]),
                IS_ALIVE_ID => add(&mut ids, &[sp.life_state]),
                IS_AIRBORNE_ID => add(&mut ids, &[sp.is_airborn]),
                STAMINA_ID => add(&mut ids, &[sp.stamina]),
                AGENT_SKIN_ID => add(&mut ids, &[sp.agent_skin_idx]),
                WEAPON_NAME_ID => add(&mut ids, &weapon),
                WEAPON_ORIGINGAL_OWNER_ID => {
                    add(&mut ids, &weapon);
                    add(&mut ids, &[sp.orig_own_low, sp.orig_own_high, sp.weapon_prev_owner]);
                }
                WEAPON_SKIN_NAME | WEAPON_SKIN_ID | WEAPON_PAINT_SEED | WEAPON_FLOAT | WEAPON_STICKERS_ID => {
                    add(&mut ids, &weapon);
                    ranges.push(weapon_skin);
                    ids.extend([WEAPON_PAINT_SEED, WEAPON_FLOAT]);
                }
                INVENTORY_ID | INVENTORY_AS_IDS_ID | INVENTORY_AS_IDS_BITMASK => {
                    // Inventory is empty unless the player is alive (find_is_alive).
                    add(&mut ids, &weapon);
                    add(&mut ids, &[sp.life_state]);
                    ranges.push(my_weapons);
                    ranges.extend(ammo);
                }
                GLOVE_PAINT_ID | GLOVE_SKIN | GLOVE_PAINT_SEED | GLOVE_PAINT_FLOAT => {
                    ranges.push(glove);
                    ids.extend([GLOVE_PAINT_SEED, GLOVE_PAINT_FLOAT, GLOVE_SKIN]);
                }
                // Not entity fields, or resolved through their own (requested) field id.
                ENTITY_ID_ID | USERID_ID | USERCMD_INPUT_HISTORY_BASEID | USERCMD_SUBTICK_MOVES_BASEID => {}
                _ => match p.prop_name.as_str() {
                    "CCSPlayerPawn.m_bSpottedByMask" | "CCSPlayerController.m_iCompTeammateColor" => {}
                    // Unknown custom prop: don't guess its dependencies.
                    _ => everything = true,
                },
            },
            _ => {}
        }
    }
    if pc.wanted_player_props.iter().any(|n| n == "inventory") {
        add(&mut ids, &weapon);
        add(&mut ids, &[sp.life_state]);
        ranges.push(my_weapons);
        ranges.extend(ammo);
    }
    if settings.parse_projectiles || settings.parse_grenades {
        add(
            &mut ids,
            &[
                sp.m_vec_x_grenade,
                sp.m_vec_y_grenade,
                sp.m_vec_z_grenade,
                sp.m_cell_x_grenade,
                sp.m_cell_y_grenade,
                sp.m_cell_z_grenade,
                sp.is_incendiary_grenade,
            ],
        );
    }
    if settings.parse_weapons {
        add(&mut ids, &weapon);
        add(&mut ids, &[sp.orig_own_low, sp.orig_own_high, sp.weapon_prev_owner]);
    }
    let wants_event = |name: &str| settings.wanted_events.first().map_or(false, |e| e == "all") || settings.wanted_events.iter().any(|e| e == name);
    if wants_event("item_purchase") || wants_event("item_sold") {
        add(
            &mut ids,
            &[
                sp.in_buy_zone,
                sp.custom_name,
                sp.sellback_entry_def_idx,
                sp.sellback_entry_n_cost,
                sp.sellback_entry_prev_armor,
                sp.sellback_entry_prev_helmet,
                sp.sellback_entry_h_item,
                sp.weapon_purchase_count,
            ],
        );
        add(&mut ids, &weapon);
        // The purchase event reports the bought weapon's skin, paint seed, float and stickers.
        ranges.push(weapon_skin);
        for base in [
            ITEM_PURCHASE_COUNT,
            ITEM_PURCHASE_DEF_IDX,
            ITEM_PURCHASE_COST,
            ITEM_PURCHASE_HANDLE,
            ITEM_PURCHASE_NEW_DEF_IDX,
        ] {
            ranges.push((base, base + FLATTENED_VEC_MAX_LEN));
        }
    }
    if everything {
        ids.extend(sp.all());
        ranges.push((CONST_PROP_ID_MIN, u32::MAX));
    }
    NeededSet::new(ids, ranges)
}

#[inline(always)]
fn prop_is_needed(needed: &NeededSet, id: u32) -> bool {
    needed.contains(id)
}

/// Lexicographically last path at which a needed field of a class can appear. Array/vector
/// element indices are i32::MAX ("any element").
#[derive(Debug, Clone, Copy)]
pub struct PathBound {
    path: [i32; 7],
    len: usize,
}

impl PathBound {
    /// Path sorts after the bound (paths within an update are ascending).
    #[inline(always)]
    fn is_past(&self, p: &FieldPath) -> bool {
        let n = p.last as usize + 1;
        for k in 0..n.min(self.len) {
            let a = p.path[k] as i32;
            let b = self.path[k];
            if a != b {
                return a > b;
            }
        }
        n > self.len
    }
}

fn compute_bound(ser: &Serializer, needed: &NeededSet) -> Option<PathBound> {
    fn walk(f: &Field, path: &mut Vec<i32>, needed: &NeededSet, best: &mut Option<Vec<i32>>) {
        match f {
            Field::Value(v) => {
                if prop_is_needed(needed, v.prop_id) && best.as_ref().map_or(true, |b| *path > *b) {
                    *best = Some(path.clone());
                }
            }
            Field::Array(a) => {
                path.push(i32::MAX);
                walk(&a.field_enum, path, needed, best);
                path.pop();
            }
            Field::Vector(v) => {
                path.push(i32::MAX);
                walk(&v.field_enum, path, needed, best);
                path.pop();
            }
            Field::Serializer(sf) => walk_ser(&sf.serializer, path, needed, best),
            Field::Pointer(pf) => walk_ser(&pf.serializer, path, needed, best),
            Field::None => {}
        }
    }
    fn walk_ser(ser: &Serializer, path: &mut Vec<i32>, needed: &NeededSet, best: &mut Option<Vec<i32>>) {
        for (i, f) in ser.fields.iter().enumerate() {
            path.push(i as i32);
            walk(f, path, needed, best);
            path.pop();
        }
    }
    let mut best = None;
    walk_ser(ser, &mut vec![], needed, &mut best);
    let best = best?;
    if best.len() > 7 {
        // Deeper than a field path can be: no bound, decode everything.
        return Some(PathBound { path: [i32::MAX; 7], len: 7 });
    }
    let mut path = [0i32; 7];
    path[..best.len()].copy_from_slice(&best);
    Some(PathBound { path, len: best.len() })
}

const NSERIALBITS: u32 = 17;
const STOP_READING_SYMBOL: u8 = 39;
const HUFFMAN_CODE_MAXLEN: u32 = 17;

#[derive(Debug, Clone)]
pub struct Entity {
    pub cls_id: u32,
    pub entity_id: i32,
    pub props: AHashMap<u32, Variant>,
    pub entity_type: EntityType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerMetaData {
    pub player_entity_id: Option<i32>,
    pub steamid: Option<u64>,
    pub controller_entid: Option<i32>,
    pub name: Option<String>,
    pub team_num: Option<u32>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    PlayerController,
    Rules,
    Projectile,
    Team,
    Normal,
    C4,
}
enum EntityCmd {
    Delete,
    CreateAndUpdate,
    Update,
}

impl<'a> SecondPassParser<'a> {
    pub fn parse_packet_ents(&mut self, bytes: &[u8], is_fullpacket: bool) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
        let msg = match CsvcMsgPacketEntities::decode(bytes) {
            Err(_) => return Err(DemoParserError::MalformedMessage),
            Ok(msg) => msg,
        };

        let mut bitreader = Bitreader::new(msg.entity_data());
        // Per-entity bit lengths: lets the needed-props mode jump over the rest of an update.
        let mut offsets_reader = Bitreader::new(msg.serialized_entities());
        let mut entity_id: i32 = -1;
        let mut events_to_emit = vec![];
        for _ in 0..msg.updated_entries() {
            entity_id += 1 + (bitreader.read_u_bit_var()? as i32);
            // Read 2 bits to know which operation should be done to the entity.
            let cmd = match bitreader.read_nbits(2)? {
                0b01 => EntityCmd::Delete,
                0b11 => EntityCmd::Delete,
                0b10 => EntityCmd::CreateAndUpdate,
                0b00 => EntityCmd::Update,
                _ => return Err(DemoParserError::ImpossibleCmd),
            };

            match cmd {
                EntityCmd::Delete => {
                    self.projectiles.remove(&entity_id);
                    self.weapons.remove(&entity_id);
                    if let Some(entry) = self.entities.get_mut(entity_id as usize) {
                        *entry = None;
                    }
                }
                EntityCmd::CreateAndUpdate => {
                    let bits_this_entity = offsets_reader.read_varint()?;
                    self.create_new_entity(&mut bitreader, &entity_id, &mut events_to_emit)?;
                    if self.needed_mode {
                        // The offset covers the update after the create header.
                        let end = bitreader.bit_pos() + bits_this_entity as usize;
                        self.update_entity_needed(&mut bitreader, entity_id, end, &mut events_to_emit, is_fullpacket, true)?;
                    } else {
                        self.update_entity(&mut bitreader, entity_id, false, &mut events_to_emit, is_fullpacket)?;
                    }
                }
                EntityCmd::Update => {
                    if msg.has_pvs_vis_bits_deprecated() != 0 {
                        // Most entities pass trough here. Seems like entities that are not updated.
                        if bitreader.read_nbits(2)? & 0x01 == 1 {
                            continue;
                        }
                    }
                    let bits_this_entity = offsets_reader.read_varint()?;
                    if self.needed_mode {
                        let end = bitreader.bit_pos() + bits_this_entity as usize;
                        self.update_entity_needed(&mut bitreader, entity_id, end, &mut events_to_emit, is_fullpacket, false)?;
                        continue;
                    }
                    self.update_entity(&mut bitreader, entity_id, false, &mut events_to_emit, is_fullpacket)?;
                }
            }
        }
        if !events_to_emit.is_empty() {
            self.emit_events(events_to_emit)?;
        }
        Ok(())
    }

    pub fn update_entity(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        is_baseline: bool,
        events_to_emit: &mut Vec<GameEventInfo>,
        is_fullpacket: bool,
    ) -> Result<(), DemoParserError> {
        let _pp = crate::second_pass::parser::prof_on().then(std::time::Instant::now);
        let n_updates = self.parse_paths(bitreader)?;
        if let Some(t) = _pp {
            crate::second_pass::parser::PROF_PATHS_NS.with(|c| c.set(c.get() + t.elapsed().as_nanos() as u64));
        }
        let _pd = crate::second_pass::parser::prof_on().then(std::time::Instant::now);
        let n_updated_values = self.decode_entity_update(bitreader, entity_id, n_updates, is_fullpacket, is_baseline, events_to_emit)?;
        if let Some(t) = _pd {
            crate::second_pass::parser::PROF_DECODE_NS.with(|c| c.set(c.get() + t.elapsed().as_nanos() as u64));
        }
        if n_updated_values > 0 {
            self.gather_extra_info(&entity_id, is_baseline)?;
        }
        Ok(())
    }
    /// Needed-props update for an existing entity. Paths come first (ascending), then values in
    /// path order; `entity_end` (from the offsets) is where the update stops.
    /// - class without needed fields: jump to the end;
    /// - while parsing paths, if the first path past the class bound comes before any path at or
    ///   below it, nothing needed is in this update: jump to the end;
    /// - otherwise decode the needed values up to the bound exactly like the normal path
    ///   (events + insert), skip the others, and jump to the end.
    /// gather_extra_info (bookkeeping from the entity's core props) runs when a needed value was
    /// decoded, plus always on the create update: team entities only register in the
    /// non-baseline call, even when their team number came from the baseline.
    fn update_entity_needed(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        entity_end: usize,
        events_to_emit: &mut Vec<GameEventInfo>,
        is_fullpacket: bool,
        is_create: bool,
    ) -> Result<(), DemoParserError> {
        let cls_id = match self.entities.get(entity_id as usize) {
            Some(Some(e)) => e.cls_id as usize,
            _ => return Err(DemoParserError::EntityNotFound),
        };
        let jump_to_end = |br: &mut Bitreader| -> Result<(), DemoParserError> {
            let pos = br.bit_pos();
            if pos > entity_end {
                return Err(DemoParserError::MalformedMessage);
            }
            br.skip((entity_end - pos) as u32);
            Ok(())
        };
        // No needed field: never a controller/team entity (those always have needed fields),
        // so there is no bookkeeping to do either.
        let Some(bound) = self.class_bound(cls_id)? else {
            return jump_to_end(bitreader);
        };
        let (last_relevant, any_path) = self.parse_paths_bounded(bitreader, &bound)?;
        let mut n_decoded = 0;
        if let Some(last_relevant) = last_relevant {
            n_decoded = self.decode_needed_values(bitreader, entity_id, cls_id, last_relevant, false, events_to_emit, is_fullpacket)?;
        }
        jump_to_end(bitreader)?;
        if n_decoded > 0 || (is_create && any_path) {
            self.gather_extra_info(&entity_id, false)?;
        }
        Ok(())
    }

    /// Needed-props version of applying the instance baseline to a freshly created entity:
    /// decode only up to the class's last needed path (the baseline isn't streamed, so nothing
    /// has to be skipped after it). Same semantics as update_entity(is_baseline = true).
    fn apply_baseline_needed(&mut self, baseline: &[u8], entity_id: i32, cls_id: usize) -> Result<(), DemoParserError> {
        let Some(bound) = self.class_bound(cls_id)? else {
            return Ok(());
        };
        let mut br = Bitreader::new(baseline);
        let (last_relevant, _any_path) = self.parse_paths_bounded(&mut br, &bound)?;
        let mut n_decoded = 0;
        if let Some(last_relevant) = last_relevant {
            n_decoded = self.decode_needed_values(&mut br, entity_id, cls_id, last_relevant, true, &mut vec![], false)?;
        }
        if n_decoded > 0 {
            self.gather_extra_info(&entity_id, true)?;
        }
        Ok(())
    }

    /// Per-class bound for the needed-props mode (computed on first use).
    fn class_bound(&mut self, cls_id: usize) -> Result<Option<PathBound>, DemoParserError> {
        if self.class_bounds.len() <= cls_id {
            self.class_bounds.resize(cls_id + 1, None);
        }
        if let Some(b) = self.class_bounds[cls_id] {
            return Ok(b);
        }
        let b = match self.cls_by_id.get(cls_id) {
            Some(c) => compute_bound(&c.serializer, &self.needed_ids),
            None => return Err(DemoParserError::ClassNotFound),
        };
        self.class_bounds[cls_id] = Some(b);
        Ok(b)
    }

    /// Values for paths[..=last_relevant]: needed ones decoded exactly like decode_entity_update
    /// (events unless baseline, then insert), the rest skipped (sibling runs grouped).
    fn decode_needed_values(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        cls_id: usize,
        last_relevant: usize,
        is_baseline: bool,
        events_to_emit: &mut Vec<GameEventInfo>,
        is_fullpacket: bool,
    ) -> Result<usize, DemoParserError> {
        let mut n_decoded = 0;
        let entity = match self.entities.get_mut(entity_id as usize) {
            Some(Some(entity)) => entity,
            _ => return Err(DemoParserError::EntityNotFound),
        };
        let class = match self.cls_by_id.get(cls_id) {
            Some(cls) => cls,
            None => return Err(DemoParserError::ClassNotFound),
        };
        let paths = &self.paths[..=last_relevant];
        let mut i = 0;
        while i < paths.len() {
            let path = &paths[i];
            let (field, parent_is_container) = find_field_with_parent(path, &class.serializer)?;
            let decoder = get_decoder_from_field(field)?;
            // Decide from the field's base prop id first (what get_propinfo starts from): it only
            // remaps the flattened constant props, and their remapped ids stay inside the same
            // constant ranges (grenade/flashbang ammo are always needed together). So an unneeded
            // base id means unneeded, without the get_propinfo call.
            let base_id = match field {
                Field::Value(v) => Some(v.prop_id),
                Field::Vector(_) => match field.get_inner(0) {
                    Ok(Field::Value(inner)) => Some(inner.prop_id),
                    _ => None,
                },
                // No prop info (e.g. pointer presence): nothing to store.
                _ => None,
            };
            let (needed, field_info) = match base_id {
                Some(id) if prop_is_needed(&self.needed_ids, id) => {
                    let field_info = get_propinfo(field, path);
                    (field_info.map_or(false, |fi| prop_is_needed(&self.needed_ids, fi.prop_id)), field_info)
                }
                _ => (false, None),
            };
            if needed {
                let result = bitreader.decode(&decoder, self.qf_mapper)?;
                if !is_baseline {
                    SecondPassParser::listen_for_events(
                        entity,
                        &result,
                        field,
                        field_info,
                        &self.prop_controller,
                        &self.prop_controller.special_ids,
                        is_fullpacket,
                        events_to_emit,
                    );
                }
                SecondPassParser::insert_field(entity, result, field_info);
                n_decoded += 1;
                i += 1;
                continue;
            }
            // Unneeded: skip this value, and with it the following siblings of the same
            // array/vector (same field). Flattened vector props have constant ids, so they are
            // always needed and never grouped here.
            let mut run = 1;
            if parent_is_container {
                let last = path.last as usize;
                while i + run < paths.len() {
                    let next = &paths[i + run];
                    if next.last as usize != last || !same_prefix(next, path, last) {
                        break;
                    }
                    run += 1;
                }
            }
            match Bitreader::fixed_bits(&decoder, self.qf_mapper) {
                Some(bits) => bitreader.skip(bits * run as u32),
                None => {
                    for _ in 0..run {
                        bitreader.skip_value(&decoder, self.qf_mapper)?;
                    }
                }
            }
            i += run;
        }
        Ok(n_decoded)
    }
    /// `parse_paths` that stops early when the update can't contain a needed field: returns None
    /// if the first path past `bound` comes before any path at or below it. Otherwise parses all
    /// paths (values start after them) and returns the index of the last path at or below it.
    fn parse_paths_bounded(&mut self, bitreader: &mut Bitreader, bound: &PathBound) -> Result<(Option<usize>, bool), DemoParserError> {
        let mut fp = generate_fp();
        let mut idx = 0;
        let mut last_relevant: Option<usize> = None;
        let mut past = false;
        loop {
            if bitreader.bits_left < HUFFMAN_CODE_MAXLEN {
                bitreader.refill();
            }
            let peeked_bits = bitreader.peek(HUFFMAN_CODE_MAXLEN);
            // SAFETY: see parse_paths.
            let (symbol, code_len) = unsafe { *self.huffman_lookup_table.get_unchecked(peeked_bits as usize) };
            bitreader.consume(code_len as u32);
            if symbol == STOP_READING_SYMBOL {
                break;
            }
            do_op(symbol, bitreader, &mut fp)?;
            if !past {
                if bound.is_past(&fp) {
                    if last_relevant.is_none() {
                        return Ok((None, true));
                    }
                    past = true;
                } else {
                    last_relevant = Some(idx);
                }
            }
            self.write_fp(&mut fp, idx)?;
            idx += 1;
        }
        Ok((last_relevant, idx > 0))
    }
    pub fn parse_paths(&mut self, bitreader: &mut Bitreader) -> Result<usize, DemoParserError> {
        /*
        Create a field path by decoding using a Huffman tree.
        The huffman tree can be found at the bottom of entities_utils.rs

        A field path is a "path trough a struct" where
        the struct can have normal fields but also pointers
        to other (nested) structs.

        Example:

        The array will be filled with these:

        Struct Field{
            wanted_information: Option<T>,
            Pointer: bool,
            fields: Option<Vec<Field>>
        },

        (struct is simplified for this example. In reality it also includes field name etc.)


        Path to each of the fields in the below fields list: [
            [0], [1, 0], [1, 1], [2]
        ]
        and they would map to:
        [0] => FloatDecoder,
        [1, 0] => IntegerDecoder,
        [1, 1] => StringDecoder,
        [2] => VectorDecoder,

        fields = [
            Field{
                wanted_information: FloatDecoder,
                pointer: false,
                fields: None,
            },
            Field{
                wanted_information: None,
                pointer: true,
                fields: Some(
                    [
                        Field{
                            wanted_information: IntegerDecoder,
                            pointer: false,
                            fields: Some(
                        },
                        Field{
                            wanted_information: StringDecoder,
                            pointer: flase,
                            fields: Some(
                        }
                    ]
                ),
            },
            Field{
                wanted_information: VectorDecoder,
                pointer: false,
                fields: None,
            },
        ]
        Not sure what the maximum depth of these structs are, but others seem to use
        7 as the max length of field path so maybe that?

        Personally I find this path idea horribly complicated. Why is this chosen over
        the way it was done in source 1 demos?
        */

        // Create an "empty" path ([-1, 0, 0, 0, 0, 0, 0])
        // For performance reasons have them always the same len
        let mut fp = generate_fp();
        let mut idx = 0;
        // Do huffman decoding with a lookup table instead of reading one bit at a time
        // and traversing a tree.
        // Here we peek ("HUFFMAN_CODE_MAXLEN" == 17) amount of bits and see from a table which
        // symbol it maps to and how many bits should be consumed from the stream.
        // The symbol is then mapped into an op for filling the field path.
        loop {
            if bitreader.bits_left < HUFFMAN_CODE_MAXLEN {
                bitreader.refill();
            }

            let peeked_bits = bitreader.peek(HUFFMAN_CODE_MAXLEN);
            // SAFETY: peek(17) yields a value in [0, 2^17-1] (it masks with (1<<17)-1), and the
            // huffman table is built with exactly 2^17 entries (huf.b = 131071 pairs + 1 sentinel,
            // see create_huffman_lookup_table). So `peeked_bits` is always a valid index. Eliding
            // the bounds check removes a per-symbol branch in the hottest decode loop.
            let (symbol, code_len) = unsafe { *self.huffman_lookup_table.get_unchecked(peeked_bits as usize) };
            bitreader.consume(code_len as u32);
            if symbol == STOP_READING_SYMBOL {
                break;
            }
            do_op(symbol, bitreader, &mut fp)?;
            self.write_fp(&mut fp, idx)?;
            idx += 1;
        }
        Ok(idx)
    }

    pub fn decode_entity_update(
        &mut self,
        bitreader: &mut Bitreader,
        entity_id: i32,
        n_updates: usize,
        is_fullpacket: bool,
        is_baseline: bool,
        events_to_emit: &mut Vec<GameEventInfo>,
    ) -> Result<usize, DemoParserError> {
        let entity = match self.entities.get_mut(entity_id as usize) {
            Some(Some(entity)) => entity,
            _ => return Err(DemoParserError::EntityNotFound),
        };
        let class = match self.cls_by_id.get(entity.cls_id as usize) {
            Some(cls) => cls,
            None => return Err(DemoParserError::ClassNotFound),
        };

        for path in self.paths.iter().take(n_updates) {
            let field = find_field(&path, &class.serializer)?;
            let field_info = get_propinfo(&field, path);
            let decoder = get_decoder_from_field(field)?;
            let result = bitreader.decode(&decoder, self.qf_mapper)?;

            // listen_to_props()
            if self.list_props {
                if let Field::Value(_v) = field {
                    if should_emit_prop_to_listen(&_v.full_name) {
                        self.uniq_prop_names.insert(convert_weapon_prefix_to_general(&_v.full_name));
                    }
                }
            }
            // Custom events
            if !is_baseline {
                SecondPassParser::listen_for_events(
                    entity,
                    &result,
                    field,
                    field_info,
                    &self.prop_controller,
                    &self.prop_controller.special_ids,
                    is_fullpacket,
                    events_to_emit,
                );
            }
            // Debug
            if self.is_debug_mode {
                SecondPassParser::debug_inspect(
                    &result,
                    field,
                    self.tick,
                    field_info,
                    path,
                    is_fullpacket,
                    is_baseline,
                    class,
                    &entity.cls_id,
                    &entity_id,
                );
            }
            SecondPassParser::insert_field(entity, result, field_info);
        }
        Ok(n_updates)
    }

    pub fn debug_inspect(
        _result: &Variant,
        field: &Field,
        _tick: i32,
        field_info: Option<FieldInfo>,
        _path: &FieldPath,
        _is_fullpacket: bool,
        _is_baseline: bool,
        _cls: &Class,
        _cls_id: &u32,
        _entity_id: &i32,
    ) {
        if let Field::Value(_v) = field {
            println!("{:?} {:?} {:?} {:?} {:?}", _path, field_info, _v.full_name, _result, _cls.name);
        }
    }

    pub fn insert_field(entity: &mut Entity, result: Variant, field_info: Option<FieldInfo>) {
        let fi = match field_info {
            Some(fi) if fi.should_parse => fi,
            _ => return,
        };
        if fi.vec_size {
            // Element count of a collected vector: shrink (elements past the count are stale)
            // or make room. Elements arrive as separate updates below.
            let n = match result {
                Variant::U32(n) => n as usize,
                _ => return,
            };
            match entity.props.get_mut(&fi.prop_id) {
                Some(Variant::List(list)) => list.truncate(n),
                Some(Variant::Bytes(bytes)) => bytes.truncate(n),
                _ => {
                    entity.props.insert(
                        fi.prop_id,
                        if fi.vec_bytes {
                            Variant::Bytes(Vec::with_capacity(n))
                        } else {
                            Variant::List(Vec::with_capacity(n))
                        },
                    );
                }
            }
            return;
        }
        if let Some(idx) = fi.vec_index {
            let idx = idx as usize;
            // Arrays are fixed size (e.g. Vector[64]); vectors are bounded by their count.
            if idx > 65536 {
                return;
            }
            if fi.vec_bytes {
                let byte = match result {
                    Variant::U32(b) => b as u8,
                    Variant::I32(b) => b as u8,
                    _ => return,
                };
                let entry = entity.props.entry(fi.prop_id).or_insert_with(|| Variant::Bytes(vec![]));
                if !matches!(entry, Variant::Bytes(_)) {
                    *entry = Variant::Bytes(vec![]);
                }
                if let Variant::Bytes(bytes) = entry {
                    if bytes.len() <= idx {
                        bytes.resize(idx + 1, 0);
                    }
                    bytes[idx] = byte;
                }
                return;
            }
            let entry = entity.props.entry(fi.prop_id).or_insert_with(|| Variant::List(vec![]));
            if !matches!(entry, Variant::List(_)) {
                *entry = Variant::List(vec![]);
            }
            if let Variant::List(list) = entry {
                if list.len() <= idx {
                    let filler = zero_like(&result);
                    list.resize(idx + 1, filler);
                }
                list[idx] = result;
            }
            return;
        }
        entity.props.insert(fi.prop_id, result);
    }

    #[inline]
    fn write_fp(&mut self, fp_src: &mut FieldPath, idx: usize) -> Result<(), DemoParserError> {
        match self.paths.get_mut(idx) {
            Some(entry) => *entry = *fp_src,
            // need to extend vec (rare)
            None => {
                // If we have over 100k fields for an entity then something definitely went wrong. Do this to avoid infinite loop/oom
                if idx > 100_000 {
                    return Err(DemoParserError::VectorResizeFailure);
                }
                self.paths.resize(idx + 1, generate_fp());
                match self.paths.get_mut(idx) {
                    Some(entry) => *entry = *fp_src,
                    None => return Err(DemoParserError::VectorResizeFailure),
                }
            }
        }
        Ok(())
    }
    fn create_new_entity(&mut self, bitreader: &mut Bitreader, entity_id: &i32, _events_to_emit: &mut Vec<GameEventInfo>) -> Result<(), DemoParserError> {
        // Class id width is dynamic: ceil(log2(num_classes + 1)). Hardcoded 8 bits
        // capped at 256 classes and broke on patches with more (14154+), causing
        // bitstream desync and cascading EntityNotFound errors. cls_by_id.len()
        // already equals num_classes + 1 (see first_pass::parser::parse_class_info).
        let cls_bits = (self.cls_by_id.len() as f32).log2().ceil() as u32;
        let cls_id: u32 = bitreader.read_nbits(cls_bits)?;
        // Both of these are not used. Don't think they are interesting for the parser
        let _serial = bitreader.read_nbits(NSERIALBITS)?;
        let _unknown = bitreader.read_varint();
        let entity_type = self.check_entity_type(&cls_id)?;
        if self.parse_weapons && self.is_weapon_class(&cls_id) {
            self.weapons.insert(*entity_id);
        }
        match entity_type {
            EntityType::Projectile => {
                self.projectiles.insert(*entity_id);
            }
            EntityType::Rules => self.rules_entity_id = Some(*entity_id),
            EntityType::C4 => self.c4_entity_id = Some(*entity_id),
            _ => {}
        };
        let entity = Entity {
            entity_id: *entity_id,
            cls_id,
            props: AHashMap::with_capacity(0),
            entity_type,
        };
        if self.entities.len() as i32 <= *entity_id {
            // if corrupt, this can cause oom allocations
            if *entity_id > 100000 {
                return Err(DemoParserError::EntityNotFound);
            }
            self.entities.resize(*entity_id as usize + 1, None);
        }
        match self.entities.get_mut(*entity_id as usize) {
            Some(entry) => *entry = Some(entity),
            None => return Err(DemoParserError::VectorResizeFailure),
        };
        // Insert baselines. The map is taken out of self for the decode (instead of cloning the
        // baseline bytes for every created entity) and put back afterwards.
        let baselines = std::mem::take(&mut self.baselines);
        let result = match baselines.get(&cls_id) {
            Some(baseline_bytes) if self.needed_mode => self.apply_baseline_needed(baseline_bytes, *entity_id, cls_id as usize),
            Some(baseline_bytes) => {
                let mut br = Bitreader::new(baseline_bytes);
                self.update_entity(&mut br, *entity_id, true, &mut vec![], false)
            }
            None => Ok(()),
        };
        self.baselines = baselines;
        result
    }

    /// Weapon entities: guns, knives, the c4 and grenades held as items. Projectiles in
    /// flight, the planted bomb and fire (CInferno) are not weapons.
    pub fn is_weapon_class(&self, cls_id: &u32) -> bool {
        let class = match self.cls_by_id.get(*cls_id as usize) {
            Some(cls) => cls,
            None => return false,
        };
        let name = class.name.as_str();
        is_grenade_or_weapon(name) && !name.contains("Projectile") && !name.contains("Planted") && !name.contains("Infer")
    }
    pub fn check_entity_type(&self, cls_id: &u32) -> Result<EntityType, DemoParserError> {
        let class = match self.cls_by_id.get(*cls_id as usize) {
            Some(cls) => cls,
            None => {
                return Err(DemoParserError::ClassNotFound);
            }
        };
        match class.name.as_str() {
            "CCSPlayerController" => return Ok(EntityType::PlayerController),
            "CCSGameRulesProxy" => return Ok(EntityType::Rules),
            "CCSTeam" => return Ok(EntityType::Team),
            "CC4" => return Ok(EntityType::C4),
            // Fire of a molotov/incendiary: lives on after the projectile, carries
            // m_firePositions/m_bFireIsBurning. Reported by parse_grenades like a projectile.
            "CInferno" => return Ok(EntityType::Projectile),
            _ => {}
        }
        let is_projectile_prop =
            (class.name.contains("Projectile") || class.name.contains("Grenade") || class.name.contains("Flash")) && !class.name.contains("Player");
        if is_projectile_prop {
            return Ok(EntityType::Projectile);
        }
        return Ok(EntityType::Normal);
    }
}

/// Filler for list slots that have not been updated yet (same kind as the element).
fn zero_like(v: &Variant) -> Variant {
    match v {
        Variant::Bool(_) => Variant::Bool(false),
        Variant::U32(_) => Variant::U32(0),
        Variant::I32(_) => Variant::I32(0),
        Variant::F32(_) => Variant::F32(0.0),
        Variant::U64(_) => Variant::U64(0),
        Variant::String(_) => Variant::String(String::new()),
        Variant::VecXY(_) => Variant::VecXY([0.0; 2]),
        Variant::VecXYZ(_) => Variant::VecXYZ([0.0; 3]),
        _ => Variant::U32(0),
    }
}

fn should_emit_prop_to_listen(prop_name: &str) -> bool {
    match prop_name.split(".").next() {
        Some("CCSGameRulesProxy") => return true,
        Some("CCSTeam") => return true,
        Some("CCSPlayerPawn") => return true,
        Some("CCSPlayerController") => return true,
        _ => {}
    };
    if is_weapon_prop(prop_name) || is_grenade_prop(prop_name) {
        return true;
    }
    false
}
fn convert_weapon_prefix_to_general(full_name: &str) -> String {
    let split_at_dot: Vec<&str> = full_name.split(".").collect();
    let grenade_or_weapon = is_grenade_or_weapon(full_name);
    // Strip first part of name from grenades and weapons.
    // if weapon prop: CAK47.m_iClip1 => m_iClip1
    // if grenade: CSmokeGrenadeProjectile.CBodyComponentBaseAnimGraph.m_cellX => CBodyComponentBaseAnimGraph.m_cellX
    if is_grenade_prop(full_name) {
        return "Grenade.".to_owned() + &split_at_dot[1..].join(".");
    }
    match grenade_or_weapon {
        true => "Weapon.".to_owned() + &split_at_dot[1..].join("."),
        false => full_name.to_string(),
    }
}
fn is_weapon_prop(full_name: &str) -> bool {
    let split_at_dot: Vec<&str> = full_name.split(".").collect();
    let is_weapon_prop =
        (split_at_dot[0].contains("Weapon") || split_at_dot[0].contains("AK")) && !split_at_dot[0].contains("Player") || split_at_dot[0].contains("CDEagle");
    is_weapon_prop
}
fn is_grenade_prop(full_name: &str) -> bool {
    if full_name.contains("CCSPlayer") {
        return false;
    }
    let parts = vec!["Molo", "Inc", "Infer", "Projectile", "Grenade", "Flash"];
    for part in parts {
        if full_name.contains(part) {
            return true;
        }
    }
    false
}
