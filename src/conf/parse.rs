use super::result::*;
use super::void::Void;
use std::fmt::Formatter;
use std::option::Option::{None, Some};
use std::{error, fmt, io, str};

pub struct Parser<T> {
  input: T,
}

impl<T> Parser<T> {
  fn new(input: T) -> Self {
    Parser { input }
  }
}
pub struct OkIter<I>(I);

impl<T, I: Iterator<Item = T>> Iterator for OkIter<I> {
  type Item = Result<T, Void>;

  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().map(Ok)
  }
}

impl<'a> Parser<OkIter<str::Lines<'a>>> {
  pub fn from_str(s: &'a str) -> Self {
    Self::new(OkIter(s.lines()))
  }
}

impl<R: io::BufRead> Parser<io::Lines<R>> {
  pub fn from_buf_read(r: R) -> Self {
    Self::new(r.lines())
  }
}

impl<R: io::Read> Parser<io::Lines<io::BufReader<R>>> {
  pub fn from_read(r: R) -> Self {
    Self::from_buf_read(io::BufReader::new(r))
  }
}

impl<T> Parser<T> {
  fn parse_next<E, S: AsRef<str>>(line: Option<S>) -> Result<Option<Item>, Error<E>> {
    let line = match line {
      Some(line) => line,
      None => return Ok(None),
    };
    let line = line.as_ref();
    if line.starts_with(';') || line.starts_with('#') {
      Ok(Some(Item::Comment { text: line.into() }))
    } else {
      let mut line = line.splitn(2, '=');
      if let Some(key) = line.next() {
        if let Some(value) = line.next() {
          Ok(Some(Item::Value {
            key: key.trim().into(),
            value: value.trim().into(),
          }))
        } else if key.is_empty() {
          Ok(Some(Item::Empty))
        } else {
          Err(Error::Syntax(SyntaxError::MissingEquals))
        }
      } else {
        unreachable!()
      }
    }
  }
}

impl<E, S: AsRef<str>, T: Iterator<Item = Result<S, E>>> Iterator for Parser<T> {
  type Item = Result<Item, Error<E>>;

  fn next(&mut self) -> Option<Self::Item> {
    self
      .input
      .next_invert()
      .map_err(Error::Inner)
      .and_then(|l| Self::parse_next(l))
      .invert()
  }
}

#[derive(Debug)]
pub enum Item {
  Empty,
  Value { key: String, value: String },
  Comment { text: String },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SyntaxError {
  MissingEquals,
}

impl fmt::Display for SyntaxError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match *self {
      SyntaxError::MissingEquals => write!(f, "missing '='"),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error<E> {
  Inner(E),
  Syntax(SyntaxError),
}

impl<E> From<E> for Error<E> {
  fn from(e: E) -> Self {
    Error::Inner(e)
  }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match *self {
      Error::Inner(ref e) => fmt::Display::fmt(e, f),
      Error::Syntax(e) => write!(f, "syntax error {}", e),
    }
  }
}

impl<E: error::Error> error::Error for Error<E> {}
