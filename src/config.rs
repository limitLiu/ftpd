use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::conf::{self, Error};

use super::defaults::*;
use serde_derive::Deserialize as De;

#[derive(Debug, De, Clone)]
pub struct Config {
  /// Passive mode
  #[serde(default = "pasv_enable_default")]
  pub pasv_enable: bool,
  /// Port mode
  #[serde(default = "port_enable_default")]
  pub port_enable: bool,

  /// 监听地址
  /// 如: 192.168.1.100
  #[serde(default)]
  pub listen_address: Option<String>,

  /// Unix umask 的思路所以是 8 进制
  /// 用于计算权限
  /// 777 - 077
  /// folders 700
  #[serde(default = "local_umask_default")]
  pub local_umask: String,

  /// default 21
  #[serde(default = "listen_port_default")]
  pub listen_port: u32,

  /// 最大客户端数
  #[serde(default = "max_clients_default")]
  pub max_clients: u32,

  /// 同一个 IP 最大连接数
  #[serde(default = "max_per_ip_default")]
  pub max_per_ip: u32,

  /// pasv 模式下超时
  #[serde(default = "accept_timeout_default")]
  pub accept_timeout: u32,

  /// 主动模式下超时
  #[serde(default = "connect_timeout_default")]
  pub connect_timeout: u32,

  /// 无动作自动超时
  #[serde(default = "idle_session_timeout_default")]
  pub idle_session_timeout: u32,

  /// 数据通道空闲超时
  #[serde(default = "data_connection_timeout_default")]
  pub data_connection_timeout: u32,

  /// 上传最大速
  #[serde(default = "upload_max_rate_default")]
  pub upload_max_rate: u32,

  /// 下载最大速
  #[serde(default = "download_max_rate_default")]
  pub download_max_rate: u32,
}

impl Config {
  pub fn new(path: &str) -> Result<Config, Error> {
    let mut buffer = String::new();
    let mut file = File::open(&Path::new(&path)).expect("Failed to open file");
    file.read_to_string(&mut buffer).expect("Failed to read file");
    let config = conf::from_str::<Config>(&buffer)?;
    Ok(config)
  }
}
