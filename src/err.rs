use std::io::Error;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum FtpdError {
  Io(Error),
  ParseInt(ParseIntError),
}

impl From<Error> for FtpdError {
  fn from(err: Error) -> Self {
    FtpdError::Io(err)
  }
}

impl From<ParseIntError> for FtpdError {
  fn from(err: ParseIntError) -> Self {
    FtpdError::ParseInt(err)
  }
}
