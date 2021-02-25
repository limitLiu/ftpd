#![allow(dead_code)]
use super::parse::{self, Item};
use serde::de::{
  self, Deserialize, DeserializeOwned, DeserializeSeed, IntoDeserializer, MapAccess, Visitor,
};
use std::fmt::Formatter;
use std::mem::replace;
use std::str::FromStr;
use std::{
  error, fmt, io,
  num::{ParseFloatError, ParseIntError},
  result,
};

pub trait NextExt {
  fn next(&mut self) -> Option<result::Result<Item, Error>>;
}

impl<E, T: Iterator<Item = result::Result<Item, E>>> NextExt for T
where
  Error: From<E>,
{
  fn next(&mut self) -> Option<result::Result<Item, Error>> {
    Iterator::next(self).map(|v| v.map_err(Into::into))
  }
}

#[derive(Debug, Clone)]
pub enum Error {
  Custom(String),
  UnexpectedEOF,
  InvalidState,
}

impl From<ParseIntError> for Error {
  fn from(e: ParseIntError) -> Self {
    Error::Custom(e.to_string())
  }
}

impl From<ParseFloatError> for Error {
  fn from(e: ParseFloatError) -> Self {
    Error::Custom(e.to_string())
  }
}

impl<E: error::Error> From<parse::Error<E>> for Error {
  fn from(e: parse::Error<E>) -> Self {
    Error::Custom(e.to_string())
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Error::Custom(msg) => write!(f, "{}", msg),
      Error::UnexpectedEOF => write!(f, "unexpected EOF"),
      Error::InvalidState => write!(f, "invalid state error"),
    }
  }
}

impl error::Error for Error {}

impl de::Error for Error {
  fn custom<T>(msg: T) -> Self
  where
    T: fmt::Display,
  {
    Error::Custom(msg.to_string())
  }
}

type Result<T> = result::Result<T, Error>;

enum PeekKind {
  Value,
}

#[derive(Debug, Copy, Clone)]
enum Next<T> {
  Init,
  EOF,
  Some(T),
}

#[derive(Debug)]
pub struct Deserializer<T> {
  input: T,
  next: Next<Result<Item>>,
}

impl<T> Deserializer<T> {
  pub fn new(input: T) -> Self {
    Deserializer {
      input,
      next: Next::Init,
    }
  }
}

impl<T: NextExt> Deserializer<T> {
  fn populate(&mut self) {
    while let Next::Init = self.next {
      let next = self.input.next();
      self.next = match next {
        Some(Ok(Item::Comment { .. })) => Next::Init,
        Some(Ok(Item::Empty)) => Next::Init,
        Some(v) => Next::Some(v),
        None => Next::EOF,
      };
    }
  }

  fn next_item(&mut self) -> Result<Item> {
    let next = match self.next {
      Next::EOF | Next::Some(Err(..)) => Next::EOF,
      _ => Next::Init,
    };
    let next = replace(&mut self.next, next);
    match next {
      Next::Some(v) => v,
      Next::EOF => Err(Error::UnexpectedEOF),
      Next::Init => unreachable!(),
    }
  }

  fn peek_item(&mut self) -> Result<Option<&mut Item>> {
    match &mut self.next {
      &mut Next::Some(Ok(ref mut v)) => Ok(Some(v)),
      e @ &mut Next::Some(Err(..)) => {
        if let Next::Some(Err(e)) = replace(e, Next::EOF) {
          Err(e)
        } else {
          unreachable!()
        }
      }
      &mut Next::EOF => Ok(None),
      &mut Next::Init => unreachable!(),
    }
  }

  fn peek_kind(&mut self) -> Result<Option<PeekKind>> {
    self.populate();
    Ok(match self.peek_item()? {
      Some(&mut Item::Value { .. }) => Some(PeekKind::Value),
      None => None,
      Some(..) => unreachable!(),
    })
  }

  fn next_key(&mut self) -> Result<String> {
    self.populate();
    match self.peek_item()? {
      Some(&mut Item::Value { ref mut key, .. }) => Ok(replace(key, "".to_string())),
      Some(..) => Err(Error::InvalidState),
      None => Err(Error::UnexpectedEOF),
    }
  }

  fn next_value(&mut self) -> Result<String> {
    self.populate();
    match self.next_item()? {
      Item::Value { value, .. } => Ok(value),
      _ => Err(Error::InvalidState),
    }
  }

  fn assert_eof(&mut self) -> Result<()> {
    self.populate();
    match self.peek_item()? {
      Some(..) => Err(Error::InvalidState),
      None => Ok(()),
    }
  }
}

impl<'de, 'a, T: NextExt> de::Deserializer<'de> for &'a mut Deserializer<T> {
  type Error = Error;

  fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_map(MapAccessTop(self))
  }

  fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_some(self)
  }

  forward_to_deserialize_any! {
    bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
    byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
    map struct identifier ignored_any enum
  }
}

impl<R: io::BufRead> Deserializer<parse::Parser<io::Lines<R>>> {
  fn from_buf_read(reader: R) -> Self {
    Deserializer::new(parse::Parser::from_buf_read(reader))
  }
}

impl<R: io::Read> Deserializer<parse::Parser<io::Lines<io::BufReader<R>>>> {
  fn from_read(reader: R) -> Self {
    Deserializer::new(parse::Parser::from_read(reader))
  }
}

struct ValueDeserializer<'a, T: 'a>(&'a mut Deserializer<T>);

impl<'de, 'a, T: NextExt> de::Deserializer<'de> for &'a mut ValueDeserializer<'a, T> {
  type Error = Error;

  fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    match self.0.peek_kind()? {
      Some(PeekKind::Value) => self.deserialize_str(visitor),
      None => Err(Error::InvalidState),
    }
  }

  fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    match self.0.peek_kind()? {
      Some(PeekKind::Value) => {
        let value = self.0.next_value()?.to_lowercase();
        visitor.visit_bool(match &*value {
          "true" | "yes" | "1" => true,
          "false" | "no" | "0" => false,
          _ => false,
        })
      }
      None => Err(Error::InvalidState),
    }
  }

  fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_i8(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_i16(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_i32(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_i64(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_u8(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_u16(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_u32(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_u64(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_f32(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_f64(FromStr::from_str(&(self.0).next_value()?)?)
  }

  fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    let value = self.0.next_value()?;
    let mut chars = value.chars();
    if let Some(c) = chars.next() {
      if chars.next().is_some() {
        visitor.visit_str(&value)
      } else {
        visitor.visit_char(c)
      }
    } else {
      visitor.visit_str(&value)
    }
  }

  fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_str(&(self.0).next_value()?)
  }

  fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_string(self.0.next_value()?)
  }

  fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    visitor.visit_some(self)
  }

  fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_unit(visitor)
  }

  fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_newtype_struct(self)
  }

  fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_tuple_struct<V: Visitor<'de>>(
    self,
    _name: &'static str,
    _len: usize,
    visitor: V,
  ) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }

  fn deserialize_struct<V>(
    self,
    _name: &'static str,
    _fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_any(visitor)
  }

  fn deserialize_enum<V>(
    self,
    _name: &'static str,
    _variants: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    match self.0.peek_kind()? {
      Some(PeekKind::Value) => visitor.visit_enum(self.0.next_value()?.into_deserializer()),
      _ => Err(Error::InvalidState),
    }
  }

  fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_str(visitor)
  }

  fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    self.deserialize_any(visitor)
  }
}

struct MapAccessTop<'a, T: NextExt + 'a>(&'a mut Deserializer<T>);

impl<'de, 'a, T: NextExt + 'a> MapAccess<'de> for MapAccessTop<'a, T> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: DeserializeSeed<'de>,
  {
    match self.0.peek_kind()? {
      Some(PeekKind::Value) => seed.deserialize(self.0.next_key()?.into_deserializer()),
      None => return Ok(None),
    }
    .map(Some)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: DeserializeSeed<'de>,
  {
    match self.0.peek_kind()? {
      Some(PeekKind::Value) => seed.deserialize(&mut ValueDeserializer(self.0)),
      None => Err(Error::UnexpectedEOF),
    }
  }
}

pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
  let mut de = Deserializer::new(parse::Parser::from_str(s.as_ref()));
  let value = Deserialize::deserialize(&mut de)?;
  de.assert_eof()?;
  Ok(value)
}

pub fn from_buf_read<R: io::BufRead, T: DeserializeOwned>(reader: R) -> Result<T> {
  let mut de = Deserializer::new(parse::Parser::from_buf_read(reader));
  let value = Deserialize::deserialize(&mut de)?;
  de.assert_eof()?;
  Ok(value)
}

pub fn from_read<R: io::Read, T: DeserializeOwned>(reader: R) -> Result<T> {
  let mut de = Deserializer::new(parse::Parser::from_read(reader));
  let value = Deserialize::deserialize(&mut de)?;
  de.assert_eof()?;
  Ok(value)
}
