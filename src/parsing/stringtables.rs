use crate::Parser;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;

use super::read_bits::Bitreader;

#[derive(Clone, Debug)]
pub struct StringTable {
    name: String,
    uds: i32,
    udfs: bool,
    data: Vec<StField>,
}
#[derive(Clone, Debug)]
pub struct StField {
    entry: String,
}

impl Parser {
    pub fn parse_create_stringtable(&mut self, table: CSVCMsg_CreateStringTable) {
        // TODO
        return;
        let bytes = table.string_data();

        let bytes = match table.data_compressed() {
            true => snap::raw::Decoder::new()
                .decompress_vec(table.string_data())
                .unwrap(),
            // slow
            false => table.string_data().to_vec(),
        };
        let mut st = StringTable {
            name: table.name().to_string(),
            udfs: table.user_data_fixed_size(),
            uds: table.user_data_size(),
            data: vec![],
        };
        for _ in 1..50000 {
            st.data.push(StField {
                entry: "".to_string(),
            })
        }
    }
    pub fn parse_string_table(&mut self) {}
}
