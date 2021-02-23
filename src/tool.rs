pub trait Trim {
  fn trim_all(&mut self) -> String;
}

impl Trim for String {
  fn trim_all(&mut self) -> String {
    self.chars().filter(|c| !c.is_whitespace()).collect::<String>()
  }
}
