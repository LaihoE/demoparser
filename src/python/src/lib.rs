use ahash::AHashMap;
use itertools::Itertools;
use memmap2::Mmap;
use parser::first_pass::parser_settings::create_mmap;
use parser::first_pass::parser_settings::rm_user_friendly_names;
use parser::first_pass::parser_settings::ParserInputs;
use parser::first_pass::read_bits::DemoParserError;
use parser::parse_demo::Parser;
use parser::second_pass::game_events::EventField;
use parser::second_pass::game_events::GameEvent;
use parser::second_pass::parser_settings::create_huffman_lookup_table;
use parser::second_pass::variants::VarVec;
use parser::second_pass::variants::Variant;
#[cfg(feature = "voice")]
use parser::second_pass::voice_data::convert_voice_data_to_wav;
use polars::prelude::ArrayRef;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::array::{
    Array, BooleanArray, Float32Array, Int32Array, UInt32Array, UInt64Array, Utf8Array,
};
use polars_arrow::ffi;
use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyBytes;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::Python;
use pyo3::{PyAny, PyObject, PyResult};
use std::sync::Arc;

use pyo3::create_exception;
create_exception!(DemoParser, Exception, pyo3::exceptions::PyException);

#[pymethods]
impl DemoParser {
    #[new]
    pub fn py_new(demo_path: String) -> PyResult<Self> {
        let mmap = match create_mmap(demo_path.clone()) {
            Ok(mmap) => mmap,
            Err(e) => return Err(Exception::new_err(format!("{e}. File name: {demo_path}"))),
        };
        let huf = create_huffman_lookup_table();
        Ok(Self { mmap, huf })
    }

    /// Parses header message (different from the first 16 bytes of the file)
    /// Should have the following fields:
    ///
    /// "addons", "server_name", "demo_file_stamp", "network_protocol",
    /// "map_name", "fullpackets_version", "allow_clientside_entities",
    /// "allow_clientside_particles", "demo_version_name", "demo_version_guid",
    /// "client_name", "game_directory"
    pub fn parse_header(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec![],
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        Ok(output
            .header
            .unwrap_or_else(AHashMap::default)
            .to_object(py))
    }
    /// Returns the names of game events present in the demo
    pub fn list_game_events(&self, _py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec!["all".to_string()],
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        let as_vec = output.game_events_counter.iter().collect_vec();
        let ge = pyo3::Python::with_gil(|py| as_vec.to_object(py));
        Ok(ge)
    }

    /// Returns all coordinates of all grenades along with info about thrower.
    ///
    /// Example:
    ///          X           Y       Z  tick  thrower_steamid grenade_type
    /// 0 -388.875  1295.46875 -5120.0   982              NaN    HeGrenade
    /// 1 -388.875  1295.46875 -5120.0   983              NaN    HeGrenade
    /// 2 -388.875  1295.46875 -5120.0   983              NaN    HeGrenade

    pub fn parse_grenades(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec![],
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: true,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };

        let entity_id: Vec<Option<i32>> = output.projectiles.iter().map(|s| s.entity_id).collect();
        let xs: Vec<Option<f32>> = output.projectiles.iter().map(|s| s.x).collect();
        let ys: Vec<Option<f32>> = output.projectiles.iter().map(|s| s.y).collect();
        let zs: Vec<Option<f32>> = output.projectiles.iter().map(|s| s.z).collect();

        let ticks: Vec<Option<i32>> = output.projectiles.iter().map(|s| s.tick).collect();
        let steamid: Vec<Option<u64>> = output.projectiles.iter().map(|s| s.steamid).collect();
        let name: Vec<Option<String>> = output.projectiles.iter().map(|s| s.name.clone()).collect();
        let grenade_type: Vec<Option<String>> = output
            .projectiles
            .iter()
            .map(|s| s.grenade_type.clone())
            .collect();

        // SoA form
        let xs = arr_to_py(Box::new(Float32Array::from(xs))).unwrap();
        let ys = arr_to_py(Box::new(Float32Array::from(ys))).unwrap();
        let zs = arr_to_py(Box::new(Float32Array::from(zs))).unwrap();
        // Actually not sure about Z coordinate. Leave out for now.
        let ticks = arr_to_py(Box::new(Int32Array::from(ticks))).unwrap();
        let grenade_type = arr_to_py(Box::new(Utf8Array::<i32>::from(grenade_type))).unwrap();
        let name = arr_to_py(Box::new(Utf8Array::<i32>::from(name))).unwrap();
        let steamids = arr_to_py(Box::new(UInt64Array::from(steamid))).unwrap();
        let entity_ids = arr_to_py(Box::new(Int32Array::from(entity_id))).unwrap();

        let polars = py.import_bound("polars")?;
        let all_series_py =
            [xs, ys, zs, ticks, steamids, name, grenade_type, entity_ids].to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = [
                "X",
                "Y",
                "Z",
                "tick",
                "thrower_steamid",
                "name",
                "grenade_type",
                "entity_id",
            ];
            df.setattr("columns", column_names.to_object(py)).unwrap();
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict_bound(py);
            let pandas_df = df.call_method("to_pandas", (), Some(&kwargs)).unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
    pub fn parse_player_info(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec![],
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        let steamids: Vec<Option<u64>> = output.player_md.iter().map(|p| p.steamid).collect();
        let team_numbers: Vec<Option<i32>> =
            output.player_md.iter().map(|p| p.team_number).collect();
        let names: Vec<Option<String>> = output.player_md.iter().map(|p| p.name.clone()).collect();

        // SoA form
        let steamid = rust_series_to_py_series(&Series::new("Steamid", steamids))?;
        let team_number = arr_to_py(Box::new(Int32Array::from(team_numbers)))?;
        let name = rust_series_to_py_series(&Series::new("param2", names))?;

        let polars = py.import_bound("polars")?;
        let all_series_py = [steamid, name, team_number].to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = ["steamid", "name", "team_number"];
            df.setattr("columns", column_names.to_object(py))?;
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict_bound(py);
            let pandas_df = df.call_method("to_pandas", (), Some(&kwargs))?;
            Ok(pandas_df.to_object(py))
        })
    }
    pub fn parse_item_drops(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec![],
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        let def_index: Vec<Option<u32>> = output.item_drops.iter().map(|x| x.def_index).collect();
        let account_id: Vec<Option<u32>> = output.item_drops.iter().map(|x| x.account_id).collect();
        let dropreason: Vec<Option<u32>> = output.item_drops.iter().map(|x| x.dropreason).collect();
        let inventory: Vec<Option<u32>> = output.item_drops.iter().map(|x| x.inventory).collect();
        let item_id: Vec<Option<u64>> = output.item_drops.iter().map(|x| x.item_id).collect();
        let paint_index: Vec<Option<u32>> =
            output.item_drops.iter().map(|x| x.paint_index).collect();
        let paint_seed: Vec<Option<u32>> = output.item_drops.iter().map(|x| x.paint_seed).collect();
        let paint_wear: Vec<Option<u32>> = output.item_drops.iter().map(|x| x.paint_wear).collect();
        let custom_name: Vec<Option<String>> = output
            .item_drops
            .iter()
            .map(|x| x.custom_name.clone())
            .collect();
        // SoA form
        let account_id = arr_to_py(Box::new(UInt32Array::from(account_id)))?;
        let def_index = arr_to_py(Box::new(UInt32Array::from(def_index)))?;
        let dropreason = arr_to_py(Box::new(UInt32Array::from(dropreason)))?;
        let inventory = arr_to_py(Box::new(UInt32Array::from(inventory)))?;
        let item_id = arr_to_py(Box::new(UInt64Array::from(item_id)))?;
        let paint_index = arr_to_py(Box::new(UInt32Array::from(paint_index)))?;
        let paint_seed = arr_to_py(Box::new(UInt32Array::from(paint_seed)))?;
        let paint_wear = arr_to_py(Box::new(UInt32Array::from(paint_wear)))?;
        let custom_name = rust_series_to_py_series(&Series::new("custom_name", custom_name))?;

        let polars = py.import_bound("polars")?;
        let all_series_py = [
            account_id,
            def_index,
            dropreason,
            inventory,
            item_id,
            paint_index,
            paint_seed,
            paint_wear,
            custom_name,
        ]
        .to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = [
                "account_id",
                "def_index",
                "dropreason",
                "inventory",
                "item_id",
                "paint_index",
                "paint_seed",
                "paint_wear",
                "custom_name",
            ];
            df.setattr("columns", column_names.to_object(py))?;
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict_bound(py);
            let pandas_df = df.call_method("to_pandas", (), Some(&kwargs))?;
            Ok(pandas_df.to_object(py))
        })
    }
    pub fn parse_skins(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            real_name_to_og_name: AHashMap::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec![],
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };

        let def_idx_vec: Vec<Option<u32>> = output.skins.iter().map(|s| s.def_index).collect();
        let item_id: Vec<Option<u64>> = output.skins.iter().map(|s| s.item_id).collect();
        let paint_index: Vec<Option<u32>> = output.skins.iter().map(|s| s.paint_index).collect();
        let paint_seed: Vec<Option<u32>> = output.skins.iter().map(|s| s.paint_seed).collect();
        let paint_wear: Vec<Option<u32>> = output.skins.iter().map(|s| s.paint_wear).collect();
        let steamid: Vec<Option<u64>> = output.skins.iter().map(|s| s.steamid).collect();
        let custom_name: Vec<Option<String>> =
            output.skins.iter().map(|s| s.custom_name.clone()).collect();

        let def_index = arr_to_py(Box::new(UInt32Array::from(def_idx_vec)))?;
        let item_id = arr_to_py(Box::new(UInt64Array::from(item_id)))?;
        let paint_index = arr_to_py(Box::new(UInt32Array::from(paint_index)))?;
        let paint_seed = arr_to_py(Box::new(UInt32Array::from(paint_seed)))?;
        let paint_wear = arr_to_py(Box::new(UInt32Array::from(paint_wear)))?;
        let steamid = arr_to_py(Box::new(UInt64Array::from(steamid)))?;
        let custom_name = rust_series_to_py_series(&Series::new("custom_name", custom_name))?;

        let polars = py.import_bound("polars")?;
        let all_series_py = [
            def_index,
            item_id,
            paint_index,
            paint_seed,
            paint_wear,
            custom_name,
            steamid,
        ]
        .to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = [
                "def_index",
                "item_id",
                "paint_index",
                "paint_seed",
                "paint_wear",
                "custom_name",
                "steamid",
            ];
            df.setattr("columns", column_names.to_object(py))?;
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict_bound(py);
            let pandas_df = df.call_method("to_pandas", (), Some(&kwargs))?;
            Ok(pandas_df.to_object(py))
        })
    }

    #[pyo3(signature = (event_name, *, player=None, other=None))]
    pub fn parse_event(
        &self,
        py: Python<'_>,
        event_name: String,
        player: Option<Vec<String>>,
        other: Option<Vec<String>>,
    ) -> PyResult<Py<PyAny>> {
        let wanted_player_props = player.unwrap_or_default();
        let wanted_other_props = other.unwrap_or_default();
        let real_player_props = rm_user_friendly_names(&wanted_player_props);
        let real_other_props = rm_user_friendly_names(&wanted_other_props);

        let real_player_props = match real_player_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{e}"))),
        };
        let real_other_props = match real_other_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{e}"))),
        };
        let mut real_name_to_og_name = AHashMap::default();
        for (real_name, user_friendly_name) in real_player_props.iter().zip(&wanted_player_props) {
            real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
        }
        for (real_name, user_friendly_name) in real_other_props.iter().zip(&wanted_other_props) {
            real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
        }

        let settings = ParserInputs {
            real_name_to_og_name,
            wanted_players: vec![],
            wanted_player_props: real_player_props,
            wanted_other_props: real_other_props,
            wanted_events: vec![event_name],
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        let event_series = match series_from_event(&output.game_events, py) {
            Ok(ser) => ser,
            Err(_e) => return Ok(PyList::empty_bound(py).into()),
        };
        Ok(event_series)
    }

    #[pyo3(signature = (event_name, *, player=None, other=None))]
    pub fn parse_events(
        &self,
        py: Python<'_>,
        event_name: Vec<String>,
        player: Option<Vec<String>>,
        other: Option<Vec<String>>,
    ) -> PyResult<Py<PyAny>> {
        let wanted_player_props = player.unwrap_or_default();
        let wanted_other_props = other.unwrap_or_default();
        let real_player_props = rm_user_friendly_names(&wanted_player_props);
        let real_other_props = rm_user_friendly_names(&wanted_other_props);

        let real_player_props = match real_player_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{e}"))),
        };
        let real_other_props = match real_other_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{e}"))),
        };
        let mut real_name_to_og_name = AHashMap::default();
        for (real_name, user_friendly_name) in real_player_props.iter().zip(&wanted_player_props) {
            real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
        }
        for (real_name, user_friendly_name) in real_other_props.iter().zip(&wanted_other_props) {
            real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
        }

        let settings = ParserInputs {
            real_name_to_og_name,
            wanted_players: vec![],
            wanted_player_props: real_player_props,
            wanted_other_props: real_other_props,
            wanted_events: event_name,
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &self.huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        let event_series = match series_from_multiple_events(&output.game_events, py) {
            Ok(ser) => ser,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        Ok(event_series)
    }
    #[cfg(feature = "voice")]
    pub fn parse_voice(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_events: vec![],
            wanted_ticks: vec![],
            real_name_to_og_name: AHashMap::default(),
            parse_ents: false,
            parse_projectiles: false,
            only_header: false,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &vec![],
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(PyValueError::new_err(format!("{e}"))),
        };
        let out = convert_voice_data_to_wav(output.voice_data).unwrap();
        let mut out_hm = AHashMap::default();
        for (steamid, bytes) in out {
            let py_bytes = PyBytes::new_bound(py, &bytes);
            out_hm.insert(steamid, py_bytes);
        }
        Ok(out_hm.to_object(py))
    }

    #[pyo3(signature = (wanted_props, *, players=None, ticks=None))]
    pub fn parse_ticks(
        &self,
        py: Python,
        wanted_props: Vec<String>,
        players: Option<Vec<u64>>,
        ticks: Option<Vec<i32>>,
    ) -> PyResult<PyObject> {
        let wanted_players = players.unwrap_or_default();
        let wanted_ticks = ticks.unwrap_or_default();
        let real_props = rm_user_friendly_names(&wanted_props);

        let real_props = match real_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };

        let arc_huf = Arc::new(&self.huf);
        let mut real_name_to_og_name = AHashMap::default();
        for (real_name, user_friendly_name) in real_props.iter().zip(&wanted_props) {
            real_name_to_og_name.insert(real_name.clone(), user_friendly_name.clone());
        }
        let settings = ParserInputs {
            real_name_to_og_name,
            wanted_players,
            wanted_player_props: real_props,
            wanted_other_props: vec![],
            wanted_events: vec![],
            parse_ents: true,
            wanted_ticks,
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
            huffman_lookup_table: &arc_huf,
            order_by_steamid: false,
        };
        let mut parser = Parser::new(settings, parser::parse_demo::ParsingMode::Normal);
        let output = match parser.parse_demo(&self.mmap) {
            Ok(output) => output,
            Err(e) => return Err(Exception::new_err(format!("{e}"))),
        };
        let mut all_series = vec![];
        let mut all_pyobjects = vec![];
        let prop_infos = output.prop_controller.prop_infos;
        let mut df_column_names_arrow = vec![];
        let mut df_column_names_py = vec![];

        for prop_info in prop_infos {
            if output.df.contains_key(&prop_info.id) {
                match &output.df[&prop_info.id].data {
                    Some(VarVec::F32(data)) => {
                        df_column_names_arrow.push(prop_info.prop_friendly_name);
                        all_series.push(arr_to_py(Box::new(Float32Array::from(data)))?);
                    }
                    Some(VarVec::I32(data)) => {
                        df_column_names_arrow.push(prop_info.prop_friendly_name);
                        all_series.push(arr_to_py(Box::new(Int32Array::from(data)))?);
                    }
                    Some(VarVec::U64(data)) => {
                        df_column_names_arrow.push(prop_info.prop_friendly_name);
                        all_series.push(arr_to_py(Box::new(UInt64Array::from(data)))?);
                    }
                    Some(VarVec::U32(data)) => {
                        df_column_names_arrow.push(prop_info.prop_friendly_name);
                        all_series.push(arr_to_py(Box::new(UInt32Array::from(data)))?);
                    }
                    Some(VarVec::Bool(data)) => {
                        df_column_names_arrow.push(prop_info.prop_friendly_name);
                        all_series.push(arr_to_py(Box::new(BooleanArray::from(data)))?);
                    }
                    Some(VarVec::String(data)) => {
                        df_column_names_arrow.push(prop_info.prop_friendly_name.clone());
                        let s = Series::new(&prop_info.prop_friendly_name.clone(), data);
                        let py_series = rust_series_to_py_series(&s)?;
                        all_series.push(py_series);
                    }
                    Some(VarVec::StringVec(data)) => {
                        df_column_names_py.push(prop_info.prop_friendly_name);
                        all_pyobjects.push(data.to_object(py));
                    }
                    Some(VarVec::U64Vec(data)) => {
                        df_column_names_py.push(prop_info.prop_friendly_name);
                        all_pyobjects.push(data.to_object(py));
                    }
                    Some(VarVec::XYZVec(data)) => {
                        df_column_names_py.push(prop_info.prop_friendly_name);
                        all_pyobjects.push(data.to_object(py));
                    }
                    Some(VarVec::U32Vec(data)) => {
                        df_column_names_py.push(prop_info.prop_friendly_name);
                        all_pyobjects.push(data.to_object(py));
                    }

                    Some(VarVec::Stickers(data)) => {
                        let mut dicts = vec![];
                        for weapon in data {
                            let mut v = vec![];
                            for sticker in weapon {
                                let dict = PyDict::new_bound(py);
                                dict.set_item("id", sticker.id.to_object(py))?;
                                dict.set_item("name", sticker.name.to_object(py))?;
                                dict.set_item("wear", sticker.wear.to_object(py))?;
                                dict.set_item("x", sticker.x.to_object(py))?;
                                dict.set_item("y", sticker.y.to_object(py))?;
                                v.push(dict);
                            }
                            dicts.push(v);
                        }
                        df_column_names_py.push(prop_info.prop_friendly_name);
                        all_pyobjects.push(dicts.to_object(py));
                    }
                    _ => {}
                }
            }
        }
        Python::with_gil(|py| {
            let polars = py.import_bound("polars")?;
            let all_series_py = all_series.to_object(py);
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            df.setattr("columns", df_column_names_arrow.to_object(py))?;
            let pandas_df = df.call_method0("to_pandas")?;
            for (pyobj, col_name) in all_pyobjects.iter().zip(&df_column_names_py) {
                pandas_df.call_method1("insert", (0, col_name, pyobj))?;
            }
            df_column_names_arrow.extend(df_column_names_py);
            df_column_names_arrow.sort();
            let kwargs = vec![("axis", 1)].into_py_dict_bound(py);
            let args = (df_column_names_arrow,);
            pandas_df.call_method("reindex", args, Some(&kwargs))?;
            Ok(pandas_df.to_object(py))
        })
    }
}

/// <https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs>
pub(crate) fn to_py_array(
    py: Python,
    pyarrow: &Bound<PyModule>,
    array: ArrayRef,
) -> PyResult<PyObject> {
    let schema = Box::new(ffi::export_field_to_c(&ArrowField::new(
        "",
        array.data_type().clone(),
        true,
    )));
    let array = Box::new(ffi::export_array_to_c(array));

    let schema_ptr: *const ffi::ArrowSchema = &*schema;
    let array_ptr: *const ffi::ArrowArray = &*array;

    let array = pyarrow.getattr("Array")?.call_method1(
        "_import_from_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    )?;

    Ok(array.to_object(py))
}

/// <https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs>
pub fn rust_series_to_py_series(series: &Series) -> PyResult<PyObject> {
    // ensure we have a single chunk
    let series = series.rechunk();
    let array = series.to_arrow(0, false);

    Python::with_gil(|py| {
        // import pyarrow
        let pyarrow = py.import_bound("pyarrow")?;

        // pyarrow array
        let pyarrow_array = to_py_array(py, &pyarrow, array)?;

        // import polars
        let polars = py.import_bound("polars")?;
        let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
        Ok(out.to_object(py))
    })
}

/// <https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs>
pub fn arr_to_py(array: Box<dyn Array>) -> PyResult<PyObject> {
    //let series = series.rechunk();
    //let array = series.to_arrow(0);
    Python::with_gil(|py| {
        let pyarrow = py.import_bound("pyarrow")?;
        let pyarrow_array = to_py_array(py, &pyarrow, array)?;
        let polars = py.import_bound("polars")?;
        let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
        Ok(out.to_object(py))
    })
}
#[pyclass]
struct DemoParser {
    mmap: Mmap,
    huf: Vec<(u8, u8)>,
}

pub fn series_from_multiple_events(
    events: &[GameEvent],
    py: Python,
) -> Result<Py<PyAny>, DemoParserError> {
    let per_ge = events.iter().into_group_map_by(|x| x.name.clone());
    let mut vv = vec![];
    for (k, v) in per_ge {
        let pairs: Vec<EventField> = v.iter().flat_map(|x| x.fields.clone()).collect();
        let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);

        let mut series_columns = vec![];
        let mut py_columns = vec![];
        let mut rows = 0;

        for (name, vals) in per_key_name {
            match column_from_pairs(&vals, name, py)? {
                DataFrameColumn::Pyany(p) => py_columns.push((p, name)),
                DataFrameColumn::Series(s) => {
                    rows = s.len().max(rows);
                    series_columns.push((s, name));
                }
            };
        }
        let mut series_col_names: Vec<String> = series_columns
            .iter()
            .map(|(_, name)| (*name).to_string())
            .collect();
        let series_columns: Vec<PyObject> = series_columns
            .iter()
            .map(|(ser, _)| rust_series_to_py_series(ser).unwrap())
            .collect();
        let py_col_names: Vec<String> = py_columns
            .iter()
            .map(|(_, name)| (*name).to_string())
            .collect();

        if rows != 0 {
            let dfp = Python::with_gil(|py| {
                let polars = py.import_bound("polars").unwrap();
                let all_series_py = series_columns.to_object(py);
                let df = polars.call_method1("DataFrame", (all_series_py,)).unwrap();
                df.setattr("columns", series_col_names.to_object(py))
                    .unwrap();
                let pandas_df = df.call_method0("to_pandas").unwrap();
                for (pyobj, col_name) in py_columns {
                    pandas_df
                        .call_method1("insert", (0, col_name, pyobj))
                        .unwrap();
                }

                series_col_names.extend(py_col_names);
                series_col_names.sort();

                let kwargs = vec![("axis", 1)].into_py_dict_bound(py);
                let args = (series_col_names,);
                let df = pandas_df
                    .call_method("reindex", args, Some(&kwargs))
                    .unwrap();
                df.to_object(py)
            });
            vv.push((k, dfp));
        }
    }
    Ok(vv.to_object(py))
}

pub enum DataFrameColumn {
    Series(Series),
    Pyany(pyo3::Py<PyAny>),
}

pub fn series_from_event(events: &[GameEvent], py: Python) -> Result<Py<PyAny>, DemoParserError> {
    let pairs: Vec<EventField> = events.iter().flat_map(|x| x.fields.clone()).collect();
    let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);

    let mut series_columns = vec![];
    let mut py_columns = vec![];
    let mut rows = 0;

    for (name, vals) in per_key_name {
        match column_from_pairs(&vals, name, py)? {
            DataFrameColumn::Pyany(p) => py_columns.push((p, name)),
            DataFrameColumn::Series(s) => {
                rows = s.len().max(rows);
                series_columns.push((s, name));
            }
        };
    }
    let mut series_col_names: Vec<String> = series_columns
        .iter()
        .map(|(_, name)| (*name).to_string())
        .collect();
    let series_columns: Vec<PyObject> = series_columns
        .iter()
        .map(|(ser, _)| rust_series_to_py_series(ser).unwrap())
        .collect();
    let py_col_names: Vec<String> = py_columns
        .iter()
        .map(|(_, name)| (*name).to_string())
        .collect();
    if rows == 0 {
        return Err(DemoParserError::NoEvents);
    }
    let dfp = Python::with_gil(|py| {
        let polars = py.import_bound("polars").unwrap();
        let all_series_py = series_columns.to_object(py);
        let df = polars.call_method1("DataFrame", (all_series_py,)).unwrap();
        df.setattr("columns", series_col_names.to_object(py))
            .unwrap();
        let pandas_df = df.call_method0("to_pandas").unwrap();
        for (pyobj, col_name) in py_columns {
            pandas_df
                .call_method1("insert", (0, col_name, pyobj))
                .unwrap();
        }
        series_col_names.extend(py_col_names);
        series_col_names.sort();
        let kwargs = vec![("axis", 1)].into_py_dict_bound(py);
        let args = (series_col_names,);
        let df = pandas_df
            .call_method("reindex", args, Some(&kwargs))
            .unwrap();
        df.to_object(py)
    });
    Ok(dfp)
}
fn to_f32_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::F32(val)) => v.push(Some(*val)),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Series(Series::new(name, v))
}
fn to_u32_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::U32(val)) => v.push(Some(*val)),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Series(Series::new(name, v))
}
fn to_i32_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::I32(val)) => v.push(Some(*val)),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Series(Series::new(name, v))
}
fn to_u64_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::U64(val)) => v.push(Some(*val)),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Series(Series::new(name, v))
}
fn to_py_string_col(pairs: &Vec<&EventField>, _name: &str, py: Python) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::StringVec(val)) => v.push(Some(val.clone())),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Pyany(v.to_object(py))
}
fn to_py_u64_col(pairs: &Vec<&EventField>, _name: &str, py: Python) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::U64Vec(val)) => v.push(Some(val.clone())),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Pyany(v.to_object(py))
}
fn to_py_u32_col(pairs: &Vec<&EventField>, _name: &str, py: Python) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::U32Vec(val)) => v.push(Some(val.clone())),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Pyany(v.to_object(py))
}
fn to_string_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::String(val)) => v.push(Some(val.to_owned())),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Series(Series::new(name, v))
}

fn to_bool_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::Bool(val)) => v.push(Some(val.to_owned())),
            _ => v.push(None),
        }
    }
    DataFrameColumn::Series(Series::new(name, v))
}
fn to_py_sticker_col(pairs: &Vec<&EventField>, _name: &str, py: Python) -> DataFrameColumn {
    let mut v: Vec<Vec<_>> = vec![];
    for pair in pairs {
        match &pair.data {
            Some(Variant::Stickers(weapon)) => {
                let mut vv = vec![];
                for sticker in weapon {
                    let dict = PyDict::new_bound(py);
                    let _ = dict.set_item("id", sticker.id.to_object(py));
                    let _ = dict.set_item("name", sticker.name.to_object(py));
                    let _ = dict.set_item("wear", sticker.wear.to_object(py));
                    let _ = dict.set_item("x", sticker.x.to_object(py));
                    let _ = dict.set_item("y", sticker.y.to_object(py));
                    vv.push(dict);
                }
                v.push(vv);
            }
            _ => v.push(vec![]),
        }
    }
    DataFrameColumn::Pyany(v.to_object(py))
}

fn to_null_series(pairs: &Vec<&EventField>, name: &str) -> DataFrameColumn {
    // All series are null can pick any type
    let mut v: Vec<Option<i32>> = vec![];
    for _ in pairs {
        v.push(None);
    }
    DataFrameColumn::Series(Series::new(name, v))
}

pub fn column_from_pairs(
    pairs: &Vec<&EventField>,
    name: &str,
    py: Python,
) -> Result<DataFrameColumn, DemoParserError> {
    let field_type = find_type_of_vals(pairs)?;

    let s = match field_type {
        None => to_null_series(pairs, name),
        Some(Variant::Bool(_)) => to_bool_series(pairs, name),
        Some(Variant::F32(_)) => to_f32_series(pairs, name),
        Some(Variant::U32(_)) => to_u32_series(pairs, name),
        Some(Variant::I32(_)) => to_i32_series(pairs, name),
        Some(Variant::U64(_)) => to_u64_series(pairs, name),
        Some(Variant::String(_)) => to_string_series(pairs, name),
        Some(Variant::StringVec(_)) => to_py_string_col(pairs, name, py),
        Some(Variant::U64Vec(_)) => to_py_u64_col(pairs, name, py),
        Some(Variant::U32Vec(_)) => to_py_u32_col(pairs, name, py),
        Some(Variant::Stickers(_)) => to_py_sticker_col(pairs, name, py),
        _ => panic!("unkown ge key: {field_type:?}"),
    };
    Ok(s)
}
fn find_type_of_vals(pairs: &Vec<&EventField>) -> Result<Option<Variant>, DemoParserError> {
    // Need to find the correct type for outgoing series,
    let mut all_types = vec![];
    for pair in pairs {
        all_types.push(match &pair.data {
            Some(Variant::Bool(v)) => Some(Variant::Bool(*v)),
            Some(Variant::I32(v)) => Some(Variant::I32(*v)),
            Some(Variant::F32(v)) => Some(Variant::F32(*v)),
            Some(Variant::String(s)) => Some(Variant::String(s.clone())),
            Some(Variant::U64(u)) => Some(Variant::U64(*u)),
            Some(Variant::U32(u)) => Some(Variant::U32(*u)),
            Some(Variant::StringVec(_u)) => Some(Variant::StringVec(vec![])),
            Some(Variant::U64Vec(_u)) => Some(Variant::U64Vec(vec![])),
            Some(Variant::U32Vec(_u)) => Some(Variant::U64Vec(vec![])),
            Some(Variant::Stickers(_u)) => Some(Variant::Stickers(vec![])),
            None => None,
            _ => {
                return Err(DemoParserError::UnknownGameEventVariant(pair.name.clone()));
            }
        });
    }
    for t in &all_types {
        if t.is_some() {
            return Ok(t.clone());
        }
    }
    Ok(None)
}

#[pymodule]
fn demoparser2(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    Ok(())
}
