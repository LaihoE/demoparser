use csgoproto::demo::CDemoClassInfo;

use super::sendtables::Serializer;
use crate::Parser;
use protobuf::Message;

#[derive(Debug, Clone)]
pub struct Class {
    pub class_id: i32,
    pub name: String,
    pub serializer: Serializer,
}

impl Parser {
    pub fn parse_class_info(&mut self, bytes: &Vec<u8>) {
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();

        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();

            let class = Class {
                class_id: cls_id,
                name: network_name.to_string(),
                serializer: self.serializers[network_name].clone(),
            };

            let cls_name = class.name.clone();
            self.cls_by_id.insert(class.class_id, class.clone());
            self.cls_by_name.insert(cls_name, class.clone());
        }
    }
}
