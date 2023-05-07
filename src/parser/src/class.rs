use super::{read_bits::DemoParserError, sendtables::Serializer};
use crate::parser_settings::Parser;
use ahash::HashSet;
use csgoproto::demo::CDemoClassInfo;
use protobuf::Message;

#[derive(Debug, Clone)]
pub struct Class {
    pub class_id: i32,
    pub name: String,
    pub serializer: Serializer,
    pub history: HashSet<u64>,
}

impl<'a> Parser<'a> {
    pub fn parse_class_info(&mut self, bytes: &[u8]) -> Result<(), DemoParserError> {
        if !self.parse_entities {
            return Ok(());
        }
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();
        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();
            self.cls_by_id[cls_id as usize] = Some(Class {
                class_id: cls_id,
                name: network_name.to_string(),
                serializer: self.serializers[network_name].clone(),
                history: HashSet::default(),
            });
        }
        Ok(())
    }
}
