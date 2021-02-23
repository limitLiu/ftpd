mod test {
  use ftpd::config::Config;
  #[test]
  fn load_file() {
    let a = Config::new(Some("examples/ftpd.conf")).unwrap();
    println!("contents: {:?}", a);
  }
}
