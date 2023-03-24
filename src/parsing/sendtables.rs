use csgoproto::{
    demo::CDemoSendTables,
    netmessages::{CSVCMsg_FlattenedSerializer, ProtoFlattenedSerializerField_t},
};

use super::read_bits::Bitreader;
use crate::Parser;
use protobuf::Message;

struct Field {
    var_name: i32,
    var_type: i32,
    send_node: i32,
    serializer_name: i32,
    serializer_version: i32,
    encoder: i32,
    encode_flags: i32,
    bitcount: i32,
    low_value: f32,
    high_value: f32,
}

struct Serializer {
    name: String,
    fields: Vec<Field>,
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
            println!("{:?}", my_serializer.name);

            for idx in serializer.fields_index {
                let field_msg = &serializer_msg.fields[idx as usize];
                let field = self.field_from_msg(field_msg);
                my_serializer.fields.push(field);
            }
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
