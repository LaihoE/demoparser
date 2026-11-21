use super::read_bits::{Bitreader, DemoParserError};
use crate::first_pass::parser_settings::FirstPassParser;
use crate::second_pass::parser_settings::SecondPassParser;
use csgoproto::CMsgPlayerInfo;
use csgoproto::CsvcMsgCreateStringTable;
use csgoproto::CsvcMsgUpdateStringTable;
use prost::Message;
use snap::raw::Decoder;
use ahash::AHashMap;
use csgoproto::CDemoStringTables;
use std::collections::BTreeMap;

/// One string table as created by svc_CreateStringTable. Its position in the parser's table list
/// is its table id (ids are assigned in creation order, reset by svc_ClearAllStringTables).
#[derive(Clone, Debug)]
pub struct StringTable {
    name: String,
    user_data_size_bits: i32,
    user_data_fixed: bool,
    flags: i32,
    var_bit_counts: bool,
    /// Entry keys by entry index. Only filled for the tables we read (see `is_tracked`), so that
    /// later updates that change an entry's value without repeating its key can be resolved.
    keys: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct UserInfo {
    pub steamid: u64,
    pub name: String,
    pub userid: i32,
    pub is_hltv: bool,
}

/// Entries can address any index, but a real table is far below this; guards the resize.
const MAX_STRING_TABLE_ENTRIES: usize = 1 << 16;

/// The only tables whose contents we use.
fn is_tracked(name: &str) -> bool {
    name == "instancebaseline" || name == "userinfo"
}

impl StringTable {
    fn from_create(table: &CsvcMsgCreateStringTable) -> Self {
        StringTable {
            name: table.name().to_string(),
            user_data_size_bits: table.user_data_size_bits(),
            user_data_fixed: table.user_data_fixed_size(),
            flags: table.flags(),
            var_bit_counts: table.using_varint_bitcounts(),
            keys: vec![],
        }
    }

    /// Decode `n_entries` entries of a create/update message, updating the stored keys and calling
    /// `on_value(key, value)` for every entry that carries a value.
    fn decode_entries(&mut self, bytes: &[u8], n_entries: i32, mut on_value: impl FnMut(&str, &[u8])) -> Result<(), DemoParserError> {
        let mut bitreader = Bitreader::new(bytes);
        let mut idx: i64 = -1;
        // Recently decoded keys, for the LZ-style key prefix references.
        let mut history: Vec<String> = vec![];

        for _ in 0..n_entries {
            // Either the next index, or an absolute one.
            idx = match bitreader.read_boolean()? {
                true => idx + 1,
                false => bitreader.read_varint()? as i64 + 1,
            };
            if idx < 0 || idx as usize >= MAX_STRING_TABLE_ENTRIES {
                return Err(DemoParserError::MalformedMessage);
            }
            let idx = idx as usize;
            if idx >= self.keys.len() {
                self.keys.resize(idx + 1, String::new());
            }
            // Key, absent when an update only changes the value of an existing entry.
            if bitreader.read_boolean()? {
                let key = match bitreader.read_boolean()? {
                    false => bitreader.read_string()?,
                    true => {
                        // Copy `length` bytes of the key `position` entries back in history.
                        let position = bitreader.read_nbits(5)? as usize;
                        let length = bitreader.read_nbits(5)? as usize;
                        let suffix = bitreader.read_string()?;
                        match history.get(position) {
                            Some(s) => s.get(..length.min(s.len())).unwrap_or("").to_string() + &suffix,
                            None => suffix,
                        }
                    }
                };
                if history.len() >= 32 {
                    history.remove(0);
                }
                history.push(key.clone());
                self.keys[idx] = key;
            }
            // Value. The flag is present for every entry, keyed or not.
            if bitreader.read_boolean()? {
                let value = self.read_value(&mut bitreader)?;
                on_value(&self.keys[idx], &value);
            }
        }
        Ok(())
    }

    fn read_value(&self, bitreader: &mut Bitreader) -> Result<Vec<u8>, DemoParserError> {
        if self.user_data_fixed {
            let bits = self.user_data_size_bits.max(0) as u32;
            let mut value = Vec::with_capacity(bits.div_ceil(8) as usize);
            for i in 0..bits.div_ceil(8) {
                value.push(bitreader.read_nbits((bits - i * 8).min(8))? as u8);
            }
            return Ok(value);
        }
        let is_compressed = (self.flags & 0x1) != 0 && bitreader.read_boolean()?;
        let n_bytes = match self.var_bit_counts {
            true => bitreader.read_u_bit_var()?,
            false => bitreader.read_nbits(17)?,
        };
        let value = bitreader.read_n_bytes(n_bytes as usize)?;
        if is_compressed {
            return Decoder::new().decompress_vec(&value).map_err(|_| DemoParserError::MalformedMessage);
        }
        Ok(value)
    }
}

/// Apply one decoded entry of a tracked table to the parser state.
fn apply_entry(table: &str, key: &str, value: &[u8], baselines: &mut AHashMap<u32, Vec<u8>>, players: &mut BTreeMap<i32, UserInfo>) {
    if table == "instancebaseline" {
        if let Ok(cls_id) = key.parse::<u32>() {
            baselines.insert(cls_id, value.to_vec());
        }
    } else if table == "userinfo" {
        if let Ok(player) = parse_userinfo(value) {
            if player.steamid != 0 {
                players.insert(player.userid, player);
            }
        }
    }
}

/// svc_CreateStringTable: register the table under the next id and read its initial entries.
/// A table created again under the same name (the second pass replays the signon packets on
/// top of the first pass's tables) replaces the existing one, keeping its id.
fn create_string_table(
    bytes: &[u8],
    tables: &mut Vec<StringTable>,
    baselines: &mut AHashMap<u32, Vec<u8>>,
    players: &mut BTreeMap<i32, UserInfo>,
) -> Result<(), DemoParserError> {
    let msg = CsvcMsgCreateStringTable::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;
    let mut table = StringTable::from_create(&msg);
    if is_tracked(&table.name) {
        let data = match msg.data_compressed() {
            true => Decoder::new()
                .decompress_vec(msg.string_data())
                .map_err(|_| DemoParserError::MalformedMessage)?,
            false => msg.string_data().to_vec(),
        };
        let name = table.name.clone();
        table.decode_entries(&data, msg.num_entries(), |k, v| apply_entry(&name, k, v, baselines, players))?;
    }
    match tables.iter().position(|t| t.name == table.name) {
        Some(i) => tables[i] = table,
        None => tables.push(table),
    }
    Ok(())
}

/// svc_UpdateStringTable: apply changed entries to the table with that id.
fn update_string_table(
    bytes: &[u8],
    tables: &mut [StringTable],
    baselines: &mut AHashMap<u32, Vec<u8>>,
    players: &mut BTreeMap<i32, UserInfo>,
) -> Result<(), DemoParserError> {
    let msg = CsvcMsgUpdateStringTable::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;
    let Some(table) = tables.get_mut(msg.table_id() as usize) else {
        return Ok(());
    };
    if !is_tracked(&table.name) {
        return Ok(());
    }
    let name = table.name.clone();
    table.decode_entries(msg.string_data(), msg.num_changed_entries(), |k, v| apply_entry(&name, k, v, baselines, players))
}

/// A FullPacket carries a snapshot of the tables: take the tracked ones' entries from it, so
/// later updates resolve entry indices against the same state the server has.
pub fn apply_full_packet_string_tables(
    snapshot: &CDemoStringTables,
    tables: &mut [StringTable],
    baselines: &mut AHashMap<u32, Vec<u8>>,
    players: &mut BTreeMap<i32, UserInfo>,
) {
    for snap in &snapshot.tables {
        let name = snap.table_name();
        if !is_tracked(name) {
            continue;
        }
        for item in &snap.items {
            apply_entry(name, item.str(), item.data(), baselines, players);
        }
        if let Some(table) = tables.iter_mut().find(|t| t.name == name) {
            table.keys = snap.items.iter().map(|item| item.str().to_string()).collect();
        }
    }
}

impl<'a> FirstPassParser<'a> {
    pub fn parse_create_stringtable(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        create_string_table(bytes, &mut self.string_tables, &mut self.baselines, &mut self.stringtable_players)
    }
    pub fn update_string_table(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        update_string_table(bytes, &mut self.string_tables, &mut self.baselines, &mut self.stringtable_players)
    }
}

impl<'a> SecondPassParser<'a> {
    pub fn parse_create_stringtable(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        create_string_table(bytes, &mut self.string_tables, &mut self.baselines, &mut self.stringtable_players)
    }
    pub fn update_string_table(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        update_string_table(bytes, &mut self.string_tables, &mut self.baselines, &mut self.stringtable_players)
    }
}

pub fn parse_userinfo(bytes: &[u8]) -> Result<UserInfo, DemoParserError> {
    let player = CMsgPlayerInfo::decode(bytes).map_err(|_| DemoParserError::MalformedMessage)?;
    Ok(UserInfo {
        is_hltv: player.ishltv(),
        steamid: player.xuid(),
        name: player.name().to_string(),
        userid: player.userid() & 0xff,
    })
}
