use serde;
use std::fmt::{self, Display, Formatter};
use super::de::Error as DeError;

#[derive(Debug, Clone)]
pub enum Error {
  Custom(String),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Error::Custom(msg) => write!(f, "{}", msg),
    }
  }
}

impl From<DeError> for Error {
  fn from(e: DeError) -> Self {
    Error::Custom(e.to_string())
  }
}

impl ::std::error::Error for Error {}

impl serde::de::Error for Error {
  fn custom<T>(msg: T) -> Self
  where
    T: Display,
  {
    Error::Custom(msg.to_string())
  }
}
