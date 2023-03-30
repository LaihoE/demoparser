use super::read_bits::Bitreader;
use crate::parsing::entities::FieldPath;
use crate::Parser;
use phf_macros::phf_map;
use regex::Regex;

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
    pub model: FieldModel,
    pub field_type: FieldType,

    pub serializer: Option<Serializer>,
    pub decoder: Decoder,
    pub base_decoder: Option<Decoder>,
    pub child_decoder: Option<Decoder>,
}
#[derive(Debug, Clone)]
pub enum FieldModel {
    FieldModelSimple,
    FieldModelFixedArray,
    FieldModelFixedTable,
    FieldModelVariableArray,
    FieldModelVariableTable,
    FieldModelNOTSET,
}

#[derive(Debug, Clone)]
pub enum Decoder {
    FloatDecoder,
    QuantalizedFloatDecoder,
    VectorNormalDecoder,
    Vector2DDecoder,
    Vector4DDecoder,
    Unsigned64Decoder,
    QangleDecoder,
    ChangleDecoder,
    CstrongHandleDecoder,
    CentityHandleDecoder,
    NoscaleDecoder,
    BooleanDecoder,
    StringDecoder,
    SignedDecoder,
    UnsignedDecoder,
    ComponentDecoder,
    FloatCoordDecoder,
    FloatSimulationTimeDecoder,
    FloatRuneTimeDecoder,
    Fixed64Decoder,
    QanglePitchYawDecoder,
    Qangle3Decoder,
    QangleVarDecoder,
    VectorDecoder,
    BaseDecoder,
}

use crate::parsing::sendtables::Decoder::*;
use crate::parsing::sendtables::FieldModel::*;

pub static BASETYPE_DECODERS: phf::Map<&'static str, Decoder> = phf_map! {
    "float32" => FloatDecoder,
    "bool"=>   BooleanDecoder,
    "char"=>    StringDecoder,
    "color32"=> UnsignedDecoder,
    "int16"=>   SignedDecoder,
    "int32"=>   SignedDecoder,
    "int64"=>   SignedDecoder,
    "int8"=>    SignedDecoder,
    "uint16"=>  UnsignedDecoder,
    "uint32"=>  UnsignedDecoder,
    "uint8"=>   UnsignedDecoder,
    "Vector"=> VectorDecoder,
    "GameTime_t"=> NoscaleDecoder,
    "CBodyComponent"=>       ComponentDecoder,
    "CGameSceneNodeHandle"=> UnsignedDecoder,
    "Color"=>                UnsignedDecoder,
    "CPhysicsComponent"=>    ComponentDecoder,
    "CRenderComponent"=>     ComponentDecoder,
    "CUtlString"=>           StringDecoder,
    "CUtlStringToken"=>      UnsignedDecoder,
    "CUtlSymbolLarge"=>      StringDecoder,
};

impl Field {
    pub fn decoder_from_path(&self, path: &FieldPath, pos: usize) -> Decoder {
        match self.model {
            FieldModelFixedArray => self.decoder.clone(),
            FieldModelFixedTable => {
                if path.last == pos - 1 {
                    return self.base_decoder.clone().unwrap();
                } else {
                    let ser = self.serializer.clone().unwrap();
                    return ser.find_decoder(path, pos);
                }
            }
            FieldModelVariableArray => {
                if path.last == pos {
                    return self.child_decoder.clone().unwrap();
                } else {
                    return self.base_decoder.clone().unwrap();
                }
            }
            FieldModelVariableTable => {
                if path.last >= pos + 1 {
                    let ser = self.serializer.clone().unwrap();
                    return ser.find_decoder(path, pos);
                } else {
                    return Decoder::BaseDecoder;
                }
            }
            _ => Decoder::BaseDecoder,
        }
    }

    pub fn find_decoder(&mut self, model: FieldModel) {
        self.model = model.clone();
        match model {
            fieldModelFixedArray => self.decoder = self.match_decoder(),
            fieldModelSimple => self.decoder = self.match_decoder(),
            fieldModelFixedTable => self.decoder = Decoder::BooleanDecoder,
            fieldModelVariableTable => self.base_decoder = Some(Decoder::UnsignedDecoder),
            fieldModelVariableArray => {
                self.base_decoder = Some(Decoder::UnsignedDecoder);
                self.child_decoder = match BASETYPE_DECODERS
                    .get(&self.field_type.generic_type.clone().unwrap().base_type)
                {
                    Some(decoder) => Some(decoder.clone()),
                    None => Some(Decoder::BaseDecoder),
                };
            }
        }
    }
    pub fn match_decoder(&self) -> Decoder {
        let dec = match BASETYPE_DECODERS.get(&self.field_type.base_type) {
            Some(decoder) => decoder.clone(),
            None => match self.field_type.base_type.as_str() {
                "float32" => self.find_float_type(),
                "Vector" => self.find_vector_type(3),
                "Vector2D" => self.find_vector_type(2),
                "Vector4D" => self.find_vector_type(4),
                "uint64" => self.find_uint_type(),
                "QAngle" => self.find_qangle_type(),
                "CHandle" => self.find_uint_type(),
                "CNetworkedQuantizedFloat" => self.find_float_type(),
                _ => Decoder::BaseDecoder, //panic!("no decoder {}", self.field_type.base_type.as_str()),
            },
        };
        dec
    }
    pub fn find_qangle_type(&self) -> Decoder {
        match self.encoder.as_str() {
            "qangle_pitch_yaw" => Decoder::QanglePitchYawDecoder,
            _ => {
                if self.bitcount != 0 {
                    Decoder::Qangle3Decoder
                } else {
                    Decoder::QangleVarDecoder
                }
            }
        }
    }
    pub fn find_float_type(&self) -> Decoder {
        match self.encoder.as_str() {
            "coord" => Decoder::FloatCoordDecoder,
            "simtime" => Decoder::FloatSimulationTimeDecoder,
            // Maybe dota only?
            "runetime" => Decoder::FloatRuneTimeDecoder,
            _ => {
                // IF NIL?
                if self.bitcount <= 0 || self.bitcount >= 32 {
                    return Decoder::NoscaleDecoder;
                } else {
                    return Decoder::QuantalizedFloatDecoder;
                }
            }
        }
    }
    pub fn find_uint_type(&self) -> Decoder {
        match self.encoder.as_str() {
            "fixed64" => Decoder::Fixed64Decoder,
            _ => Decoder::Unsigned64Decoder,
        }
    }
    pub fn find_vector_type(&self, n: u32) -> Decoder {
        if n == 3 && self.encoder == "normal" {
            return Decoder::VectorNormalDecoder;
        }
        return Decoder::VectorNormalDecoder;
    }
}

#[derive(Debug, Clone)]
pub struct FieldType {
    pub base_type: String,
    pub generic_type: Option<Box<FieldType>>,
    pub pointer: bool,
    pub count: i32,
}

fn find_field_type(name: &str) -> FieldType {
    let re = Regex::new(r"([^<\[\*]+)(<\s(.*)\s>)?(\*)?(\[(.*)\])?").unwrap();
    let captures = re.captures(name).unwrap();
    println!("{:?}", captures);
    let base_type = captures.get(1).unwrap().as_str().to_owned();
    let pointer = match captures.get(4) {
        Some(s) => s.as_str() == "*",
        None => false,
    };
    let mut ft = FieldType {
        base_type: base_type,
        pointer: pointer,
        generic_type: None,
        count: 0,
    };
    ft.generic_type = match captures.get(3) {
        Some(generic) => Some(Box::new(find_field_type(generic.as_str()))),
        None => None,
    };
    ft.count = match captures.get(6) {
        Some(n) => n.as_str().parse::<i32>().unwrap(),
        None => 0,
    };
    return ft;
}
#[derive(Debug, Clone)]
pub struct Serializer {
    pub name: String,
    // Maybe hm?
    pub fields: Vec<Field>,
}

impl Field {
    pub fn is_vector(&self) -> bool {
        // Serializer id ?!?
        match self.var_type.as_str() {
            "CUtlVector" => return true,
            "CNetworkUtlVectorBase" => return true,
            _ => false,
        }
    }
    pub fn is_ptr(&self) -> bool {
        match self.var_type.as_str() {
            "CBodyComponent" => true,
            "CLightComponent" => true,
            "CPhysicsComponent" => true,
            "CRenderComponent" => true,
            "CPlayerLocalData" => true,
            _ => false,
        }
    }
    pub fn is_array(&self) -> bool {
        match self.var_type.as_str() {
            "CBodyComponent" => true,
            "CLightComponent" => true,
            "CPhysicsComponent" => true,
            "CRenderComponent" => true,
            "CPlayerLocalData" => true,
            _ => false,
        }
    }
}

impl Serializer {
    pub fn find_decoder(&self, path: &FieldPath, pos: usize) -> Decoder {
        let idx = path.path[pos];
        let f = self.fields[idx as usize].decoder_from_path(path, pos);
        f
    }
}

const PointerTypes: &'static [&'static str] = &[
    // TODO CS ONES
    "PhysicsRagdollPose_t",
    "CBodyComponent",
    "CEntityIdentity",
    "CPhysicsComponent",
    "CRenderComponent",
    "CDOTAGamerules",
    "CDOTAGameManager",
    "CDOTASpectatorGraphManager",
    "CPlayerLocalData",
    "CPlayer_CameraServices",
    "CDOTAGameRules",
];

impl Parser {
    pub fn parse_sendtable(&mut self, tables: CDemoSendTables) {
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint().unwrap();

        let bytes = bitreader.read_n_bytes(n_bytes as usize);
        let serializer_msg: CSVCMsg_FlattenedSerializer =
            Message::parse_from_bytes(&bytes).unwrap();

        for serializer in &serializer_msg.serializers {
            let mut my_serializer = Serializer {
                name: serializer_msg.symbols[serializer.serializer_name_sym() as usize].clone(),
                fields: vec![],
            };

            for idx in &serializer.fields_index {
                let field_msg = &serializer_msg.fields[*idx as usize];
                let mut field = field_from_msg(field_msg, &serializer_msg);

                if field.serializer_name != "" {
                    match self.serializers.get(&field.serializer_name) {
                        Some(ser) => {
                            field.serializer =
                                Some(self.serializers[&field.serializer_name].clone());
                        }
                        None => {}
                    }
                }
                match &field.serializer {
                    Some(ser) => {
                        if field.field_type.pointer
                            || PointerTypes.contains(&field.field_type.base_type.as_str())
                        {
                            field.find_decoder(FieldModelFixedArray)
                        } else {
                            field.find_decoder(FieldModelVariableTable)
                        }
                    }
                    None => {
                        if field.field_type.count > 0 && field.field_type.base_type != "char" {
                            field.find_decoder(FieldModelFixedArray)
                        } else if field.field_type.base_type == "CUtlVector"
                            || field.field_type.base_type == "CNetworkUtlVectorBase"
                        {
                            field.find_decoder(FieldModelVariableArray)
                        } else {
                            field.find_decoder(FieldModelSimple)
                        }
                    }
                }
                my_serializer.fields.push(field);
            }

            self.serializers
                .insert(my_serializer.name.clone(), my_serializer);
        }
    }
}

fn field_from_msg(
    field: &ProtoFlattenedSerializerField_t,
    serializer_msg: &CSVCMsg_FlattenedSerializer,
) -> Field {
    let field_type = find_field_type(&serializer_msg.symbols[field.var_type_sym() as usize]);
    Field {
        bitcount: field.bit_count(),
        var_name: serializer_msg.symbols[field.var_name_sym() as usize].clone(),
        var_type: serializer_msg.symbols[field.var_type_sym() as usize].clone(),
        send_node: serializer_msg.symbols[field.send_node_sym() as usize].clone(),
        serializer_name: serializer_msg.symbols[field.field_serializer_name_sym() as usize].clone(),
        serializer_version: field.field_serializer_version().clone(),
        encoder: serializer_msg.symbols[field.var_encoder_sym() as usize].clone(),
        encode_flags: field.encode_flags(),
        low_value: field.low_value(),
        high_value: field.high_value(),
        model: FieldModelNOTSET,
        field_type: field_type,
        serializer: None,
        decoder: BaseDecoder,
        base_decoder: None,
        child_decoder: None,
    }
}
