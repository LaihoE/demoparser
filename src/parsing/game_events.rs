use crate::Parser;
use csgoproto::networkbasetypes::csvcmsg_game_event::Key_t;
use csgoproto::networkbasetypes::CSVCMsg_GameEvent;

fn parse_key(key: &Key_t) -> Option<KeyData> {
    match key.type_() {
        1 => Some(KeyData::Str(key.val_string().to_owned())),
        2 => Some(KeyData::Float(key.val_float())),
        3 => Some(KeyData::Long(key.val_long())),
        4 => Some(KeyData::Short(key.val_short().try_into().unwrap())),
        5 => Some(KeyData::Byte(key.val_byte().try_into().unwrap())),
        6 => Some(KeyData::Bool(key.val_bool())),
        7 => Some(KeyData::Uint64(key.val_uint64())),
        _ => {
            //println!("Unknown key type for game event key: {}", key.type_());
            return None;
        }
    }
}

#[derive(Debug)]
pub enum KeyData {
    Str(String),
    Float(f32),
    Long(i32),
    Short(i16),
    Byte(u8),
    Bool(bool),
    Uint64(u64),
}

impl Parser {
    pub fn parse_event(&self, event: CSVCMsg_GameEvent) {
        let ge_list = match &self.ge_list {
            Some(gel) => gel,
            None => panic!("Game event before descriptor list was parsed."),
        };
        let event_desc = &ge_list[&event.eventid()];

        println!("{}", event_desc.name());

        for i in 0..event.keys.len() {
            let ge = &event.keys[i];
            let desc = &event_desc.keys[i];
            let val = parse_key(ge);
            println!(">> {} {:?}", desc.name(), val);
        }
    }
}
