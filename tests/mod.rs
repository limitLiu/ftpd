mod test {
  use ftpd::config::Config;
  #[test]
  fn load_file() {
    let a = Config::new("examples/ftpd.conf").unwrap();
    assert!(a.listen_address.is_some());
  }
}
