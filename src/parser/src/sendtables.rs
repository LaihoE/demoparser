use super::read_bits::{Bitreader, DemoParserError};
use crate::decoder::QfMapper;
use crate::entities_utils::FieldPath;
use crate::parser_settings::Parser;
use crate::prop_controller::PropController;
use crate::prop_controller::WEAPON_SKIN_ID;
use crate::q_float::QuantalizedFloat;
use crate::sendtables::Decoder::*;
use crate::sendtables::FieldModel::*;
use ahash::AHashMap;
use ahash::HashMap;
use csgoproto::{
    demo::CDemoSendTables,
    netmessages::{CSVCMsg_FlattenedSerializer, ProtoFlattenedSerializerField_t},
};
use lazy_static::lazy_static;
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
    pub prop_id: u32,
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
    GameModeRulesDecoder,
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
const WEAPON_SKIN_PATH: [i32; 7] = [87, 0, 1, 0, 0, 0, 0];

impl Field {
    pub fn decoder_from_path(&self, path: &FieldPath, pos: usize, parse_inventory: bool) -> FieldInfo {
        match self.model {
            FieldModelSimple => {
                // EHHH IDK WILL HAVE TO DO FOR NOW
                // Hack as this is the only arraylike prop that works like this
                // and proper support for these would be much work so lets see if more
                // arr props are needed, else probably leave it like this.
                if path.path == WEAPON_SKIN_PATH {
                    return FieldInfo {
                        decoder: self.decoder,
                        should_parse: self.should_parse,
                        prop_id: WEAPON_SKIN_ID,
                        controller_prop: self.controller_prop,
                    };
                }
                return FieldInfo {
                    decoder: self.decoder,
                    should_parse: self.should_parse,
                    prop_id: self.prop_id as u32,
                    controller_prop: self.controller_prop,
                };
            }
            FieldModelFixedArray => {
                return FieldInfo {
                    decoder: self.decoder,
                    should_parse: self.should_parse,
                    prop_id: self.prop_id as u32,
                    controller_prop: self.controller_prop,
                };
            }
            FieldModelFixedTable => {
                if path.last == pos - 1 {
                    if self.base_decoder.is_some() {
                        return FieldInfo {
                            decoder: self.base_decoder.unwrap(),
                            should_parse: self.should_parse,
                            prop_id: self.prop_id as u32,
                            controller_prop: self.controller_prop,
                        };
                    }
                    return FieldInfo {
                        decoder: self.decoder,
                        should_parse: self.should_parse,
                        prop_id: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                } else {
                    match &self.serializer {
                        Some(ser) => {
                            return ser.find_decoder(path, pos, parse_inventory);
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
                        prop_id: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                } else {
                    return FieldInfo {
                        decoder: self.base_decoder.unwrap(),
                        should_parse: self.should_parse,
                        prop_id: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                }
            }
            FieldModelVariableTable => {
                if path.last >= pos + 1 {
                    match &self.serializer {
                        Some(ser) => {
                            return ser.find_decoder(path, pos + 1, parse_inventory);
                        }
                        None => panic!("no serializer for path"),
                    }
                } else {
                    return FieldInfo {
                        decoder: self.base_decoder.unwrap(),
                        should_parse: self.should_parse,
                        prop_id: self.prop_id as u32,
                        controller_prop: self.controller_prop,
                    };
                }
            }
            _ => panic!("HUH"),
        }
    }
    pub fn debug_decoder_from_path(&self, path: &FieldPath, pos: usize, prop_name: String) -> DebugField {
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
                            return ser.debug_find_decoder(path, pos + 1, prop_name + "." + &ser.name);
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
                    "m_OwnerOnlyPredNetVectorVariables" => self.child_decoder = Some(VectorNoscaleDecoder),
                    "m_PredVectorVariables" => self.child_decoder = Some(VectorNoscaleDecoder),
                    _ => {
                        self.child_decoder = match BASETYPE_DECODERS.get(&self.field_type.generic_type.clone().unwrap().base_type)
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
const FLASH_AMMO_PATH: [i32; 7] = [86, 2, 14, 0, 0, 0, 0];
use crate::prop_controller::GRENADE_AMMO_ID;
use crate::prop_controller::MY_WEAPONS_OFFSET;

impl Serializer {
    pub fn find_decoder(&self, path: &FieldPath, pos: usize, parse_inventory: bool) -> FieldInfo {
        // Edge case for now...
        if parse_inventory {
            if let Some(info) = self.find_inventory_info(path) {
                return info;
            }
        }
        self.fields[path.path[pos] as usize].decoder_from_path(path, pos + 1, parse_inventory)
    }
    pub fn debug_find_decoder(&self, path: &FieldPath, pos: usize, prop_name: String) -> DebugField {
        let idx = path.path[pos];
        let f = &self.fields[idx as usize];
        f.debug_decoder_from_path(path, pos + 1, prop_name)
    }
    fn find_inventory_info(&self, path: &FieldPath) -> Option<FieldInfo> {
        if path.path == FLASH_AMMO_PATH && self.name == "CCSPlayerPawn" {
            return Some(FieldInfo {
                controller_prop: None,
                decoder: UnsignedDecoder,
                should_parse: true,
                prop_id: GRENADE_AMMO_ID,
            });
        }
        if path.path[0] == 86 && path.path[1] == 0 && self.name == "CCSPlayerPawn" && path.last == 2 {
            return Some(FieldInfo {
                controller_prop: None,
                decoder: UnsignedDecoder,
                should_parse: true,
                prop_id: MY_WEAPONS_OFFSET + path.path[2] as u32,
            });
        }
        None
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
        &mut self,
        tables: CDemoSendTables,
    ) -> Result<(AHashMap<String, Serializer>, QfMapper, PropController), DemoParserError> {
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint()?;
        let bytes = bitreader.read_n_bytes(n_bytes as usize)?;
        let serializer_msg: CSVCMsg_FlattenedSerializer = Message::parse_from_bytes(&bytes).unwrap();
        let mut serializers: AHashMap<String, Serializer> = AHashMap::default();
        let mut qf_mapper = QfMapper {
            idx: 0,
            map: AHashMap::default(),
        };
        let mut fields: HashMap<i32, Field> = HashMap::default();
        let mut prop_controller = PropController::new(
            self.wanted_player_props.clone(),
            self.wanted_other_props.clone(),
            self.real_name_to_og_name.clone(),
        );
        for serializer in serializer_msg.serializers.iter() {
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
                                if field.field_type.pointer || POINTER_TYPES.contains(&field.field_type.base_type.as_str()) {
                                    field.find_decoder(FieldModelFixedTable, &mut qf_mapper)
                                } else {
                                    field.find_decoder(FieldModelVariableTable, &mut qf_mapper)
                                }
                            }
                            None => {
                                if field.field_type.count > 0 && field.field_type.base_type != "char" {
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
                        if field.var_name == "m_pGameModeRules" {
                            field.decoder = GameModeRulesDecoder
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
                || my_serializer.name.contains("Knife")
                || my_serializer.name.contains("CDEagle")
                || my_serializer.name.contains("Rules")
                || my_serializer.name.contains("C4")
                || my_serializer.name.contains("Grenade")
                || my_serializer.name.contains("Flash")
                || my_serializer.name.contains("Molo")
                || my_serializer.name.contains("Inc")
                || my_serializer.name.contains("Infer")
            {
                prop_controller.find_prop_name_paths(&mut my_serializer);
            }
            serializers.insert(my_serializer.name.clone(), my_serializer);
        }
        prop_controller.set_custom_propinfos();
        Ok((serializers, qf_mapper, prop_controller))
    }
}

#[derive(Debug, Clone)]
pub struct DebugField {
    pub full_name: String,
    pub field: Option<Field>,
    pub decoder: Decoder,
}
#[derive(Debug, Clone)]
pub struct DebugFieldAndPath {
    pub field: DebugField,
    pub path: [i32; 7],
}

fn field_from_msg(field: &ProtoFlattenedSerializerField_t, serializer_msg: &CSVCMsg_FlattenedSerializer) -> Field {
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
