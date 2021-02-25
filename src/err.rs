use std::io::Error;

#[derive(Debug)]
pub enum FtpdError {
  Io(Error),
}

impl From<Error> for FtpdError {
  fn from(err: Error) -> Self {
    FtpdError::Io(err)
  }
}
