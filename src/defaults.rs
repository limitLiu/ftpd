pub fn pasv_enable_default() -> bool {
  true
}

pub fn port_enable_default() -> bool {
  true
}

pub fn local_umask_default() -> String {
  "077".into()
}

pub fn listen_port_default() -> u32 {
  21
}

pub fn max_clients_default() -> u32 {
  3
}

pub fn max_per_ip_default() -> u32 {
  2
}

pub fn accept_timeout_default() -> u32 {
  60
}

pub fn connect_timeout_default() -> u32 {
  100
}

pub fn idle_session_timeout_default() -> u32 {
  300
}

pub fn data_connection_timeout_default() -> u32 {
  900
}

pub fn upload_max_rate_default() -> u32 {
  102400
}

pub fn download_max_rate_default() -> u32 {
  204800
}
