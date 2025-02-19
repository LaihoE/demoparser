use std::collections::hash_map::Entry::*;
use std::collections::HashMap;
use std::str;

use winnow::prelude::*;
use winnow::{
    ascii::multispace0,
    combinator::{alt, cut_err, delimited, preceded, repeat, separated, separated_pair, terminated},
    error::{AddContext, ParserError, StrContext},
    token::{any, none_of, take, take_till},
};

#[derive(Clone)]
pub enum JsonValue {
    Str(String),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn extend_from(&mut self, other: JsonValue) {
        if let JsonValue::Object(self_hashmap) = self {
            if let JsonValue::Object(other_hashmap) = other {
                self_hashmap.extend(other_hashmap);
            }
        }
    }
}

type Stream<'i> = &'i str;

pub(crate) fn read_file(path: &str) -> JsonValue {
    let items = std::fs::read_to_string(path)
        .expect("Input json-like file")
        .trim_start_matches("\u{FEFF}")
        .replace(" n\\", "\\n");
    let (_, data) = json_like::<winnow::error::ContextError>.parse_peek(&items).expect("Parsed file data");
    data
}

fn json_like<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, StrContext>>(input: &mut Stream<'i>) -> ModalResult<JsonValue, E> {
    delimited(multispace0, key_value, multispace0)
        .parse_next(input)
        .map(|pair| JsonValue::Object(HashMap::from([pair])))
}

fn json_value<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, StrContext>>(input: &mut Stream<'i>) -> ModalResult<JsonValue, E> {
    alt((
        string.map(JsonValue::Str),
        object.map(|f| {
            let hashmap = f.into_iter().fold(HashMap::default(), |mut acc: HashMap<String, JsonValue>, (key, value)| {
                match acc.entry(key) {
                    Occupied(mut entry) => {
                        entry.get_mut().extend_from(value);
                    }
                    Vacant(entry) => {
                        entry.insert(value);
                    }
                };
                acc
            });
            JsonValue::Object(hashmap)
        }),
    ))
    .parse_next(input)
}

fn string<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, StrContext>>(input: &mut Stream<'i>) -> ModalResult<String, E> {
    preceded(
        '\"',
        cut_err(terminated(
            repeat(0.., character).fold(String::new, |mut string, c| {
                string.push(c);
                string
            }),
            '\"',
        )),
    )
    .context(StrContext::Expected("string".into()))
    .parse_next(input)
}

fn character<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> ModalResult<char, E> {
    let c = none_of('\"').parse_next(input)?;
    if c == '\\' {
        alt((
            any.verify_map(|c| {
                Some(match c {
                    '"' | '\'' | '\\' | '/' => c,
                    'b' => '\x08',
                    'f' => '\x0C',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return None,
                })
            }),
            preceded('u', unicode_escape),
            preceded('x', unicode_escape),
        ))
        .parse_next(input)
    } else {
        Ok(c)
    }
}

fn unicode_escape<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> ModalResult<char, E> {
    alt((
        u16_hex.verify(|cp| !(0xD800..0xE000).contains(cp)).map(|cp| cp as u32),
        separated_pair(u16_hex, "\\u", u16_hex)
            .verify(|(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
            .map(|(high, low)| {
                let high_ten = (high as u32) - 0xD800;
                let low_ten = (low as u32) - 0xDC00;
                (high_ten << 10) + low_ten + 0x10000
            }),
    ))
    .verify_map(std::char::from_u32)
    .parse_next(input)
}

fn u16_hex<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> ModalResult<u16, E> {
    take(4usize).verify_map(|s| u16::from_str_radix(s, 16).ok()).parse_next(input)
}

fn object<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, StrContext>>(input: &mut Stream<'i>) -> ModalResult<Vec<(String, JsonValue)>, E> {
    preceded(
        ('{', multispace_with_opt_comment),
        cut_err(terminated(
            separated(0.., key_value, multispace_with_opt_comment),
            (multispace_with_opt_comment, '}'),
        )),
    )
    .context(StrContext::Expected("object".into()))
    .parse_next(input)
}

fn key_value<'i, E: ParserError<Stream<'i>> + AddContext<Stream<'i>, StrContext>>(input: &mut Stream<'i>) -> ModalResult<(String, JsonValue), E> {
    separated_pair(string.map(|s| s.to_ascii_lowercase()), multispace0, json_value).parse_next(input)
}

fn multispace_with_opt_comment<'i, E: ParserError<Stream<'i>>>(input: &mut Stream<'i>) -> ModalResult<(), E> {
    preceded(multispace0, repeat(0.., preceded("//", terminated(take_till(0.., ['\n', '\r']), multispace0)))).parse_next(input)
}

pub struct Translation {
    hashmap: HashMap<String, JsonValue>,
}

impl TryFrom<&str> for Translation {
    type Error = std::io::Error;

    fn try_from(path: &str) -> core::result::Result<Self, Self::Error> {
        if let JsonValue::Object(mut translation) = read_file(path) {
            if let Some(JsonValue::Object(mut lang)) = translation.remove("lang") {
                if let Some(JsonValue::Object(tokens)) = lang.remove("tokens") {
                    return Ok(Translation { hashmap: tokens });
                }
            }
        }
        Err(Self::Error::new(std::io::ErrorKind::Other, "Cannot read translation tokens"))
    }
}

impl Translation {
    pub fn get(&self, key: &str) -> Option<String> {
        let key = key.trim_start_matches('#').to_ascii_lowercase();
        let token = self.hashmap.get(&key)?;
        if let JsonValue::Str(v) = token {
            Some(v.to_owned())
        } else {
            None
        }
    }
}

pub struct GameItems {
    hashmap: HashMap<String, JsonValue>,
}

impl TryFrom<&str> for GameItems {
    type Error = std::io::Error;

    fn try_from(path: &str) -> core::result::Result<Self, Self::Error> {
        if let JsonValue::Object(mut translation) = read_file(path) {
            if let Some(JsonValue::Object(items_game)) = translation.remove("items_game") {
                return Ok(GameItems { hashmap: items_game });
            }
        }
        Err(Self::Error::new(std::io::ErrorKind::Other, "Cannot read game items"))
    }
}

impl GameItems {
    pub fn get(&self, key: &str) -> Option<&HashMap<String, JsonValue>> {
        if let Some(JsonValue::Object(value)) = self.hashmap.get(key) {
            return Some(value);
        }
        None
    }
}
