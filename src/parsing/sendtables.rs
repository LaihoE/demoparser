use super::read_bits::Bitreader;
use crate::parsing::entities::FieldPath;
use crate::Parser;
use ahash::HashMap;
use csgoproto::{
    demo::CDemoSendTables,
    netmessages::{CSVCMsg_FlattenedSerializer, ProtoFlattenedSerializerField_t},
};
use protobuf::Message;

#[derive(Debug, Clone)]

pub struct Field {
    //pub parent_name: String,
    pub var_name: String,
    pub var_type: String,
    pub send_node: String,
    pub serializer_name: String,
    pub serializer_version: i32,
    pub encoder: String,
    pub encode_flags: i32,
    pub bitcount: i32,
    pub low_value: f32,
    pub high_value: f32,
    pub model: i32,
}

#[derive(Debug, Clone)]
pub struct Serializer {
    pub name: String,
    // Maybe hm?
    pub fields: Vec<Field>,
}
impl Serializer {
    pub fn find_decoder(&self, field_path: &FieldPath, idx: usize) -> Field {
        let i = field_path.path[idx];
        self.fields[i as usize].clone() //.find_decoder_field_path(field_path, idx)
    }
}
impl Field {
    pub fn find_decoder_field_path(&self, field_path: &FieldPath, idx: usize) {
        // Todo
        // self
    }
}

impl Parser {
    pub fn parse_sendtable(&mut self, tables: CDemoSendTables) {
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint().unwrap();

        let bytes = bitreader.read_n_bytes(n_bytes as usize);
        let serializer_msg: CSVCMsg_FlattenedSerializer =
            Message::parse_from_bytes(&bytes).unwrap();

        //let field_types = HashMap::default();
        //println!("{:?}", serializer_msg);
        //serializer_msg.sm
        for serializer in &serializer_msg.serializers {
            let mut my_serializer = Serializer {
                name: serializer_msg.symbols[serializer.serializer_name_sym() as usize].clone(),
                fields: vec![],
            };

            for idx in &serializer.fields_index {
                let field_msg = &serializer_msg.fields[*idx as usize];
                let field = self.field_from_msg(field_msg, &serializer_msg);
                println!("{:#?}", field);
                my_serializer.fields.push(field);
            }

            self.serializers
                .insert(my_serializer.name.clone(), my_serializer);
        }
    }
    fn field_from_msg(
        &self,
        field: &ProtoFlattenedSerializerField_t,
        serializer_msg: &CSVCMsg_FlattenedSerializer,
    ) -> Field {
        Field {
            bitcount: field.bit_count(),
            var_name: serializer_msg.symbols[field.var_name_sym() as usize].clone(),
            var_type: serializer_msg.symbols[field.var_type_sym() as usize].clone(),
            send_node: serializer_msg.symbols[field.send_node_sym() as usize].clone(),
            serializer_name: serializer_msg.symbols[field.field_serializer_name_sym() as usize]
                .clone(),
            serializer_version: field.field_serializer_version().clone(),
            encoder: serializer_msg.symbols[field.var_encoder_sym() as usize].clone(),
            encode_flags: field.encode_flags(),
            low_value: field.low_value(),
            high_value: field.high_value(),
            model: 0,
        }
    }
}
