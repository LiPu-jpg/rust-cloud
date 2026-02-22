// [知识点 #021] 结构体与 Serde 序列化
// ----------------------------------------
// 题目：#[derive(Debug, Clone, Deserialize)] 做了什么？
//
// 讲解：
// Rust 使用派生宏自动实现 trait：
// - Debug：允许 println!("{:?}", config) 调试输出
// - Clone：允许 .clone() 深拷贝
// - Deserialize：serde 提供的反序列化能力，从 TOML/JSON 解析
//
// serde 是 Rust 生态的序列化框架，零成本抽象，编译期生成代码
// #[serde(default)] 表示字段缺失时使用默认值
// #[serde(rename = "name")] 重命名字段
//
// 思考：哪些 trait 可以 derive？哪些必须手动实现？
// ----------------------------------------

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_storage_path")]
    pub storage_path: PathBuf,

    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_storage_path() -> PathBuf {
    PathBuf::from("./storage")
}

fn default_max_file_size() -> u64 {
    100 * 1024 * 1024 // 100MB
}

impl Config {
    pub fn from_file(path: &str) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::Error::Config(format!("Failed to read config: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::error::Error::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn from_env_or_default() -> Self {
        let host = std::env::var("RUSTCLOUD_HOST").unwrap_or_else(|_| default_host());
        let port = std::env::var("RUSTCLOUD_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(default_port);
        let storage_path = std::env::var("RUSTCLOUD_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_storage_path());
        let max_file_size = std::env::var("RUSTCLOUD_MAX_FILE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_max_file_size);

        Config {
            host,
            port,
            storage_path,
            max_file_size,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
