use csgoproto::{
    demo::CDemoSendTables,
    netmessages::{CSVCMsg_FlattenedSerializer, ProtoFlattenedSerializerField_t},
};

use super::read_bits::Bitreader;
use crate::Parser;
use protobuf::Message;

#[derive(Debug, Clone)]

pub struct Field {
    pub var_name: i32,
    pub var_type: i32,
    pub send_node: i32,
    pub serializer_name: i32,
    pub serializer_version: i32,
    pub encoder: i32,
    pub encode_flags: i32,
    pub bitcount: i32,
    pub low_value: f32,
    pub high_value: f32,
}

#[derive(Debug, Clone)]
pub struct Serializer {
    pub name: String,
    // Maybe hm?
    pub fields: Vec<Field>,
}

impl Parser {
    pub fn parse_sendtable(&mut self, tables: CDemoSendTables) {
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint().unwrap();

        let bytes = bitreader.read_n_bytes(n_bytes as usize);
        let serializer_msg: CSVCMsg_FlattenedSerializer =
            Message::parse_from_bytes(&bytes).unwrap();

        for serializer in serializer_msg.serializers {
            let mut my_serializer = Serializer {
                name: serializer_msg.symbols[serializer.serializer_name_sym() as usize].clone(),
                fields: vec![],
            };

            for idx in serializer.fields_index {
                let field_msg = &serializer_msg.fields[idx as usize];
                let field = self.field_from_msg(field_msg);
                my_serializer.fields.push(field);
            }
            self.serializers
                .insert(my_serializer.name.clone(), my_serializer);
        }
    }
    fn field_from_msg(&self, field: &ProtoFlattenedSerializerField_t) -> Field {
        Field {
            bitcount: field.bit_count(),
            var_name: field.var_name_sym(),
            var_type: field.var_type_sym(),
            send_node: field.send_node_sym(),
            serializer_name: field.field_serializer_name_sym(),
            serializer_version: field.field_serializer_version(),
            encoder: field.var_encoder_sym(),
            encode_flags: field.encode_flags(),
            low_value: field.low_value(),
            high_value: field.high_value(),
        }
    }
}
