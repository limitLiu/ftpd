use cmp::Ordering;
use fmt::Formatter;
use std::{cmp, error, fmt};

#[derive(Copy)]
pub enum Void {}

#[inline(always)]
pub fn unreachable(x: Void) -> ! {
  match x {}
}

impl Clone for Void {
  fn clone(&self) -> Self {
    unreachable(*self)
  }
}

impl fmt::Debug for Void {
  fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
    unreachable(*self)
  }
}
impl fmt::Display for Void {
  fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
    unreachable(*self)
  }
}

impl<T> cmp::PartialEq<T> for Void {
  fn eq(&self, _: &T) -> bool {
    unreachable(*self)
  }
}

impl<T> cmp::PartialOrd<T> for Void {
  fn partial_cmp(&self, _: &T) -> Option<Ordering> {
    unreachable(*self)
  }
}

impl error::Error for Void {}

pub trait ResultVoidExt<T>: Sized {
  fn void_unwrap(self) -> T;
}

impl<T> ResultVoidExt<T> for Result<T, Void> {
  #[inline]
  fn void_unwrap(self) -> T {
    match self {
      Ok(val) => val,
      Err(e) => unreachable(e),
    }
  }
}
pub trait ResultVoidErrExt<E>: Sized {
  fn void_unwrap_err(self) -> E;
}

impl<E> ResultVoidErrExt<E> for Result<Void, E> {
  #[inline]
  fn void_unwrap_err(self) -> E {
    match self {
      Ok(val) => unreachable(val),
      Err(e) => e,
    }
  }
}
