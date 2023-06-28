use super::read_bits::{Bitreader, DemoParserError};
use crate::collect_data::PropType;
use crate::collect_data::TYPEHM;
use crate::decoder::QfMapper;
use crate::entities_utils::FieldPath;
use crate::parser_settings::Parser;
use crate::parser_thread_settings::SpecialIDs;
use crate::q_float::QuantalizedFloat;
use crate::sendtables::Decoder::*;
use crate::sendtables::FieldModel::*;
use ahash::AHashMap;
use ahash::AHashSet;
use ahash::HashMap;
use csgoproto::{
    demo::CDemoSendTables,
    netmessages::{CSVCMsg_FlattenedSerializer, ProtoFlattenedSerializerField_t},
};
use itertools::Itertools;
use lazy_static::lazy_static;
use phf_macros::phf_map;
use protobuf::Message;
use regex::Regex;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Field {
    //pub parent_name: String,
    pub var_name: String,
    pub var_type: String,
    pub send_node: String,
    pub serializer_name: Option<String>,
    pub encoder: String,
    pub encode_flags: i32,
    pub bitcount: i32,
    pub low_value: f32,
    pub high_value: f32,
    pub model: FieldModel,
    pub field_type: FieldType,
    pub decoder: Decoder,

    pub serializer: Option<Serializer>,
    pub base_decoder: Option<Decoder>,
    pub child_decoder: Option<Decoder>,

    pub should_parse: bool,
    pub prop_id: usize,
    pub is_controller_prop: bool,
    pub controller_prop: Option<ControllerProp>,
    pub idx: u32,
}
#[derive(Debug, Clone, Copy)]
pub struct FieldInfo {
    pub decoder: Decoder,
    pub should_parse: bool,
    pub df_pos: u32,
    pub controller_prop: Option<ControllerProp>,
}
#[derive(Debug, Clone, Copy)]
pub enum ControllerProp {
    SteamId,
    Name,
    TeamNum,
    PlayerEntityId,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FieldModel {
    FieldModelSimple,
    FieldModelFixedArray,
    FieldModelFixedTable,
    FieldModelVariableArray,
    FieldModelVariableTable,
    FieldModelNOTSET,
}
use std::fmt;

impl fmt::Display for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}
#[derive(Clone, Debug)]
pub struct PropController {
    pub id: u32,
    pub wanted_player_props: Vec<String>,
    pub wanted_prop_ids: Vec<u32>,
    pub prop_infos: Vec<PropInfo>,
    pub prop_name_to_path: AHashMap<String, [i32; 7]>,
    pub path_to_prop_name: AHashMap<[i32; 7], String>,
    pub name_to_id: AHashMap<String, u32>,
    pub id_to_path: AHashMap<u32, [i32; 7]>,
    pub id_to_name: AHashMap<u32, String>,
    pub special_ids: SpecialIDs,
    pub wanted_player_og_props: Vec<String>,
    pub real_name_to_og_name: AHashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decoder {
    QuantalizedFloatDecoder(u8),
    VectorNormalDecoder,
    VectorNoscaleDecoder,
    VectorFloatCoordDecoder,
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
    Fixed64Decoder,
    QanglePitchYawDecoder,
    Qangle3Decoder,
    QangleVarDecoder,
    BaseDecoder,
    AmmoDecoder,
    QanglePresDecoder,
}

pub static BASETYPE_DECODERS: phf::Map<&'static str, Decoder> = phf_map! {
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

    // "float32" => NoscaleDecoder,
    // "Vector" => VectorDecoder,

    "GameTime_t" => NoscaleDecoder,
    "CBodyComponent" =>       ComponentDecoder,
    "CGameSceneNodeHandle" => UnsignedDecoder,
    "Color" =>                UnsignedDecoder,
    "CPhysicsComponent" =>    ComponentDecoder,
    "CRenderComponent" =>     ComponentDecoder,
    "CUtlString" =>           StringDecoder,
    "CUtlStringToken" =>      UnsignedDecoder,
    "CUtlSymbolLarge" =>      StringDecoder,

    "Quaternion" => NoscaleDecoder,
    "CTransform" => NoscaleDecoder,
    "HSequence" => Unsigned64Decoder,
    "AttachmentHandle_t"=> Unsigned64Decoder,
    "CEntityIndex"=> Unsigned64Decoder,

    "MoveCollide_t"=> Unsigned64Decoder,
    "MoveType_t"=> Unsigned64Decoder,
    "RenderMode_t"=> Unsigned64Decoder,
    "RenderFx_t"=> Unsigned64Decoder,
    "SolidType_t"=> Unsigned64Decoder,
    "SurroundingBoundsType_t"=> Unsigned64Decoder,
    "ModelConfigHandle_t"=> Unsigned64Decoder,
    "NPC_STATE"=> Unsigned64Decoder,
    "StanceType_t"=> Unsigned64Decoder,
    "AbilityPathType_t"=> Unsigned64Decoder,
    "WeaponState_t"=> Unsigned64Decoder,
    "DoorState_t"=> Unsigned64Decoder,
    "RagdollBlendDirection"=> Unsigned64Decoder,
    "BeamType_t"=> Unsigned64Decoder,
    "BeamClipStyle_t"=> Unsigned64Decoder,
    "EntityDisolveType_t"=> Unsigned64Decoder,
    "tablet_skin_state_t" => Unsigned64Decoder,
    "CStrongHandle" => Unsigned64Decoder,
    "CSWeaponMode" => Unsigned64Decoder,
    "ESurvivalSpawnTileState"=> Unsigned64Decoder,
    "SpawnStage_t"=> Unsigned64Decoder,
    "ESurvivalGameRuleDecision_t"=> Unsigned64Decoder,
    "RelativeDamagedDirection_t"=> Unsigned64Decoder,
    "CSPlayerState"=> Unsigned64Decoder,
    "MedalRank_t"=> Unsigned64Decoder,
    "CSPlayerBlockingUseAction_t"=> Unsigned64Decoder,
    "MoveMountingAmount_t"=> Unsigned64Decoder,
    "QuestProgress::Reason"=> Unsigned64Decoder,


};

impl Field {
    pub fn decoder_from_path(&self, path: &FieldPath, pos: usize) -> FieldInfo {
        match self.model {
            FieldModelSimple => {
                return FieldInfo {
                    decoder: self.decoder,
                    should_parse: self.should_parse,
                    df_pos: self.prop_id as u32,
                    controller_prop: self.controller_prop,
                };
            }
            FieldModelFixedArray => {
                return FieldInfo {
                    decoder: self.decoder,
                    should_parse: self.should_parse,
                    df_pos: self.prop_id as u32,
                    controller_prop: self.controller_prop,
                }
            }
            FieldModelFixedTable => {
                if path.last == pos - 1 {
                    if self.base_decoder.is_some() {
                        return FieldInfo {
                            decoder: self.base_decoder.unwrap(),
                            should_parse: self.should_parse,
                            df_pos: self.prop_id as u32,
                            controller_prop: self.controller_prop,
                        };
                    }
                    return FieldInfo {
                        decoder: self.decoder,
                        should_parse: self.should_parse,
                        df_pos: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                } else {
                    match &self.serializer {
                        Some(ser) => {
                            return ser.find_decoder(path, pos);
                        }
                        None => panic!("no serializer for path"),
                    }
                }
            }
            FieldModelVariableArray => {
                if path.last == pos {
                    return FieldInfo {
                        decoder: self.child_decoder.unwrap(),
                        should_parse: self.should_parse,
                        df_pos: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                } else {
                    return FieldInfo {
                        decoder: self.base_decoder.unwrap(),
                        should_parse: self.should_parse,
                        df_pos: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                }
            }
            FieldModelVariableTable => {
                if path.last >= pos + 1 {
                    match &self.serializer {
                        Some(ser) => {
                            return ser.find_decoder(path, pos + 1);
                        }
                        None => panic!("no serializer for path"),
                    }
                } else {
                    return FieldInfo {
                        decoder: self.base_decoder.unwrap(),
                        should_parse: self.should_parse,
                        df_pos: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                }
            }
            _ => panic!("HUH"),
        }
    }
    pub fn debug_decoder_from_path(
        &self,
        path: &FieldPath,
        pos: usize,
        prop_name: String,
    ) -> DebugField {
        match self.model {
            FieldModelSimple => {
                return DebugField {
                    full_name: prop_name + "." + &self.var_name.clone(),
                    field: Some(self.clone()),
                    decoder: self.decoder,
                };
            }
            FieldModelFixedArray => DebugField {
                full_name: prop_name + "." + &self.var_name.clone(),
                field: Some(self.clone()),
                decoder: self.decoder,
            },
            FieldModelFixedTable => {
                if path.last == pos - 1 {
                    if self.base_decoder.is_some() {
                        return DebugField {
                            full_name: prop_name + "." + &self.var_name.clone(),
                            field: Some(self.clone()),
                            decoder: self.base_decoder.unwrap(),
                        };
                    } else {
                        return DebugField {
                            full_name: prop_name + "." + &self.var_name.clone(),
                            field: Some(self.clone()),
                            decoder: self.decoder,
                        };
                    }
                } else {
                    match &self.serializer {
                        Some(ser) => {
                            return ser.debug_find_decoder(path, pos, prop_name + "." + &ser.name);
                        }
                        None => panic!("no serializer for path"),
                    }
                }
            }
            FieldModelVariableArray => {
                if path.last == pos {
                    return DebugField {
                        full_name: prop_name + "." + &self.var_name.clone(),
                        field: Some(self.clone()),
                        decoder: self.child_decoder.unwrap(),
                    };
                } else {
                    return DebugField {
                        full_name: prop_name + "." + &self.var_name.clone(),
                        field: Some(self.clone()),
                        decoder: self.base_decoder.unwrap(),
                    };
                }
            }
            FieldModelVariableTable => {
                if path.last >= pos + 1 {
                    match &self.serializer {
                        Some(ser) => {
                            return ser.debug_find_decoder(
                                path,
                                pos + 1,
                                prop_name + "." + &ser.name,
                            );
                        }
                        None => panic!("no serializer for path"),
                    }
                } else {
                    return DebugField {
                        full_name: prop_name + "." + &self.var_name.clone(),
                        field: Some(self.clone()),
                        decoder: self.base_decoder.unwrap(),
                    };
                }
            }
            _ => panic!("HUH"),
        }
    }

    pub fn find_decoder(&mut self, model: FieldModel, qf_map: &mut QfMapper) {
        self.model = model.clone();
        match model {
            FieldModelFixedArray => self.decoder = self.match_decoder(qf_map),
            FieldModelSimple => self.decoder = self.match_decoder(qf_map),
            FieldModelFixedTable => self.decoder = Decoder::BooleanDecoder,
            FieldModelVariableTable => self.base_decoder = Some(Decoder::UnsignedDecoder),
            FieldModelVariableArray => {
                self.base_decoder = Some(Decoder::UnsignedDecoder);
                match self.var_name.as_str() {
                    // Dont know why these 4 break the parsing
                    "m_PredFloatVariables" => self.child_decoder = Some(NoscaleDecoder),
                    "m_OwnerOnlyPredNetFloatVariables" => self.child_decoder = Some(NoscaleDecoder),
                    "m_OwnerOnlyPredNetVectorVariables" => {
                        self.child_decoder = Some(VectorNoscaleDecoder)
                    }
                    "m_PredVectorVariables" => self.child_decoder = Some(VectorNoscaleDecoder),
                    _ => {
                        self.child_decoder = match BASETYPE_DECODERS
                            .get(&self.field_type.generic_type.clone().unwrap().base_type)
                        {
                            Some(decoder) => Some(decoder.clone()),
                            None => Some(Decoder::BaseDecoder),
                        };
                    }
                }
            }
            FieldModelNOTSET => panic!("Field model not set??"),
        }
    }
    pub fn match_decoder(&self, qf_map: &mut QfMapper) -> Decoder {
        if self.var_name == "m_iClip1" {
            return Decoder::AmmoDecoder;
        }
        let dec = match BASETYPE_DECODERS.get(&self.field_type.base_type) {
            Some(decoder) => decoder.clone(),
            None => match self.field_type.base_type.as_str() {
                "float32" => self.find_float_type(qf_map),
                "Vector" => self.find_vector_type(3, qf_map),
                "Vector2D" => self.find_vector_type(2, qf_map),
                "Vector4D" => self.find_vector_type(4, qf_map),
                "uint64" => self.find_uint_type(),
                "QAngle" => self.find_qangle_type(),
                "CHandle" => UnsignedDecoder,
                "CNetworkedQuantizedFloat" => self.find_float_type(qf_map),
                "CStrongHandle" => self.find_uint_type(),
                "CEntityHandle" => self.find_uint_type(),
                _ => Decoder::UnsignedDecoder,
            },
        };
        dec
    }
    pub fn find_qangle_type(&self) -> Decoder {
        match self.var_name.as_str() {
            "m_angEyeAngles" => Decoder::QanglePitchYawDecoder,
            _ => {
                if self.bitcount != 0 {
                    Decoder::Qangle3Decoder
                } else {
                    Decoder::QangleVarDecoder
                }
            }
        }
    }
    pub fn find_float_type(&self, qf_map: &mut QfMapper) -> Decoder {
        match self.var_name.as_str() {
            "m_flSimulationTime" => return Decoder::FloatSimulationTimeDecoder,
            "m_flAnimTime" => return Decoder::FloatSimulationTimeDecoder,
            _ => {}
        }
        match self.encoder.as_str() {
            "coord" => Decoder::FloatCoordDecoder,
            "m_flSimulationTime" => Decoder::FloatSimulationTimeDecoder,
            _ => {
                if self.bitcount <= 0 || self.bitcount >= 32 {
                    return Decoder::NoscaleDecoder;
                } else {
                    let qf = QuantalizedFloat::new(
                        self.bitcount.try_into().unwrap(),
                        Some(self.encode_flags),
                        Some(self.low_value),
                        Some(self.high_value),
                    );
                    let idx = qf_map.idx;
                    qf_map.map.insert(idx, qf);
                    qf_map.idx += 1;
                    return Decoder::QuantalizedFloatDecoder(idx as u8);
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
    pub fn find_vector_type(&self, n: u32, qf_map: &mut QfMapper) -> Decoder {
        if n == 3 && self.encoder == "normal" {
            return Decoder::VectorNormalDecoder;
        }
        let float_type = self.find_float_type(qf_map);
        match float_type {
            NoscaleDecoder => return VectorNoscaleDecoder,
            FloatCoordDecoder => return VectorFloatCoordDecoder,
            _ => panic!("e"),
        }
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
    lazy_static! {
        static ref RE: Regex = Regex::new(r"([^<\[\*]+)(<\s(.*)\s>)?(\*)?(\[(.*)\])?").unwrap();
    }
    let captures = RE.captures(name).unwrap();
    let base_type = captures.get(1).unwrap().as_str().to_owned();
    let pointer = match captures.get(4) {
        Some(s) => {
            if s.as_str() == "*" {
                true
            } else {
                if POINTER_TYPES.contains(&name) {
                    true
                } else {
                    false
                }
            }
        }
        None => {
            if POINTER_TYPES.contains(&name) {
                true
            } else {
                false
            }
        }
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
    pub fields: Vec<Field>,
}

impl Serializer {
    pub fn find_decoder(&self, path: &FieldPath, pos: usize) -> FieldInfo {
        self.fields[path.path[pos] as usize].decoder_from_path(path, pos + 1)
    }
    pub fn debug_find_decoder(
        &self,
        path: &FieldPath,
        pos: usize,
        prop_name: String,
    ) -> DebugField {
        let idx = path.path[pos];
        let f = &self.fields[idx as usize];
        f.debug_decoder_from_path(path, pos + 1, prop_name)
    }
}

const POINTER_TYPES: &'static [&'static str] = &[
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
    "CBodyComponentDCGBaseAnimating",
    "CBodyComponentBaseAnimating",
    "CBodyComponentBaseAnimatingOverlay",
    "CBodyComponentBaseModelEntity",
    "CBodyComponentSkeletonInstance",
    "CBodyComponentPoint",
    "CLightComponent",
    "CRenderComponent",
    "CPhysicsComponent",
];

impl Parser {
    // This part is so insanely complicated. There are multiple versions of each serializer and
    // each serializer is this huge nested struct.
    pub fn parse_sendtable(
        tables: CDemoSendTables,
        wanted_props: Vec<String>,
        wanted_props_og_names: Vec<String>,
        real_name_to_og_name: AHashMap<String, String>,
    ) -> Result<(AHashMap<String, Serializer>, QfMapper, PropController), DemoParserError> {
        let before = Instant::now();
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint()?;
        let bytes = bitreader.read_n_bytes(n_bytes as usize)?;
        let serializer_msg: CSVCMsg_FlattenedSerializer =
            Message::parse_from_bytes(&bytes).unwrap();
        let mut serializers: AHashMap<String, Serializer> = AHashMap::default();
        let mut qf_mapper = QfMapper {
            idx: 0,
            map: AHashMap::default(),
        };
        let mut fields: HashMap<i32, Field> = HashMap::default();
        let mut prop_controller = PropController::new(
            wanted_props.clone(),
            wanted_props_og_names,
            real_name_to_og_name,
        );
        for (ii, serializer) in serializer_msg.serializers.iter().enumerate() {
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
                            Some(name) => match serializers.get(name) {
                                Some(ser) => {
                                    field.serializer = Some(ser.clone());
                                }
                                None => {}
                            },
                            None => {}
                        }

                        match &field.serializer {
                            Some(_) => {
                                if field.field_type.pointer
                                    || POINTER_TYPES.contains(&field.field_type.base_type.as_str())
                                {
                                    field.find_decoder(FieldModelFixedTable, &mut qf_mapper)
                                } else {
                                    field.find_decoder(FieldModelVariableTable, &mut qf_mapper)
                                }
                            }
                            None => {
                                if field.field_type.count > 0
                                    && field.field_type.base_type != "char"
                                {
                                    field.find_decoder(FieldModelFixedArray, &mut qf_mapper)
                                } else if field.field_type.base_type == "CUtlVector"
                                    || field.field_type.base_type == "CNetworkUtlVectorBase"
                                {
                                    field.find_decoder(FieldModelVariableArray, &mut qf_mapper)
                                } else {
                                    field.find_decoder(FieldModelSimple, &mut qf_mapper)
                                }
                            }
                        }
                        if field.encoder == "qangle_precise" {
                            field.decoder = QanglePresDecoder;
                        }
                        fields.insert(*idx, field.clone());
                        my_serializer.fields.push(field);
                    }
                }
            }
            if my_serializer.name.contains("Player")
                || my_serializer.name.contains("Controller")
                || my_serializer.name.contains("Team")
                || my_serializer.name.contains("Weapon")
                || my_serializer.name.contains("AK")
                || my_serializer.name.contains("cell")
                || my_serializer.name.contains("vec")
                || my_serializer.name.contains("Projectile")
            {
                prop_controller.find_prop_name_paths(&mut my_serializer);
            }
            //prop_controller.find_prop_name_paths(&mut my_serializer);
            serializers.insert(my_serializer.name.clone(), my_serializer);
        }
        if wanted_props.contains(&("weapon_name".to_string())) {
            prop_controller.prop_infos.push(PropInfo {
                id: 9999992,
                prop_type: Some(PropType::Custom),
                prop_name: "weapon_name".to_string(),
                prop_friendly_name: "active_weapon_name".to_string(),
            });
        }
        prop_controller.prop_infos.push(PropInfo {
            id: 9999999,
            prop_type: None,
            prop_name: "tick".to_string(),
            prop_friendly_name: "tick".to_string(),
        });
        prop_controller.prop_infos.push(PropInfo {
            id: 9999998,
            prop_type: None,
            prop_name: "steamid".to_string(),
            prop_friendly_name: "steamid".to_string(),
        });
        prop_controller.prop_infos.push(PropInfo {
            id: 9999997,
            prop_type: None,
            prop_name: "name".to_string(),
            prop_friendly_name: "name".to_string(),
        });
        Ok((serializers, qf_mapper, prop_controller))
    }
}
impl PropController {
    pub fn new(
        wanted_player_props: Vec<String>,
        wanted_player_props_og_names: Vec<String>,
        real_name_to_og_name: AHashMap<String, String>,
    ) -> Self {
        PropController {
            id: 0,
            wanted_player_props: wanted_player_props,
            wanted_prop_ids: vec![],
            prop_infos: vec![],
            prop_name_to_path: AHashMap::default(),
            path_to_prop_name: AHashMap::default(),
            name_to_id: AHashMap::default(),
            id_to_path: AHashMap::default(),
            special_ids: SpecialIDs::new(),
            wanted_player_og_props: wanted_player_props_og_names,
            id_to_name: AHashMap::default(),
            real_name_to_og_name: real_name_to_og_name,
        }
    }
    pub fn find_prop_name_paths(&mut self, ser: &mut Serializer) {
        self.traverse_fields(&mut ser.fields, vec![], ser.name.clone())
    }
    fn vec_to_arr(path: &Vec<i32>) -> [i32; 7] {
        let mut arr = [0, 0, 0, 0, 0, 0, 0];
        for (idx, val) in path.iter().enumerate() {
            arr[idx] = *val;
        }
        arr
    }
    fn handle_normal_prop(&mut self, full_name: &str) {
        self.prop_infos.push(PropInfo {
            id: self.id,
            prop_type: TYPEHM.get(&full_name).copied(),
            prop_name: full_name.to_string(),
            prop_friendly_name: self
                .real_name_to_og_name
                .get(&full_name.to_string())
                .unwrap_or(&full_name.to_string())
                .to_string(),
        });
    }
    fn handle_weapon_prop(&mut self, weap_prop: &str, f: &mut Field) {
        match self.name_to_id.get(weap_prop) {
            // If we already have an id for prop of same name then use that
            Some(id) => {
                f.prop_id = *id as usize;
                return; // f.prop_id = *id as usize;
            }
            None => match TYPEHM.get(&weap_prop) {
                Some(t) => {
                    self.name_to_id.insert(weap_prop.to_string(), self.id);
                    self.prop_infos.push(PropInfo {
                        id: self.id,
                        prop_type: Some(t.clone()),
                        prop_name: weap_prop.to_string(),
                        prop_friendly_name: self
                            .real_name_to_og_name
                            .get(&weap_prop.to_string())
                            .unwrap_or(&weap_prop.to_string())
                            .to_string(),
                    })
                }
                _ => panic!("weapon prop: {:?} not found", weap_prop),
            },
        };
    }
    fn handle_prop(&mut self, full_name: &str, f: &mut Field, arr: [i32; 7]) {
        let split_at_dot: Vec<&str> = full_name.split(".").collect();
        let weap_prop = split_at_dot.last().unwrap();

        let is_wanted_normal_prop = self.wanted_player_props.contains(&full_name.to_string());
        let is_wanted_weapon_prop = self.wanted_player_props.contains(&weap_prop.to_string());

        if is_wanted_normal_prop && is_wanted_weapon_prop {
            panic!("{:?} {:?}", full_name, weap_prop);
        }

        if full_name.contains("m_iItemDefinitionIndex")
            && (full_name.contains("Weapon") || full_name.contains("AK"))
        {
            f.should_parse = true;
            f.prop_id = 699999;
            self.special_ids.item_def = Some(699999);
            return;
        }

        if is_wanted_normal_prop {
            self.handle_normal_prop(full_name);
            self.name_to_id.insert(full_name.to_string(), self.id);
        } else if is_wanted_weapon_prop {
            self.handle_weapon_prop(weap_prop, f);
            f.should_parse = true;
        }
        self.set_grenades(full_name, f);
        self.set_custom(full_name, f);
        self.match_names(full_name, f);

        if is_wanted_normal_prop {
            f.prop_id = self.id as usize;
            f.should_parse = true;
        }
        self.id += 1;
    }
    fn set_custom(&mut self, full_name: &str, f: &mut Field) {}
    fn match_names(&mut self, full_name: &str, f: &mut Field) {
        match full_name {
            "CCSTeam.m_iTeamNum" => self.special_ids.team_team_num = Some(self.id),
            "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellX" => {
                f.should_parse = true;
                self.special_ids.cell_x_player = Some(self.id);
                self.prop_infos.push(PropInfo {
                    id: 9999944,
                    prop_type: Some(PropType::Custom),
                    prop_name: "X".to_string(),
                    prop_friendly_name: "X".to_string(),
                });
            }
            "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecX" => {
                self.special_ids.cell_x_offset_player = Some(self.id)
            }
            "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellY" => {
                f.should_parse = true;
                self.special_ids.cell_y_player = Some(self.id);
                self.prop_infos.push(PropInfo {
                    id: 9999945,
                    prop_type: Some(PropType::Custom),
                    prop_name: "Y".to_string(),
                    prop_friendly_name: "Y".to_string(),
                });
            }
            "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecY" => {
                self.special_ids.cell_y_offset_player = Some(self.id)
            }
            "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellZ" => {
                self.special_ids.cell_z_player = Some(self.id)
            }
            "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecZ" => {
                self.special_ids.cell_z_offset_player = Some(self.id)
            }
            "CCSPlayerPawn.m_iTeamNum" => self.special_ids.player_team_pointer = Some(self.id),
            "CBasePlayerWeapon.m_nOwnerId" => self.special_ids.weapon_owner_pointer = Some(self.id),
            "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon" => {
                self.special_ids.active_weapon = Some(self.id)
            }
            "CCSPlayerController.m_iTeamNum" => self.special_ids.teamnum = Some(self.id),
            "CCSPlayerController.m_iszPlayerName" => self.special_ids.player_name = Some(self.id),
            "CCSPlayerController.m_steamID" => self.special_ids.steamid = Some(self.id),
            "CCSPlayerController.m_hPlayerPawn" => self.special_ids.player_pawn = Some(self.id),
            _ => {}
        };
    }

    fn set_grenades(&mut self, full_name: &str, f: &mut Field) {
        if full_name.contains("Projectile.m_nOwnerId") {
            if self.special_ids.grenade_owner_id.is_none() {
                self.special_ids.grenade_owner_id = Some(self.id)
            }
            f.should_parse = true;
            f.prop_id = self.special_ids.grenade_owner_id.unwrap() as usize;
        };
        if full_name.contains("Projectile.CBodyComponentBaseAnimGraph.m_vec") {
            f.should_parse = true;
            match full_name.chars().last() {
                Some('X') => {
                    if self.special_ids.m_vecX_grenade.is_none() {
                        self.special_ids.m_vecX_grenade = Some(self.id);
                    }
                    f.prop_id = self.special_ids.m_vecX_grenade.unwrap() as usize;
                }
                Some('Y') => {
                    if self.special_ids.m_vecY_greande.is_none() {
                        self.special_ids.m_vecY_greande = Some(self.id);
                    }
                    f.prop_id = self.special_ids.m_vecY_greande.unwrap() as usize;
                }
                Some('Z') => {
                    if self.special_ids.m_vecZ_grenade.is_none() {
                        self.special_ids.m_vecZ_grenade = Some(self.id);
                    }
                    f.prop_id = self.special_ids.m_vecZ_grenade.unwrap() as usize;
                }
                _ => {}
            }
        }
        if full_name.contains("Projectile.CBodyComponentBaseAnimGraph.m_cell") {
            f.should_parse = true;
            match full_name.chars().last() {
                Some('X') => {
                    if self.special_ids.m_cellX_grenade.is_none() {
                        self.special_ids.m_cellX_grenade = Some(self.id);
                    }
                    f.prop_id = self.special_ids.m_cellX_grenade.unwrap() as usize;
                }
                Some('Y') => {
                    if self.special_ids.m_cellY_greande.is_none() {
                        self.special_ids.m_cellY_greande = Some(self.id);
                    }
                    f.prop_id = self.special_ids.m_cellY_greande.unwrap() as usize;
                }
                Some('Z') => {
                    if self.special_ids.m_cellZ_grenade.is_none() {
                        self.special_ids.m_cellZ_grenade = Some(self.id);
                    }
                    f.prop_id = self.special_ids.m_cellZ_grenade.unwrap() as usize;
                }
                _ => {}
            }
        }
    }

    fn traverse_fields(&mut self, fields: &mut Vec<Field>, mut path: Vec<i32>, ser_name: String) {
        for (idx, f) in fields.iter_mut().enumerate() {
            if let Some(ser) = &mut f.serializer {
                let mut tmp = path.clone();
                tmp.push(idx as i32);
                self.traverse_fields(&mut ser.fields, tmp, ser_name.clone() + "." + &ser.name)
            } else {
                let mut tmp = path.clone();
                tmp.push(idx as i32);
                let arr = PropController::vec_to_arr(&tmp);
                let full_name = ser_name.clone() + "." + &f.var_name;

                if self.is_wanted_prop(&full_name) {
                    f.should_parse = true;
                    f.prop_id = self.id as usize;
                }
                self.handle_prop(&full_name, f, arr);
                self.id_to_name.insert(self.id, full_name.clone());
                self.id_to_path.insert(self.id, arr);
                self.prop_name_to_path.insert(full_name.clone(), arr);
                self.path_to_prop_name.insert(arr, full_name);
                self.id += 1;
            }
        }
    }
    fn find_controller_prop_type(&self, name: &str) -> Option<ControllerProp> {
        match name {
            "CCSPlayerController.m_iTeamNum" => Some(ControllerProp::TeamNum),
            "CCSPlayerController.m_iszPlayerName" => Some(ControllerProp::Name),
            "CCSPlayerController.m_steamID" => Some(ControllerProp::SteamId),
            "CCSPlayerController.m_hPlayerPawn" => Some(ControllerProp::PlayerEntityId),
            _ => None,
        }
    }
    fn is_wanted_prop(&self, name: &str) -> bool {
        if self.wanted_player_props.contains(&"X".to_string())
            || self.wanted_player_props.contains(&"Y".to_string())
            || self.wanted_player_props.contains(&"Z".to_string())
        {
            if name.contains("cell") || name.contains("m_vec") {
                return true;
            }
        }
        let temp = name.split(".").collect_vec();
        let weap_prop_part = temp.last().unwrap_or(&"Whatever");
        match TYPEHM.get(weap_prop_part) {
            Some(PropType::Weapon) => return true,
            _ => {}
        };
        if name.contains("CCSTeam.m_iTeamNum")
            || name.contains("CCSPlayerPawn.m_iTeamNum")
            || name.contains("CCSPlayerController.m_iTeamNum")
            || name.contains("CCSPlayerController.m_iszPlayerName")
            || name.contains("CCSPlayerController.m_steamID")
            || name.contains("CCSPlayerController.m_hPlayerPawn")
            || name.contains("CCSPlayerController.m_bPawnIsAlive")
            || name.contains("m_hActiveWeapon")
        {
            return true;
        }
        if self.wanted_player_props.contains(&name.to_owned()) {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct PropInfo {
    pub id: u32,
    pub prop_type: Option<PropType>,
    pub prop_name: String,
    pub prop_friendly_name: String,
}

#[derive(Debug, Clone)]
pub struct DebugField {
    pub full_name: String,
    pub field: Option<Field>,
    pub decoder: Decoder,
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
    let f = Field {
        bitcount: field.bit_count(),
        var_name: serializer_msg.symbols[field.var_name_sym() as usize].clone(),
        var_type: serializer_msg.symbols[field.var_type_sym() as usize].clone(),
        send_node: serializer_msg.symbols[field.send_node_sym() as usize].clone(),
        serializer_name: ser_name,
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
        should_parse: false,
        prop_id: 0,
        is_controller_prop: false,
        controller_prop: None,
        idx: 0,
    };
    f
}

#[cfg(test)]
mod tests {
    use super::PropController;
    use crate::sendtables::Decoder::BaseDecoder;
    use crate::sendtables::Field;
    use crate::sendtables::FieldModel::FieldModelNOTSET;
    use crate::sendtables::FieldType;

    pub fn gen_default_field() -> Field {
        Field {
            var_name: "m_nRandomSeedOffset".to_string(),
            var_type: "int32".to_string(),
            send_node: "m_animationController.m_animGraphNetworkedVars".to_string(),
            serializer_name: None,
            encoder: "".to_string(),
            encode_flags: 0,
            bitcount: 0,
            low_value: 0.0,
            high_value: 0.0,
            model: FieldModelNOTSET,
            field_type: FieldType {
                base_type: "int32".to_string(),
                generic_type: None,
                pointer: false,
                count: 0,
            },
            serializer: None,
            decoder: BaseDecoder,
            base_decoder: None,
            child_decoder: None,
            should_parse: false,
            prop_id: 0,
            is_controller_prop: false,
            controller_prop: None,
            idx: 0,
        }
    }
    /*
    #[test]
    pub fn test_smoke_owner_set() {
        let mut f = gen_default_field();
        let mut pc = PropController::new(vec![], vec![]);
        pc.handle_prop("SmokeGrenadeProjectile.m_nOwnerId", &mut f);
        assert!(pc.special_ids.grenade_owner_id.is_some())
    }
    #[test]
    pub fn test_smoke_owner_not_set() {
        let mut f = gen_default_field();
        let mut pc = PropController::new(vec![], vec![]);
        pc.handle_prop("X", &mut f);
        assert!(pc.special_ids.grenade_owner_id.is_none())
    }
    */
}
