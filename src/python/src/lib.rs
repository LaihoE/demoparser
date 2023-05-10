use std::fs;

use crate::arrow::array::*;
use ahash::HashMap;
use arrow::ffi;
use itertools::Itertools;
use parser::game_events::EventField;
use parser::game_events::GameEvent;
use parser::parser_settings::Parser;
use parser::parser_settings::ParserInputs;
use parser::read_bits::DemoParserError;
use parser::variants::VarVec;
use parser::variants::Variant;
use phf_macros::phf_map;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::export::arrow;
use polars_arrow::prelude::ArrayRef;
use pyo3::exceptions::PyIndexError;
use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::Python;
use pyo3::{PyAny, PyObject, PyResult};

#[pymethods]
impl DemoParser {
    #[new]
    pub fn py_new(demo_path: String) -> PyResult<Self> {
        let bytes = match fs::read(&demo_path) {
            Ok(bytes) => bytes,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        Ok(DemoParser {
            path: demo_path,
            bytes: bytes,
        })
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
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };
        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        Ok(parser.header.to_object(py))
    }
    /// Returns a dictionary with console vars set. This includes data
    /// like this: "mp_roundtime": "1.92", "mp_buytime": "20" ...
    pub fn parse_convars(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };
        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        Ok(parser.convars.to_object(py))
    }
    /// Returns the names and frequencies of game events during the game.
    ///
    /// Example: {"player_death": 43, "bomb_planted": 4 ...}
    pub fn list_game_events(&self, _py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        // Sort by freq
        let mut v: Vec<_> = parser.game_events_counter.iter().collect();
        v.sort_by(|x, y| x.1.cmp(&y.1));
        let h = HashMap::from_iter(v);
        let dict = pyo3::Python::with_gil(|py| h.to_object(py));
        Ok(dict)
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
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: None,
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: true,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        // SoA form
        let xs = arr_to_py(Box::new(Float32Array::from(parser.projectile_records.x))).unwrap();
        let ys = arr_to_py(Box::new(Float32Array::from(parser.projectile_records.y))).unwrap();
        // Actually not sure about Z coordinate. Leave out for now.
        //let zs = arr_to_py(Box::new(Float32Array::from(parser.projectile_records.z))).unwrap();
        let ticks = arr_to_py(Box::new(Int32Array::from(parser.projectile_records.tick))).unwrap();

        let grenade_type = arr_to_py(Box::new(Utf8Array::<i32>::from(
            parser.projectile_records.grenade_type,
        )))
        .unwrap();
        let name = arr_to_py(Box::new(Utf8Array::<i32>::from(
            parser.projectile_records.name,
        )))
        .unwrap();

        let steamids = arr_to_py(Box::new(UInt64Array::from(
            parser.projectile_records.steamid,
        )))
        .unwrap();

        let polars = py.import("polars")?;
        let all_series_py = [xs, ys, ticks, steamids, name, grenade_type].to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = ["X", "Y", "tick", "thrower_steamid", "name", "grenade_type"];
            df.setattr("columns", column_names.to_object(py)).unwrap();
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict(py);
            let pandas_df = df.call_method("to_pandas", (), Some(kwargs)).unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
    /// returns a DF with chat messages
    ///
    /// Example output:
    ///   entid           name     message  param3 param4
    /// 0     8        person1       asdfa
    /// 1     8        person2        asdf  TSpawn
    pub fn parse_chat_messages(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };

        // SoA form
        let entids =
            arr_to_py(Box::new(Int32Array::from(parser.chat_messages.entity_idx))).unwrap();
        let param1 =
            rust_series_to_py_series(&Series::new("param1", parser.chat_messages.param1)).unwrap();
        let param2 =
            rust_series_to_py_series(&Series::new("param2", parser.chat_messages.param2)).unwrap();
        let param3 =
            rust_series_to_py_series(&Series::new("param3", parser.chat_messages.param3)).unwrap();
        let param4 =
            rust_series_to_py_series(&Series::new("param4", parser.chat_messages.param4)).unwrap();

        let polars = py.import("polars")?;
        let all_series_py = [entids, param1, param2, param3, param4].to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = ["entid", "name", "message", "param3", "param4"];
            df.setattr("columns", column_names.to_object(py)).unwrap();
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict(py);
            let pandas_df = df.call_method("to_pandas", (), Some(kwargs)).unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
    pub fn parse_player_info(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        // SoA form
        let steamid =
            rust_series_to_py_series(&Series::new("param1", parser.player_end_data.steamid))
                .unwrap();
        let team_number = arr_to_py(Box::new(Int32Array::from(
            parser.player_end_data.team_number,
        )))
        .unwrap();
        let name =
            rust_series_to_py_series(&Series::new("param2", parser.player_end_data.name)).unwrap();

        let polars = py.import("polars")?;
        let all_series_py = [steamid, name, team_number].to_object(py);
        Python::with_gil(|py| {
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            // Set column names
            let column_names = ["steamid", "name", "team_number"];
            df.setattr("columns", column_names.to_object(py)).unwrap();
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict(py);
            let pandas_df = df.call_method("to_pandas", (), Some(kwargs)).unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
    pub fn parse_item_drops(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        // SoA form
        let account_id =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.account_id))).unwrap();
        let def_index =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.def_index))).unwrap();
        let dropreason =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.dropreason))).unwrap();
        let inventory =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.inventory))).unwrap();
        let item_id = arr_to_py(Box::new(UInt64Array::from(parser.item_drops.item_id))).unwrap();
        let paint_index =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.paint_index))).unwrap();
        let paint_seed =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.paint_seed))).unwrap();
        let paint_wear =
            arr_to_py(Box::new(UInt32Array::from(parser.item_drops.paint_wear))).unwrap();
        let custom_name =
            rust_series_to_py_series(&Series::new("custom_name", parser.item_drops.custom_name))
                .unwrap();

        let polars = py.import("polars")?;
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
            df.setattr("columns", column_names.to_object(py)).unwrap();
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict(py);
            let pandas_df = df.call_method("to_pandas", (), Some(kwargs)).unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
    pub fn parse_skins(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: vec![],
            wanted_player_props_og_names: vec![],
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: Some("-".to_owned()),
            parse_ents: false,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };

        // Projectile records are in SoA form
        let def_index = arr_to_py(Box::new(UInt32Array::from(parser.skins.def_index))).unwrap();
        let item_id = arr_to_py(Box::new(UInt64Array::from(parser.skins.item_id))).unwrap();
        let paint_index = arr_to_py(Box::new(UInt32Array::from(parser.skins.paint_index))).unwrap();
        let paint_seed = arr_to_py(Box::new(UInt32Array::from(parser.skins.paint_seed))).unwrap();
        let paint_wear = arr_to_py(Box::new(UInt32Array::from(parser.skins.paint_wear))).unwrap();
        let steamid = arr_to_py(Box::new(UInt64Array::from(parser.skins.steamid))).unwrap();

        let custom_name =
            rust_series_to_py_series(&Series::new("custom_name", parser.skins.custom_name))
                .unwrap();

        let polars = py.import("polars")?;
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
            df.setattr("columns", column_names.to_object(py)).unwrap();
            // Call to_pandas with use_pyarrow_extension_array = true
            let kwargs = vec![("use_pyarrow_extension_array", true)].into_py_dict(py);
            let pandas_df = df.call_method("to_pandas", (), Some(kwargs)).unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
    #[args(py_kwargs = "**")]
    pub fn parse_events(
        &self,
        _py: Python<'_>,
        event_name: Option<String>,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let (wanted_player_props, wanted_other_props) = parse_kwargs_event(py_kwargs);
        let real_player_props = rm_user_friendly_names(&wanted_player_props);
        let real_other_props = rm_user_friendly_names(&wanted_other_props);

        let mut real_player_props = match real_player_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        let mut real_other_props = match real_other_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };

        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: real_player_props,
            wanted_player_props_og_names: wanted_player_props,
            wanted_event: event_name.clone(),
            wanted_other_props: real_other_props,
            wanted_other_props_og_names: wanted_other_props,
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        let event_series = match series_from_events(&parser.game_events) {
            Ok(ser) => ser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        let column_names: Vec<&str> = event_series.iter().map(|x| x.name().clone()).collect();
        let mut rows = 0;

        let mut all_series = vec![];

        for ser in &event_series {
            rows = ser.len().max(rows);
            let py_series = rust_series_to_py_series(&ser).unwrap();
            all_series.push(py_series);
        }
        if rows == 0 {
            return Err(PyIndexError::new_err(format!(
                "No {:?} events found!",
                event_name.unwrap()
            )));
        }
        Python::with_gil(|py| {
            let polars = py.import("polars").unwrap();
            let df = polars.call_method1("DataFrame", (all_series,)).unwrap();
            df.setattr("columns", column_names.to_object(py)).unwrap();
            let pandas_df = df.call_method0("to_pandas").unwrap();
            Ok(pandas_df.to_object(py))
        })
    }

    #[args(py_kwargs = "**")]
    pub fn parse_ticks(
        &self,
        _py: Python,
        mut wanted_props: Vec<String>,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let (_, wanted_ticks) = parse_kwargs_ticks(py_kwargs);
        let real_props = rm_user_friendly_names(&wanted_props);

        let mut real_props = match real_props {
            Ok(real_props) => real_props,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };

        let settings = ParserInputs {
            bytes: &self.bytes,
            wanted_player_props: real_props.clone(),
            wanted_player_props_og_names: wanted_props.clone(),
            wanted_other_props: vec![],
            wanted_other_props_og_names: vec![],
            wanted_event: None,
            parse_ents: true,
            wanted_ticks: vec![],
            parse_projectiles: false,
            only_header: true,
            count_props: false,
            only_convars: false,
        };

        let mut parser = match Parser::new(settings) {
            Ok(parser) => parser,
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        match parser.start() {
            Ok(_) => {}
            Err(e) => return Err(PyValueError::new_err(format!("{}", e))),
        };
        let mut all_series = vec![];

        real_props.push("tick".to_owned());
        real_props.push("steamid".to_owned());
        real_props.push("name".to_owned());
        wanted_props.push("tick".to_owned());
        wanted_props.push("steamid".to_owned());
        wanted_props.push("name".to_owned());

        for prop_name in &real_props {
            if parser.output.contains_key(prop_name) {
                match &parser.output[prop_name].data {
                    Some(VarVec::F32(data)) => {
                        all_series.push(arr_to_py(Box::new(Float32Array::from(data))).unwrap());
                    }
                    Some(VarVec::I32(data)) => {
                        all_series.push(arr_to_py(Box::new(Int32Array::from(data))).unwrap());
                    }
                    Some(VarVec::U64(data)) => {
                        all_series.push(arr_to_py(Box::new(UInt64Array::from(data))).unwrap());
                    }
                    Some(VarVec::U32(data)) => {
                        all_series.push(arr_to_py(Box::new(UInt32Array::from(data))).unwrap());
                    }
                    Some(VarVec::Bool(data)) => {
                        all_series.push(arr_to_py(Box::new(BooleanArray::from(data))).unwrap());
                    }
                    Some(VarVec::String(data)) => {
                        let s = Series::new(prop_name, data);
                        let py_series = rust_series_to_py_series(&s).unwrap();
                        all_series.push(py_series);
                    }
                    _ => {}
                }
            }
        }
        Python::with_gil(|py| {
            let polars = py.import("polars")?;
            let all_series_py = all_series.to_object(py);
            let df = polars.call_method1("DataFrame", (all_series_py,))?;
            df.setattr("columns", wanted_props.to_object(py)).unwrap();
            let pandas_df = df.call_method0("to_pandas").unwrap();
            Ok(pandas_df.to_object(py))
        })
    }
}
/// https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs
pub(crate) fn to_py_array(py: Python, pyarrow: &PyModule, array: ArrayRef) -> PyResult<PyObject> {
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
/// https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs
pub fn rust_series_to_py_series(series: &Series) -> PyResult<PyObject> {
    let series = series.rechunk();
    let array = series.to_arrow(0);
    let gil = Python::acquire_gil();
    let py = gil.python();
    let pyarrow = py.import("pyarrow")?;
    let pyarrow_array = to_py_array(py, pyarrow, array)?;
    let polars = py.import("polars")?;
    let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
    Ok(out.to_object(py))
}
/// https://github.com/pola-rs/polars/blob/master/examples/python_rust_compiled_function/src/ffi.rs
pub fn arr_to_py(array: Box<dyn Array>) -> PyResult<PyObject> {
    //let series = series.rechunk();
    //let array = series.to_arrow(0);
    let gil = Python::acquire_gil();
    let py = gil.python();
    let pyarrow = py.import("pyarrow")?;
    let pyarrow_array = to_py_array(py, pyarrow, array)?;
    let polars = py.import("polars")?;
    let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
    Ok(out.to_object(py))
}
#[pyclass]
struct DemoParser {
    path: String,
    bytes: Vec<u8>,
}

pub fn parse_kwargs_ticks(kwargs: Option<&PyDict>) -> (Vec<u64>, Vec<i32>) {
    match kwargs {
        Some(k) => {
            let mut players: Vec<u64> = vec![];
            let mut ticks: Vec<i32> = vec![];
            match k.get_item("players") {
                Some(p) => {
                    players = p.extract().unwrap();
                }
                None => {}
            }
            match k.get_item("ticks") {
                Some(t) => {
                    ticks = t.extract().unwrap();
                }
                None => {}
            }
            (players, ticks)
        }
        None => (vec![], vec![]),
    }
}
pub fn parse_kwargs_event(kwargs: Option<&PyDict>) -> (Vec<String>, Vec<String>) {
    match kwargs {
        Some(k) => {
            let mut player_props: Vec<String> = vec![];
            let mut other_props: Vec<String> = vec![];

            match k.get_item("player_extra") {
                Some(t) => {
                    player_props = t.extract().unwrap();
                }
                None => {}
            }
            match k.get_item("other_extra") {
                Some(t) => {
                    other_props = t.extract().unwrap();
                }
                None => {}
            }
            (player_props, other_props)
        }
        None => (vec![], vec![]),
    }
}
fn rm_user_friendly_names(names: &Vec<String>) -> Result<Vec<String>, DemoParserError> {
    let mut real_names = vec![];
    for name in names {
        match FRIENDLY_NAMES_MAPPING.get(name) {
            Some(real_name) => real_names.push(real_name.to_string()),
            None => return Err(DemoParserError::UnknownPropName(name.clone())),
        }
    }
    Ok(real_names)
}
pub fn series_from_events(events: &Vec<GameEvent>) -> Result<Vec<Series>, DemoParserError> {
    let pairs: Vec<EventField> = events.iter().map(|x| x.fields.clone()).flatten().collect();
    let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);
    let mut series = vec![];

    for (name, vals) in per_key_name {
        let s = series_from_pairs(&vals, name)?;
        series.push(s);
    }
    series.sort_by_key(|x| x.name().to_string());
    Ok(series)
}
fn to_f32_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(f) => match f {
                Variant::F32(val) => v.push(Some(*val)),
                _ => v.push(None),
            },
            None => v.push(None),
        }
    }
    Series::new(name, v)
}
fn to_i32_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(k) => match k {
                Variant::I32(val) => v.push(Some(*val)),
                _ => v.push(None),
            },
            None => v.push(None),
        }
    }
    Series::new(name, v)
}
fn to_u64_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(k) => match k {
                Variant::U64(val) => v.push(Some(*val)),
                _ => v.push(None),
            },
            None => v.push(None),
        }
    }
    Series::new(name, v)
}
fn to_string_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(k) => match k {
                Variant::String(val) => v.push(Some(val.to_owned())),
                _ => v.push(None),
            },
            None => v.push(None),
        }
    }
    Series::new(name, v)
}

fn to_bool_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(k) => match k {
                Variant::Bool(val) => v.push(Some(val.to_owned())),
                _ => v.push(None),
            },
            None => v.push(None),
        }
    }
    Series::new(name, v)
}
fn to_u8_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    let mut v = vec![];
    for pair in pairs {
        match &pair.data {
            Some(k) => match k {
                Variant::I32(val) => v.push(Some(*val as u32)),
                _ => v.push(None),
            },
            None => v.push(None),
        }
    }
    Series::new(name, v)
}
fn to_null_series(pairs: &Vec<&EventField>, name: &String) -> Series {
    // All series are null can pick any type
    let mut v: Vec<Option<i32>> = vec![];
    for _ in pairs {
        v.push(None);
    }
    Series::new(name, v)
}

pub fn series_from_pairs(
    pairs: &Vec<&EventField>,
    name: &String,
) -> Result<Series, DemoParserError> {
    let field_type = find_type_of_vals(pairs)?;

    let s = match field_type {
        None => to_null_series(pairs, name),
        Some(Variant::Bool(_)) => to_bool_series(pairs, name),
        Some(Variant::F32(_)) => to_f32_series(pairs, name),
        Some(Variant::I32(_)) => to_i32_series(pairs, name),
        Some(Variant::U64(_)) => to_u64_series(pairs, name),
        Some(Variant::String(_)) => to_string_series(pairs, name),
        _ => panic!("unkown ge key: {:?}", field_type),
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
            None => None,
            _ => {
                return Err(DemoParserError::UnknownGameEventVariant(
                    pair.name.clone(),
                    pair.data.clone(),
                ));
            }
        });
    }

    for t in &all_types {
        if t.is_some() {
            return Ok(t.clone());
        }
    }
    return Ok(None);
}

#[pymodule]
fn demoparser2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DemoParser>()?;
    Ok(())
}

pub static FRIENDLY_NAMES_MAPPING: phf::Map<&'static str, &'static str> = phf_map! {
    "team_surrendered" => "CCSTeam.m_bSurrendered",
    "team_rounds_total" => "CCSTeam.m_iScore",
    "team_name" => "CCSTeam.m_szTeamname",
    "team_score_overtime" => "CCSTeam.m_scoreOvertime",
    "team_match_stat"=>"CCSTeam.m_szTeamMatchStat",
    "team_num_map_victories"=>"CCSTeam.m_numMapVictories",
    "team_score_first_half"=>"CCSTeam.m_scoreFirstHalf",
    "team_score_second_half"=>"CCSTeam.m_scoreSecondHalf",
    "team_clan_name" =>"CCSTeam.m_szClanTeamname",
    "is_freeze_period"=>"CCSGameRulesProxy.CCSGameRules.m_bFreezePeriod",
    "is_warmup_period"=>"CCSGameRulesProxy.CCSGameRules.m_bWarmupPeriod" ,
    "warmup_period_end"=>"CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodEnd" ,
    "warmup_period_start"=>"CCSGameRulesProxy.CCSGameRules.m_fWarmupPeriodStart" ,
    "is_terrorist_timeout"=>"CCSGameRulesProxy.CCSGameRules.m_bTerroristTimeOutActive" ,
    "is_ct_timeout"=>"CCSGameRulesProxy.CCSGameRules.m_bCTTimeOutActive" ,
    "terrorist_timeout_remaining"=>"CCSGameRulesProxy.CCSGameRules.m_flTerroristTimeOutRemaining" ,
    "ct_timeout_remaining"=>"CCSGameRulesProxy.CCSGameRules.m_flCTTimeOutRemaining" ,
    "num_terrorist_timeouts"=>"CCSGameRulesProxy.CCSGameRules.m_nTerroristTimeOuts" ,
    "num_ct_timeouts"=>"CCSGameRulesProxy.CCSGameRules.m_nCTTimeOuts" ,
    "is_technical_timeout"=>"CCSGameRulesProxy.CCSGameRules.m_bTechnicalTimeOut" ,
    "is_waiting_for_resume"=>"CCSGameRulesProxy.CCSGameRules.m_bMatchWaitingForResume" ,
    "match_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_fMatchStartTime" ,
    "round_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_fRoundStartTime" ,
    "restart_round_time"=>"CCSGameRulesProxy.CCSGameRules.m_flRestartRoundTime" ,
    "is_game_restart?"=>"CCSGameRulesProxy.CCSGameRules.m_bGameRestart" ,
    "game_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_flGameStartTime" ,
    "time_until_next_phase_start"=>"CCSGameRulesProxy.CCSGameRules.m_timeUntilNextPhaseStarts" ,
    "game_phase"=>"CCSGameRulesProxy.CCSGameRules.m_gamePhase" ,
    "total_rounds_played"=>"CCSGameRulesProxy.CCSGameRules.m_totalRoundsPlayed" ,
    "rounds_played_this_phase"=>"CCSGameRulesProxy.CCSGameRules.m_nRoundsPlayedThisPhase" ,
    "hostages_remaining"=>"CCSGameRulesProxy.CCSGameRules.m_iHostagesRemaining" ,
    "any_hostages_reached"=>"CCSGameRulesProxy.CCSGameRules.m_bAnyHostageReached" ,
    "has_bombites"=>"CCSGameRulesProxy.CCSGameRules.m_bMapHasBombTarget" ,
    "has_rescue_zone"=>"CCSGameRulesProxy.CCSGameRules.m_bMapHasRescueZone" ,
    "has_buy_zone"=>"CCSGameRulesProxy.CCSGameRules.m_bMapHasBuyZone" ,
    "is_matchmaking"=>"CCSGameRulesProxy.CCSGameRules.m_bIsQueuedMatchmaking" ,
    "match_making_mode"=>"CCSGameRulesProxy.CCSGameRules.m_nQueuedMatchmakingMode" ,
    "is_valve_dedicated_server"=>"CCSGameRulesProxy.CCSGameRules.m_bIsValveDS" ,
    "gungame_prog_weap_ct"=>"CCSGameRulesProxy.CCSGameRules.m_iNumGunGameProgressiveWeaponsCT" ,
    "gungame_prog_weap_t"=>"CCSGameRulesProxy.CCSGameRules.m_iNumGunGameProgressiveWeaponsT" ,
    "spectator_slot_count"=>"CCSGameRulesProxy.CCSGameRules.m_iSpectatorSlotCount" ,
    "is_match_started"=>"CCSGameRulesProxy.CCSGameRules.m_bHasMatchStarted" ,
    "n_best_of_maps"=>"CCSGameRulesProxy.CCSGameRules.m_numBestOfMaps" ,
    "is_bomb_dropped"=>"CCSGameRulesProxy.CCSGameRules.m_bBombDropped" ,
    "is_bomb_planed"=>"CCSGameRulesProxy.CCSGameRules.m_bBombPlanted" ,
    "round_win_status"=>"CCSGameRulesProxy.CCSGameRules.m_iRoundWinStatus" ,
    "round_win_reason"=>"CCSGameRulesProxy.CCSGameRules.m_eRoundWinReason" ,
    "terrorist_cant_buy"=>"CCSGameRulesProxy.CCSGameRules.m_bTCantBuy" ,
    "ct_cant_buy"=>"CCSGameRulesProxy.CCSGameRules.m_bCTCantBuy" ,
    "num_player_alive_ct"=>"CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_CT" ,
    "num_player_alive_t"=>"CCSGameRulesProxy.CCSGameRules.m_iMatchStats_PlayersAlive_T" ,
    "ct_losing_streak"=>"CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveCTLoses" ,
    "t_losing_streak"=>"CCSGameRulesProxy.CCSGameRules.m_iNumConsecutiveTerroristLoses" ,
    "survival_start_time"=>"CCSGameRulesProxy.CCSGameRules.m_flSurvivalStartTime" ,
    "round_in_progress"=>"CCSGameRulesProxy.CCSGameRules.m_bRoundInProgress" ,
    "i_bomb_site?"=>"CCSGameRulesProxy.CCSGameRules.m_iBombSite" ,
    "is_auto_muted"=>"CCSPlayerController.m_bHasCommunicationAbuseMute",
    "crosshair_code"=>"CCSPlayerController.m_szCrosshairCodes",
    "pending_team_num"=>"CCSPlayerController.m_iPendingTeamNum",
    "player_color"=>"CCSPlayerController.m_iCompTeammateColor",
    "ever_played_on_team"=>"CCSPlayerController.m_bEverPlayedOnTeam",
    "clan_name"=>"CCSPlayerController.m_szClan",
    "is_coach_team"=>"CCSPlayerController.m_iCoachingTeam",
    "comp_rank"=>"CCSPlayerController.m_iCompetitiveRanking",
    "comp_wins"=>"CCSPlayerController.m_iCompetitiveWins",
    "comp_rank_type"=>"CCSPlayerController.m_iCompetitiveRankType",
    "is_controlling_bot"=>"CCSPlayerController.m_bControllingBot",
    "has_controlled_bot_this_round"=>"CCSPlayerController.m_bHasControlledBotThisRound",
    "can_control_bot"=>"CCSPlayerController.m_bCanControlObservedBot",
    "is_alive"=>"CCSPlayerController.m_bPawnIsAlive",
    "armor"=>"CCSPlayerController.m_iPawnArmor",
    "has_defuser"=>"CCSPlayerController.m_bPawnHasDefuser",
    "has_helmet"=>"CCSPlayerController.m_bPawnHasHelmet",
    "spawn_time"=>"CCSPlayerController.m_iPawnLifetimeStart",
    "death_time"=>"CCSPlayerController.m_iPawnLifetimeEnd",
    "score"=>"CCSPlayerController.m_iScore",
    "game_time"=>"CCSPlayerController.m_flSimulationTime",
    "is_connected"=>"CCSPlayerController.m_iConnected",
    "player_name"=>"CCSPlayerController.m_iszPlayerName",
    "player_steamid"=>"CCSPlayerController.m_steamID",
    "fov"=>"CCSPlayerController.m_iDesiredFOV",
    "balance"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iAccount",
    "start_balance"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iStartAccount",
    "total_cash_spent"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iTotalCashSpent",
    "cash_spent_this_round"=>"CCSPlayerController.CCSPlayerController_InGameMoneyServices.m_iCashSpentThisRound",
    "music_kit_id"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_unMusicID",
    "leader_honors"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsLeader",
    "teacher_honors"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsTeacher",
    "friendly_honors"=>"CCSPlayerController.CCSPlayerController_InventoryServices.m_nPersonaDataPublicCommendsFriendly",
    "kills_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKills",
    "deaths_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDeaths",
    "assists_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iAssists",
    "alive_time_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iLiveTime",
    "headshot_kills_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iHeadShotKills",
    "damage_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iDamage",
    "objective_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iObjective",
    "utility_damage_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iUtilityDamage",
    "enemies_flashed_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEnemiesFlashed",
    "equipment_value_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iEquipmentValue",
    "money_saved_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iMoneySaved",
    "kill_reward_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iKillReward",
    "cash_earned_this_round"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.CSPerRoundStats_t.m_iCashEarned",
    "kills_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKills",
    "deaths_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDeaths",
    "assists_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iAssists",
    "alive_time_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iLiveTime",
    "headshot_kills_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iHeadShotKills",
    "ace_rounds_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy5Ks",
    "4k_rounds_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy4Ks",
    "3k_rounds_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemy3Ks",
    "damage_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iDamage",
    "objective_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iObjective",
    "utility_damage_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iUtilityDamage",
    "enemies_flashed_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEnemiesFlashed",
    "equipment_value_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iEquipmentValue",
    "money_saved_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iMoneySaved",
    "kill_reward_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iKillReward",
    "cash_earned_total"=>"CCSPlayerController.CCSPlayerController_ActionTrackingServices.m_iCashEarned",
    "ping"=>"CCSPlayerController.m_iPing",
    "move_collide" => "CCSPlayerPawn.m_MoveCollide",
    "move_type" =>  "CCSPlayerPawn.m_MoveType",
    "team_num" => "CCSPlayerPawn.m_iTeamNum",
    "active_weapon" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon",
    "ammo" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_iAmmo",
    "looking_at_weapon" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_bIsLookingAtWeapon",
    "holding_look_at_weapon" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_bIsHoldingLookAtWeapon",
    "next_attack_time" => "CCSPlayerPawn.CCSPlayer_WeaponServices.m_flNextAttack",
    "duck_time_ms" =>"CCSPlayerPawn.CCSPlayer_MovementServices.m_nDuckTimeMsecs",
    "max_speed" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxspeed",
    "max_fall_velo" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flMaxFallVelocity",
    "duck_amount" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckAmount",
    "duck_speed" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flDuckSpeed",
    "duck_overrdie" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDuckOverride",
    "old_jump_pressed" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bOldJumpPressed",
    "jump_until" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpUntil",
    "jump_velo" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flJumpVel",
    "fall_velo" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flFallVelocity",
    "in_crouch" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bInCrouch",
    "crouch_state" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_nCrouchState",
    "ducked" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDucked",
    "ducking" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bDucking",
    "in_duck_jump" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bInDuckJump",
    "allow_auto_movement" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_bAllowAutoMovement",
    "jump_time_ms" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_nJumpTimeMsecs",
    "last_duck_time" => "CCSPlayerPawn.CCSPlayer_MovementServices.m_flLastDuckTime",
    "is_rescuing" => "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_bIsRescuing",
    "weapon_purchases_this_match" => "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_iWeaponPurchasesThisMatch",
    "weapon_purchases_this_round" => "CCSPlayerPawn.CCSPlayer_ActionTrackingServices.m_iWeaponPurchasesThisRound",
    "spotted" => "CCSPlayerPawn.m_bSpotted",
    "spotted_mask" => "CCSPlayerPawn.m_bSpottedByMask",
    "time_last_injury" => "CCSPlayerPawn.m_flTimeOfLastInjury",
    "direction_last_injury" => "CCSPlayerPawn.m_nRelativeDirectionOfLastInjury",
    "player_state" => "CCSPlayerPawn.m_iPlayerState",
    "passive_items" => "CCSPlayerPawn.m_passiveItems",
    "is_scoped" => "CCSPlayerPawn.m_bIsScoped",
    "is_walking" => "CCSPlayerPawn.m_bIsWalking",
    "resume_zoom" => "CCSPlayerPawn.m_bResumeZoom",
    "is_defusing" =>"CCSPlayerPawn.m_bIsDefusing",
    "is_grabbing_hostage" => "CCSPlayerPawn.m_bIsGrabbingHostage",
    "blocking_use_in_progess" => "CCSPlayerPawn.m_iBlockingUseActionInProgress",
    "molotov_damage_time" => "CCSPlayerPawn.m_fMolotovDamageTime",
    "moved_since_spawn" => "CCSPlayerPawn.m_bHasMovedSinceSpawn",
    "in_bomb_zone" => "CCSPlayerPawn.m_bInBombZone",
    "in_buy_zone" => "CCSPlayerPawn.m_bInBuyZone",
    "in_no_defuse_area" => "CCSPlayerPawn.m_bInNoDefuseArea",
    "killed_by_taser" => "CCSPlayerPawn.m_bKilledByTaser",
    "move_state" => "CCSPlayerPawn.m_iMoveState",
    "which_bomb_zone" => "CCSPlayerPawn.m_nWhichBombZone",
    "in_hostage_rescue_zone" => "CCSPlayerPawn.m_bInHostageRescueZone",
    "stamina" => "CCSPlayerPawn.m_flStamina",
    "direction" => "CCSPlayerPawn.m_iDirection",
    "shots_fired" => "CCSPlayerPawn.m_iShotsFired",
    "armor_value" => "CCSPlayerPawn.m_ArmorValue",
    "velo_modifier" => "CCSPlayerPawn.m_flVelocityModifier",
    "ground_accel_linear_frac_last_time" => "CCSPlayerPawn.m_flGroundAccelLinearFracLastTime",
    "flash_duration" => "CCSPlayerPawn.m_flFlashDuration",
    "flash_max_alpha" => "CCSPlayerPawn.m_flFlashMaxAlpha",
    "wait_for_no_attack" => "CCSPlayerPawn.m_bWaitForNoAttack",
    "last_place_name" => "CCSPlayerPawn.m_szLastPlaceName",
    "is_strafing" => "CCSPlayerPawn.m_bStrafing",
    "round_start_equip_value" => "CCSPlayerPawn.m_unRoundStartEquipmentValue",
    "current_equip_value" => "CCSPlayerPawn.m_unCurrentEquipmentValue",
    "time" => "CCSPlayerPawn.m_flSimulationTime",
    "health" => "CCSPlayerPawn.m_iHealth",
    // "pitchyaw" => "CCSPlayerPawn.m_angEyeAngles"
    "life_state" => "CCSPlayerPawn.m_lifeState",
    "X"=> "X",
    "Y"=> "Y",
    "Z"=> "Z",
};
