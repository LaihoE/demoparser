use crate::netmessage_types;
use crate::parser::demo_cmd_type_from_int;
use crate::parser_settings::create_huffman_lookup_table;
use crate::parser_settings::Parser;
use crate::parser_settings::ParserInputs;
use crate::sendtables::Serializer;
use crate::{other_netmessages::Class, parser, read_bits::DemoParserError};
use csgoproto::demo::CDemoSendTables;
use csgoproto::demo::EDemoCommands::*;
use dashmap::DashMap;
use memmap2::Mmap;
use protobuf::Message;
use snap::raw::Decoder as SnapDecoder;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct DemoSearcher {
    pub fullpacket_offsets: Vec<usize>,
    pub ptr: usize,
    pub bytes: Arc<Mmap>,
    pub tick: i32,
    pub state: State,
    pub huf: Arc<Vec<(u32, u8)>>,
    pub settings: ParserInputs,
    pub handles: Vec<JoinHandle<()>>,
}

pub struct State {
    pub serializers: Arc<DashMap<String, Serializer>>,
    pub cls_by_id: Arc<DashMap<u32, Class>>,
}

impl DemoSearcher {
    pub fn front_demo_metadata(&mut self) -> Result<(), DemoParserError> {
        self.ptr = 16;
        let mut handles = vec![];
        let mut spanwed_threads = 0;

        loop {
            let before = self.ptr;
            let cmd = self.read_varint()?;
            let tick = self.read_varint()?;
            let size = self.read_varint()?;
            self.tick = tick as i32;

            let msg_type = cmd & !64;
            let is_compressed = (cmd & 64) == 64;
            let cmd = demo_cmd_type_from_int(msg_type as i32);
            if cmd == Some(DEM_SendTables)
                || cmd == Some(DEM_ClassInfo)
                || cmd == Some(DEM_FullPacket)
                || cmd == Some(DEM_Stop)
            {
                let bytes = match is_compressed {
                    true => SnapDecoder::new()
                        .decompress_vec(self.read_n_bytes(size)?)
                        .unwrap(),
                    false => self.read_n_bytes(size)?.to_vec(),
                };

                let ok: Result<(), DemoParserError> = match demo_cmd_type_from_int(msg_type as i32)
                    .unwrap()
                {
                    DEM_SendTables => {
                        let ser_arc = self.state.serializers.clone();
                        let handle = thread::spawn(move || {
                            let tables: CDemoSendTables =
                                Message::parse_from_bytes(&bytes).unwrap();
                            DemoSearcher::parse_sendtable(tables, ser_arc).unwrap();
                        });
                        handles.push(handle);
                        Ok(())
                    }
                    DEM_ClassInfo => {
                        let ser_arc = self.state.serializers.clone();
                        let cls_by_id_arc = self.state.cls_by_id.clone();
                        let handle = thread::spawn(move || {
                            DemoSearcher::parse_class_info(&bytes, cls_by_id_arc, ser_arc).unwrap();
                        });
                        handles.push(handle);
                        Ok(())
                    }
                    DEM_FullPacket => {
                        self.fullpacket_offsets.push(before);
                        /*
                        let mut parser = Parser::new(self.settings.clone()).unwrap();
                        parser.ptr = before;
                        parser.cls_by_id = self.state.cls_by_id.clone();
                        if spanwed_threads < 10 {
                            let handle = thread::spawn(move || {
                                // DemoSearcher::parse_class_info(&bytes, cls_by_id_arc, ser_arc).unwrap();
                                parser.start().unwrap();
                            });
                            spanwed_threads += 1;

                            self.handles.push(handle);
                        }
                        */
                        Ok(())
                    }
                    DEM_Stop => {
                        println!("STOP");
                        break;
                    }
                    _ => Ok(()),
                };
                ok?;
            } else {
                self.ptr += size as usize;
            };
        }
        for hanle in handles {
            hanle.join().unwrap();
        }
        use rayon::prelude::*;

        let v: Vec<()> = self
            .fullpacket_offsets
            .par_iter()
            .map(|o| {
                let mut parser = Parser::new(self.settings.clone()).unwrap();
                parser.ptr = *o;
                parser.cls_by_id = self.state.cls_by_id.clone();
                parser.start().unwrap();
            })
            .collect();

        println!("{:?}", self.fullpacket_offsets);
        Ok(())
    }
}

use csgoproto::demo::CDemoClassInfo;

impl DemoSearcher {
    pub fn parse_class_info(
        bytes: &[u8],
        cls_by_id: Arc<DashMap<u32, Class>>,
        serializers: Arc<DashMap<String, Serializer>>,
    ) -> Result<(), DemoParserError> {
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();
        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();

            loop {
                match serializers.get(network_name) {
                    Some(ser) => {
                        println!("CLSID {} DONE", cls_id);
                        cls_by_id.insert(
                            cls_id as u32,
                            Class {
                                class_id: cls_id,
                                name: network_name.to_string(),
                                serializer: ser.clone(),
                            },
                        );
                        break;
                    }
                    None => {
                        println!("CLSID {} NOT FOUND", cls_id);
                        let ten_millis = Duration::from_millis(100);
                        thread::sleep(ten_millis);
                    }
                }
            }
        }
        println!("CLS DONE");
        Ok(())
    }
}
