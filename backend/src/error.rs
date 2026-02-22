// [知识点 #041] 自定义错误类型
// ----------------------------------------
// 题目：为什么要自定义错误类型而不是用 anyhow？
//
// 讲解：
// anyhow 适合应用层快速开发，但库代码应该定义自己的错误类型：
// 1. 提供清晰的错误信息给调用者
// 2. 允许调用者针对不同错误类型做不同处理
// 3. thiserror 自动实现 std::error::Error trait
//
// #[error(...)] 宏定义错误显示信息
// #[from] 自动实现 From trait，允许用 ? 转换
//
// 思考：什么时候用 anyhow，什么时候自定义错误？
// ----------------------------------------

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    #[error("File already exists: {0}")]
    AlreadyExists(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
