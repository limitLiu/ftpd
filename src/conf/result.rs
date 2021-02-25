pub trait ResultOptionExt {
  type Out;
  fn invert(self) -> Self::Out;
}

pub use ResultOptionExt as OptionResultExt;

impl<T, E> OptionResultExt for Option<Result<T, E>> {
  type Out = Result<Option<T>, E>;

  fn invert(self) -> Self::Out {
    match self {
      Some(Err(e)) => Err(e),
      Some(Ok(v)) => Ok(Some(v)),
      None => Ok(None),
    }
  }
}

impl<T, E> ResultOptionExt for Result<Option<T>, E> {
  type Out = Option<Result<T, E>>;

  fn invert(self) -> Self::Out {
    match self {
      Ok(None) => None,
      Ok(Some(v)) => Some(Ok(v)),
      Err(e) => Some(Err(e)),
    }
  }
}

pub trait ResultIterExt {
  type Val;
  type Err;

  fn next_invert(&mut self) -> Result<Option<Self::Val>, Self::Err>;
}

impl<T, E, I: Iterator<Item = Result<T, E>>> ResultIterExt for I {
  type Val = T;
  type Err = E;

  fn next_invert(&mut self) -> Result<Option<Self::Val>, Self::Err> {
    self.next().invert()
  }
}
