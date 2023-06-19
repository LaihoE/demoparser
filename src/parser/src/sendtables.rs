use super::read_bits::{Bitreader, DemoParserError};
use crate::collect_data::PropType;
use crate::collect_data::TYPEHM;
use crate::demo_searcher::DemoSearcher;
use crate::entities_utils::FieldPath;
use crate::parser_settings::Parser;
use crate::q_float::QuantalizedFloat;
use crate::sendtables::Decoder::*;
use crate::sendtables::FieldModel::*;
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
    pub serializer: Option<Serializer>,
    pub decoder: Decoder,
    pub base_decoder: Option<Decoder>,
    pub child_decoder: Option<Decoder>,

    pub should_parse: bool,
    pub df_pos: usize,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decoder {
    QuantalizedFloatDecoder(QuantalizedFloat),
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
                    df_pos: self.df_pos as u32,
                    controller_prop: self.controller_prop,
                };
            }
            FieldModelFixedArray => {
                return FieldInfo {
                    decoder: self.decoder,
                    should_parse: self.should_parse,
                    df_pos: self.df_pos as u32,
                    controller_prop: self.controller_prop,
                }
            }
            FieldModelFixedTable => {
                if path.last == pos - 1 {
                    if self.base_decoder.is_some() {
                        return FieldInfo {
                            decoder: self.base_decoder.unwrap(),
                            should_parse: self.should_parse,
                            df_pos: self.df_pos as u32,
                            controller_prop: self.controller_prop,
                        };
                    }
                    return FieldInfo {
                        decoder: self.decoder,
                        should_parse: self.should_parse,
                        df_pos: self.df_pos as u32,
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
                        df_pos: self.df_pos as u32,
                        controller_prop: self.controller_prop,
                    };
                } else {
                    return FieldInfo {
                        decoder: self.base_decoder.unwrap(),
                        should_parse: self.should_parse,
                        df_pos: self.df_pos as u32,
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
                        df_pos: self.df_pos as u32,
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

    pub fn find_decoder(&mut self, model: FieldModel) {
        self.model = model.clone();
        match model {
            FieldModelFixedArray => self.decoder = self.match_decoder(),
            FieldModelSimple => self.decoder = self.match_decoder(),
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
    pub fn match_decoder(&self) -> Decoder {
        if self.var_name == "m_iClip1" {
            return Decoder::AmmoDecoder;
        }
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
    pub fn find_float_type(&self) -> Decoder {
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

impl DemoSearcher {
    // This part is so insanely complicated. There are multiple versions of each serializer and
    // each serializer is this huge nested struct.
    pub fn parse_sendtable(&mut self, tables: CDemoSendTables) -> Result<(), DemoParserError> {
        let mut bitreader = Bitreader::new(tables.data());
        let n_bytes = bitreader.read_varint()?;

        let bytes = bitreader.read_n_bytes(n_bytes as usize)?;
        let serializer_msg: CSVCMsg_FlattenedSerializer =
            Message::parse_from_bytes(&bytes).unwrap();

        let mut fields: HashMap<i32, Field> = HashMap::default();
        for serializer in &serializer_msg.serializers {
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
                            Some(_) => {
                                if field.field_type.pointer
                                    || POINTER_TYPES.contains(&field.field_type.base_type.as_str())
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
                        fields.insert(*idx, field.clone());
                        my_serializer.fields.push(field);
                    }
                }
            }
            if my_serializer.name.contains("Player")
                || my_serializer.name.contains("Team")
                || my_serializer.name.contains("Weapon")
                || my_serializer.name.contains("AK")
            {
                self.find_prop_name_paths(&mut my_serializer);
            }
            self.serializers
                .insert(my_serializer.name.clone(), my_serializer);
        }

        self.prop_infos.push(PropInfo {
            id: 9999997,
            prop_type: None,
            prop_name: "name".to_string(),
        });
        self.prop_infos.push(PropInfo {
            id: 9999998,
            prop_type: None,
            prop_name: "steamid".to_string(),
        });
        self.prop_infos.push(PropInfo {
            id: 9999999,
            prop_type: None,
            prop_name: "tick".to_string(),
        });
        Ok(())
    }
    pub fn find_prop_name_paths(&mut self, ser: &mut Serializer) {
        // Finds mapping from name to path.
        // Example: "m_iHealth" => [4, 0, 0, 0, 0, 0, 0]
        self.traverse_fields(&mut ser.fields, vec![], ser.name.clone())
    }
    pub fn traverse_fields(&mut self, fields: &mut Vec<Field>, path: Vec<i32>, ser_name: String) {
        for (idx, f) in fields.iter_mut().enumerate() {
            if let Some(ser) = &mut f.serializer {
                let mut tmp = path.clone();
                tmp.push(idx as i32);
                self.traverse_fields(&mut ser.fields, tmp, ser_name.clone() + "." + &ser.name)
            } else {
                let mut tmp = path.clone();
                tmp.push(idx as i32);

                let mut arr = [0, 0, 0, 0, 0, 0, 0];
                for (idx, val) in tmp.iter().enumerate() {
                    arr[idx] = *val;
                }
                let full_name = ser_name.clone() + "." + &f.var_name;

                if self.is_wanted_prop(&full_name) {
                    f.should_parse = true;
                    self.wanted_prop_paths.insert(arr);
                    if full_name.contains("Controller") {
                        f.is_controller_prop = true;
                        f.controller_prop = self.find_controller_prop_type(&full_name);
                    }
                    f.df_pos = self.id as usize;
                }
                let split_at_dot: Vec<&str> = full_name.split(".").collect();
                let weap_prop = split_at_dot.last().unwrap();

                if self.wanted_player_props.contains(&full_name) {
                    self.wanted_prop_ids.push(self.id);
                    match TYPEHM.get(&full_name) {
                        Some(t) => self.prop_infos.push(PropInfo {
                            id: self.id,
                            prop_type: Some(t.clone()),
                            prop_name: full_name.clone(),
                        }),
                        None => {
                            self.prop_infos.push(
                                PropInfo {
                                    id: self.id,
                                    prop_type: None,
                                    prop_name: full_name.clone(),
                                }
                                .clone(),
                            );
                        }
                    }
                }
                if self.wanted_player_props.contains(&weap_prop.to_string()) {
                    self.wanted_prop_ids.push(self.id);
                    self.prop_name_to_path.insert(full_name.clone(), arr);
                    self.path_to_prop_name.insert(arr, full_name.clone());

                    let id = match self.name_to_id.get(weap_prop.to_owned()) {
                        Some(i) => {}
                        None => match TYPEHM.get(&weap_prop) {
                            Some(t) => self.prop_infos.push(PropInfo {
                                id: self.id,
                                prop_type: Some(t.clone()),
                                prop_name: weap_prop.to_string(),
                            }),
                            None => {
                                self.prop_infos.push(
                                    PropInfo {
                                        id: self.id,
                                        prop_type: None,
                                        prop_name: weap_prop.to_string(),
                                    }
                                    .clone(),
                                );
                            }
                        },
                    };
                    self.name_to_id.insert(weap_prop.to_string(), self.id);
                    self.id_to_path.insert(self.id, arr);
                    self.id += 1;
                    continue;
                }

                match full_name.as_str() {
                    "CCSTeam.m_iTeamNum" => self.controller_ids.team_team_num = Some(self.id),
                    "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellX" => {
                        self.controller_ids.cell_x_player = Some(self.id);
                    }
                    "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecX" => {
                        self.controller_ids.cell_x_offset_player = Some(self.id);
                        if self.wanted_player_props.contains(&"X".to_string()) {
                            self.prop_infos.push(PropInfo {
                                id: 9999907,
                                prop_type: Some(PropType::Custom),
                                prop_name: "X".to_string(),
                            });
                        }
                    }
                    "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellY" => {
                        self.controller_ids.cell_y_player = Some(self.id);
                    }
                    "CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecY" => {
                        self.controller_ids.cell_y_offset_player = Some(self.id);
                        if self.wanted_player_props.contains(&"Y".to_string()) {
                            self.prop_infos.push(PropInfo {
                                id: 9999902,
                                prop_type: Some(PropType::Custom),
                                prop_name: "Y".to_string(),
                            });
                        }
                    }
                    "CCSPlayerPawn.m_iTeamNum" => {
                        self.controller_ids.player_team_pointer = Some(self.id)
                    }
                    "CBasePlayerWeapon.m_nOwnerId" => {
                        self.controller_ids.weapon_owner_pointer = Some(self.id)
                    }
                    "CCSPlayerPawn.CCSPlayer_WeaponServices.m_hActiveWeapon" => {
                        self.controller_ids.active_weapon = Some(self.id)
                    }
                    "CCSPlayerController.m_iTeamNum" => self.controller_ids.teamnum = Some(self.id),
                    "CCSPlayerController.m_iszPlayerName" => {
                        self.controller_ids.player_name = Some(self.id)
                    }
                    "CCSPlayerController.m_steamID" => self.controller_ids.steamid = Some(self.id),
                    "CCSPlayerController.m_hPlayerPawn" => {
                        self.controller_ids.player_pawn = Some(self.id)
                    }
                    _ => {}
                };
                self.id_to_path.insert(self.id, arr);
                self.id += 1;

                self.prop_name_to_path.insert(full_name.clone(), arr);
                self.path_to_prop_name.insert(arr, full_name);
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
        df_pos: 0,
        is_controller_prop: false,
        controller_prop: None,
        idx: 0,
    };
    f
}
