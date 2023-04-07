use super::sendtables::Serializer;
use crate::parsing::parser::Parser;
use crate::parsing::sendtables::Decoder::*;
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

impl Parser {
    pub fn parse_class_info(&mut self, bytes: &[u8]) {
        if !self.parse_entities {
            return;
        }
        let msg: CDemoClassInfo = Message::parse_from_bytes(&bytes).unwrap();

        for class_t in msg.classes {
            let cls_id = class_t.class_id();
            let network_name = class_t.network_name();

            let class = Class {
                class_id: cls_id,
                name: network_name.to_string(),
                serializer: self.serializers[network_name].clone(),
                history: HashSet::default(),
            };

            // self.print_struct(cls_id, &class.serializer, 0, vec![]);

            let cls_name = class.name.clone();
            self.cls_by_id
                .insert(class.class_id.try_into().unwrap(), class.clone());
            self.cls_by_name.insert(cls_name, class.clone());
        }
    }

    fn print_struct(&mut self, cls_id: i32, ser: &Serializer, depth: i32, mut path: Vec<i32>) {
        for (idx, f) in ser.fields.iter().enumerate() {
            match &f.serializer {
                Some(s) => {
                    let mut tmp = path.clone();
                    tmp.push(idx as i32);
                    self.print_struct(cls_id, &s, depth + 1, tmp)
                }
                None => {
                    let mut tmp = path.clone();
                    tmp.push(idx as i32);
                    /*
                    if tmp[0] == 11 && tmp[1] == 34 {
                        println!("{:#?}", f);
                    }
                    */
                    use crate::parsing::sendtables::FieldModel;

                    if f.model == FieldModel::FieldModelVariableArray && tmp.last().unwrap() != &0 {
                        println!("{:?}", tmp);
                        /*
                        for i in 0..512 {
                            let mut x = tmp.clone();
                            x.push(i);
                            let key = path_to_key_no_fp(x, cls_id as u32);
                            */
                        tmp.push(9999);

                        let key = path_to_key_no_fp(tmp, cls_id as u32);

                        match f.var_name.as_str() {
                            "m_PredFloatVariables" => {
                                self.cache.insert(key, (f.var_name.clone(), NoscaleDecoder));
                            }
                            "m_OwnerOnlyPredNetFloatVariables" => {
                                self.cache.insert(key, (f.var_name.clone(), NoscaleDecoder));
                            }
                            "m_OwnerOnlyPredNetVectorVariables" => {
                                self.cache.insert(
                                    key,
                                    (
                                        f.var_name.clone(),
                                        VectorSpecialDecoder(Some(Box::new(NoscaleDecoder))),
                                    ),
                                );
                            }
                            "m_PredVectorVariables" => {
                                self.cache.insert(
                                    key,
                                    (
                                        f.var_name.clone(),
                                        VectorSpecialDecoder(Some(Box::new(NoscaleDecoder))),
                                    ),
                                );
                            }

                            _ => {
                                self.cache
                                    .insert(key, (f.var_name.clone(), f.decoder.clone()));
                            }
                        }
                    } else {
                        let key = path_to_key_no_fp(tmp, cls_id as u32);
                        self.cache
                            .insert(key, (f.var_name.clone(), f.decoder.clone()));
                    }

                    // path.push(idx as i32);
                    // path_to_key_no_fp(&path);
                    // self.cache
                    // .push((f.var_name.clone(), path.clone(), f.decoder.clone()));

                    /*
                    println!(
                        "{} {} {:?} {} {:?}",
                        f.var_name, depth, path, idx, f.decoder
                    );
                    */
                }
            }
        }
    }
}

#[inline(always)]
pub fn path_to_key_no_fp(path: Vec<i32>, cls_id: u32) -> u128 {
    // println!("K {:?}", path);
    // [1, 34, 6] --> one u128 key
    // Each val gets 10 bits worth of space in the key 2**10
    // should cover max possible value
    let mut key: u128 = 0;
    for val in path {
        key |= val as u128;
        key <<= 14;
    }
    key | cls_id as u128
}
