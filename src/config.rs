use super::err::FtpdError;
use super::tool::Trim;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Config {
  /// Passive mode
  pub pasv_enable: bool,
  /// Port mode
  pub port_enable: bool,

  /// 监听地址
  /// 如: 192.168.1.100
  pub listen_address: Option<String>,

  /// Unix umask 的思路所以是 8 进制
  /// 用于计算权限
  /// 777 - 077
  /// folders 700
  pub local_umask: u32,

  /// default 21
  pub listen_port: u32,

  /// 最大客户端数
  pub max_clients: u32,

  /// 同一个 IP 最大连接数
  pub max_per_ip: u32,

  /// pasv 模式下超时
  pub accept_timeout: u32,
  /// 主动模式下超时
  pub connect_timeout: u32,
  /// 无动作自动超时
  pub idle_session_timeout: u32,
  /// 数据通道空闲超时
  pub data_connection_timeout: u32,
  /// 上传最大速
  pub upload_max_rate: u32,
  /// 下载最大速
  pub download_max_rate: u32,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      pasv_enable: true,
      port_enable: true,
      listen_address: None,
      local_umask: 077,
      listen_port: 21,
      max_clients: 3,
      max_per_ip: 2,
      accept_timeout: 60,
      connect_timeout: 100,
      idle_session_timeout: 300,
      data_connection_timeout: 900,
      upload_max_rate: 102400,
      download_max_rate: 204800,
    }
  }
}

impl Config {
  pub fn new(path: Option<&str>) -> Result<Config, FtpdError> {
    let mut config = Config::default();
    if let Some(path) = path {
      let path = Path::new(&path);
      let file = BufReader::new(File::open(&path)?);
      for line in file.lines() {
        let line = line?.trim_all();
        let strings: Vec<&str> = line.split('=').collect();
        let key = strings[0];
        let value = strings[1];
        match key {
          "pasv_enable" => {
            config.pasv_enable = Self::boolean(value);
          }
          "port_enable" => {
            config.port_enable = Self::boolean(value);
          }
          "listen_address" => {
            config.listen_address = if value.len() > 0 {
              Some(value.to_string())
            } else {
              None
            };
          }
          "local_umask" => {
            config.local_umask = value.parse::<u32>()?;
          }
          "listen_port" => {
            config.listen_port = value.parse::<u32>()?;
          }
          "max_clients" => {
            config.max_clients = value.parse::<u32>()?;
          }
          "max_per_ip" => {
            config.max_per_ip = value.parse::<u32>()?;
          }
          "accept_timeout" => {
            config.accept_timeout = value.parse::<u32>()?;
          }
          "connect_timeout" => {
            config.connect_timeout = value.parse::<u32>()?;
          }
          "idle_session_timeout" => {
            config.idle_session_timeout = value.parse::<u32>()?;
          }
          "data_connection_timeout" => {
            config.data_connection_timeout = value.parse::<u32>()?;
          }
          "upload_max_rate" => {
            config.upload_max_rate = value.parse::<u32>()?;
          }
          "download_max_rate" => {
            config.download_max_rate = value.parse::<u32>()?;
          }
          _ => {}
        };
      }
    }

    Ok(config)
  }

  fn boolean(value: &str) -> bool {
    if value == "TRUE" || value == "YES" || value == "1" {
      return true;
    } else if value == "FALSE" || value == "NO" || value == "0" {
      return false;
    }
    return false;
  }
}
