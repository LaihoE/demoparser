use super::read_bits::Bitreader;
use crate::parsing::entities::FieldPath;
use crate::parsing::q_float::QuantalizedFloat;
use crate::Parser;
use ahash::HashMap;
use csgoproto::{
    demo::CDemoSendTables,
    netmessages::{CSVCMsg_FlattenedSerializer, ProtoFlattenedSerializerField_t},
};
use phf_macros::phf_map;
use protobuf::Message;
use regex::Regex;

#[derive(Debug, Clone)]

pub struct Field {
    //pub parent_name: String,
    pub var_name: String,
    pub var_type: String,
    pub send_node: String,
    pub serializer_name: Option<String>,
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
    QuantalizedFloatDecoder(QuantalizedFloat),
    VectorNormalDecoder,
    VectorSpecialDecoder(Option<Box<Decoder>>),
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
    QanglePitchYawDecoder(i32),
    Qangle3Decoder(i32),
    QangleVarDecoder(i32),
    VectorDecoder,
    BaseDecoder,
    NO,
}

use crate::parsing::sendtables::Decoder::*;
use crate::parsing::sendtables::FieldModel::*;

pub static BASETYPE_DECODERS: phf::Map<&'static str, Decoder> = phf_map! {
    //"float32" => FloatDecoder,
    "bool" =>   BooleanDecoder,
    "char" =>    StringDecoder,
    "int16" =>   SignedDecoder,
    "int32" =>   SignedDecoder,
    "int64" =>   SignedDecoder,
    "int8" =>    SignedDecoder,
    "uint16" =>  UnsignedDecoder,
    "uint32" =>  UnsignedDecoder,
    "uint8" =>   UnsignedDecoder,
    "color32" => UnsignedDecoder,
    //"Vector" => VectorDecoder,
    "GameTime_t" => NoscaleDecoder,
    "CBodyComponent" =>       ComponentDecoder,
    "CGameSceneNodeHandle" => UnsignedDecoder,
    "Color" =>                UnsignedDecoder,
    "CPhysicsComponent" =>    ComponentDecoder,
    "CRenderComponent" =>     ComponentDecoder,
    "CUtlString" =>           StringDecoder,
    "CUtlStringToken" =>      UnsignedDecoder,
    "CUtlSymbolLarge" =>      StringDecoder,
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
                    let (_, decoder) = ser.find_decoder(path, pos);
                    return decoder;
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
                    let (_, decoder) = ser.find_decoder(path, pos + 1);
                    return decoder;
                } else {
                    return self.base_decoder.clone().unwrap();
                }
            }
            FieldModelSimple => {
                //println!("DECCCCCCCCC {:?}", self.decoder);
                return self.decoder.clone();
            }
            _ => panic!("HUH"),
        }
    }

    pub fn find_decoder(&mut self, model: FieldModel) {
        self.model = model.clone();
        //println!("MODEL {:?}", model);
        match model {
            FieldModelFixedArray => self.decoder = self.match_decoder(),
            FieldModelSimple => self.decoder = self.match_decoder(),
            FieldModelFixedTable => self.decoder = Decoder::BooleanDecoder,
            FieldModelVariableTable => self.base_decoder = Some(Decoder::UnsignedDecoder),
            FieldModelVariableArray => {
                self.base_decoder = Some(Decoder::UnsignedDecoder);
                //println!("BASE: {:?}", self.base_decoder);

                self.child_decoder = match BASETYPE_DECODERS
                    .get(&self.field_type.generic_type.clone().unwrap().base_type)
                {
                    Some(decoder) => Some(decoder.clone()),
                    None => Some(Decoder::BaseDecoder),
                };
                //println!("childdecoder: {:?}", self.child_decoder);
            }
            FieldModelNOTSET => panic!("wtf"),
        }
    }
    pub fn match_decoder(&self) -> Decoder {
        //println!("BT {}", self.field_type.base_type);
        let dec = match BASETYPE_DECODERS.get(&self.field_type.base_type) {
            Some(decoder) => decoder.clone(),
            None => match self.field_type.base_type.as_str() {
                "float32" => self.find_float_type(),
                "Vector" => self.find_vector_type(3),
                "Vector2D" => self.find_vector_type(2),
                "Vector4D" => self.find_vector_type(4),
                "uint64" => self.find_uint_type(),
                "QAngle" => self.find_qangle_type(),
                "CHandle" => UnsignedDecoder,
                "CNetworkedQuantizedFloat" => self.find_float_type(),
                "CStrongHandle" => self.find_uint_type(),
                "CEntityHandle" => self.find_uint_type(),
                _ => Decoder::NO, //panic!("no decoder {}", self.field_type.base_type.as_str()),
            },
        };
        dec
    }
    pub fn find_qangle_type(&self) -> Decoder {
        match self.encoder.as_str() {
            "qangle_pitch_yaw" => Decoder::QanglePitchYawDecoder(self.bitcount),
            _ => {
                if self.bitcount != 0 {
                    Decoder::Qangle3Decoder(self.bitcount)
                } else {
                    Decoder::QangleVarDecoder(self.bitcount)
                }
            }
        }
    }
    pub fn find_float_type(&self) -> Decoder {
        println!("BT {:?} {}", self.var_name, self.bitcount);
        match self.var_name.as_str() {
            "m_flSimulationTime" => return Decoder::FloatSimulationTimeDecoder,
            "m_flAnimTime" => return Decoder::FloatSimulationTimeDecoder,
            _ => {}
        }

        match self.encoder.as_str() {
            "coord" => Decoder::FloatCoordDecoder,
            "m_flSimulationTime" => Decoder::FloatSimulationTimeDecoder,
            // Maybe dota only?
            "runetime" => Decoder::FloatRuneTimeDecoder,
            _ => {
                // IF NIL?
                if self.bitcount <= 0 || self.bitcount >= 32 {
                    return Decoder::NoscaleDecoder;
                } else {
                    let mut qf = QuantalizedFloat::new(
                        self.bitcount.try_into().unwrap(),
                        Some(self.encode_flags),
                        Some(self.low_value),
                        Some(self.high_value),
                    );
                    return Decoder::QuantalizedFloatDecoder(qf);
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
        let float_type = self.find_float_type();
        return Decoder::VectorSpecialDecoder(Some(Box::new(float_type)));
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
    /*
    println!(
        "{} 0{:?} 1{:?} 2{:?} 3{:?} 4{:?} 5{:?} 6{:?}",
        name,
        captures.get(0),
        captures.get(1),
        captures.get(2),
        captures.get(3),
        captures.get(4),
        captures.get(5),
        captures.get(6),
    );
    */
    //panic!("x");

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
    pub fn find_decoder(&self, path: &FieldPath, pos: usize) -> (Field, Decoder) {
        let idx = path.path[pos];
        let f = &self.fields[idx as usize];
        println!("{}", f.var_name);
        if f.var_name == "m_angRotation" {
            //return (f.clone(), Qangle3Decoder(f.bitcount));
        }

        if f.var_name == "m_PredFloatVariables" && path.last != 1 {
            println!("p {:?} {} last{}", f.field_type, f.var_type, path.last);
            return (f.clone(), NoscaleDecoder);
        }
        if f.var_name == "m_PredVectorVariables" {
            return (
                f.clone(),
                VectorSpecialDecoder(Some(Box::new(NoscaleDecoder))),
            );
        }
        let decoder = f.decoder_from_path(path, pos + 1);
        (f.clone(), decoder)
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
    "CPostProcessingVolume",
];

impl Parser {
    pub fn parse_sendtable(&mut self, tables: CDemoSendTables) {
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint().unwrap();

        let bytes = bitreader.read_n_bytes(n_bytes as usize);
        let serializer_msg: CSVCMsg_FlattenedSerializer =
            Message::parse_from_bytes(&bytes).unwrap();

        let mut fields: HashMap<i32, Field> = HashMap::default();

        for (idx, serializer) in serializer_msg.serializers.iter().enumerate() {
            let mut my_serializer = Serializer {
                name: serializer_msg.symbols[serializer.serializer_name_sym() as usize].clone(),
                fields: vec![],
            };

            for idx in &serializer.fields_index {
                match fields.get(idx) {
                    Some(field) => my_serializer.fields.push(field.clone()),
                    None => {
                        let field_msg = &serializer_msg.fields[*idx as usize];
                        let mut field = field_from_msg(field_msg, &serializer_msg);

                        match &field.serializer_name {
                            Some(name) => match self.serializers.get(name) {
                                Some(ser) => {
                                    field.serializer = Some(ser.clone());
                                }
                                None => {}
                            },
                            None => {}
                        }

                        match &field.serializer {
                            Some(ser) => {
                                if field.field_type.pointer
                                    || PointerTypes.contains(&field.field_type.base_type.as_str())
                                {
                                    field.find_decoder(FieldModelFixedTable)
                                } else {
                                    field.find_decoder(FieldModelVariableTable)
                                }
                            }
                            None => {
                                if field.field_type.count > 0
                                    && field.field_type.base_type != "char"
                                {
                                    //println!("3");
                                    field.find_decoder(FieldModelFixedArray)
                                } else if field.field_type.base_type == "CUtlVector"
                                    || field.field_type.base_type == "CNetworkUtlVectorBase"
                                {
                                    //println!("4");
                                    field.find_decoder(FieldModelVariableArray)
                                } else {
                                    //println!("5");
                                    field.find_decoder(FieldModelSimple)
                                }
                            }
                        }
                        fields.insert(*idx, field.clone());
                        my_serializer.fields.push(field);
                    }
                }
            }
            if idx > 30 {
                //panic!("s")
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

    let ser_name = match field.has_field_serializer_name_sym() {
        true => Some(serializer_msg.symbols[field.field_serializer_name_sym() as usize].clone()),
        false => None,
    };
    let enc_name = match field.has_var_encoder_sym() {
        true => serializer_msg.symbols[field.var_encoder_sym() as usize].clone(),
        false => "".to_string(),
    };
    let hb = field.has_bit_count();
    // println!("GGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG {:?}", hb);
    Field {
        bitcount: field.bit_count(),
        var_name: serializer_msg.symbols[field.var_name_sym() as usize].clone(),
        var_type: serializer_msg.symbols[field.var_type_sym() as usize].clone(),
        send_node: serializer_msg.symbols[field.send_node_sym() as usize].clone(),
        serializer_name: ser_name,
        serializer_version: field.field_serializer_version().clone(),
        encoder: enc_name,
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
