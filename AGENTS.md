# AGENTS.md - rustcloud 项目指南

本文档为 AI 编码代理提供项目上下文，同时作为 Rust 学习笔记的索引。

## 项目概述

**rustcloud** 是一个 Rust 实现的文件同步与存储服务，支持实时文件监控、版本控制和多设备同步。

## 构建与测试命令

```bash
# 构建项目
cargo build

# 构建 release 版本
cargo build --release

# 运行项目
cargo run

# 运行所有测试
cargo test

# 运行单个测试（通过测试名过滤）
cargo test test_name

# 运行单个测试文件中的所有测试
cargo test --test filename

# 运行特定模块的测试
cargo test module_name::

# 显示测试输出（包括 println!）
cargo test -- --nocapture

# 代码检查（无构建）
cargo check

# 严格 lint 检查
cargo clippy

# 自动修复 clippy 警告
cargo clippy --fix

# 格式化代码
cargo fmt

# 检查格式是否符合规范
cargo fmt -- --check

# 生成文档
cargo doc --open

# 添加依赖
cargo add package_name

# 添加开发依赖
cargo add --dev package_name
```

## 项目结构

```
rustcloud/
├── Cargo.toml              # 项目配置与依赖
├── Cargo.lock              # 依赖版本锁定
├── src/
│   ├── main.rs             # 程序入口
│   ├── lib.rs              # 库入口（如果存在）
│   ├── config.rs           # 配置管理
│   ├── error.rs            # 错误类型定义
│   ├── api/                # HTTP API 层
│   │   ├── mod.rs          # 模块导出
│   │   └── routes.rs       # 路由定义
│   ├── service/            # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── storage.rs      # 文件存储服务
│   │   ├── version.rs      # 版本控制服务
│   │   └── sync.rs         # 同步引擎
│   ├── db/                 # 数据库层
│   │   ├── mod.rs
│   │   ├── models.rs       # 数据模型
│   │   └── repository.rs   # 数据访问
│   └── watcher/            # 文件监控
│       └── mod.rs
├── tests/                  # 集成测试
├── migrations/             # 数据库迁移脚本
├── web/                    # React 前端（后期）
└── config.toml             # 应用配置文件
```

## 代码风格指南

### Imports 导入规范

```rust
// 导入顺序（用空行分隔）：
// 1. 标准库
// 2. 外部 crate
// 3. 当前项目模块
// 4. 父模块和兄弟模块

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{Router, routing::get};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::Result;

use super::storage::StorageService;
```

### 格式化规范

- 使用 `cargo fmt` 自动格式化，不自定义配置
- 最大行宽：100 字符
- 缩进：4 空格
- 链式方法调用：每行一个方法

```rust
// 好的例子
let result = database
    .files()
    .filter(path.eq(&file_path))
    .order(version.desc())
    .first::<FileRecord>(conn)?;

// 函数签名换行
pub async fn sync_files(
    source: &Path,
    target: &Path,
    config: &SyncConfig,
) -> Result<SyncReport> {
    // ...
}
```

### 类型定义规范

```rust
// 使用类型别名提高可读性
pub type Result<T> = std::result::Result<T, Error>;
pub type FileId = uuid::Uuid;

// 结构体使用 CamelCase
pub struct FileMetadata {
    pub id: FileId,
    pub path: PathBuf,
    pub size: u64,
    pub hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// 枚举使用 CamelCase，变体也使用 CamelCase
pub enum SyncStatus {
    Idle,
    Syncing { progress: f32 },
    Error { message: String },
}

// 使用 thiserror 定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 结构体 | CamelCase | `FileMetadata`, `SyncConfig` |
| 枚举 | CamelCase | `SyncStatus`, `ErrorCode` |
| 函数/方法 | snake_case | `sync_files`, `get_metadata` |
| 变量 | snake_case | `file_path`, `sync_config` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_FILE_SIZE`, `DEFAULT_PORT` |
| 模块 | snake_case | `file_storage`, `sync_engine` |
| 类型参数 | 单个大写字母或简短 CamelCase | `T`, `K`, `V`, `Ctx` |

### 错误处理规范

```rust
// 使用 Result<T> 而非 unwrap/expect（测试代码除外）
// 好的做法
pub fn read_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::ConfigRead(path.to_owned(), e))?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

// 使用 ? 运算符传播错误
pub async fn process_file(path: &Path) -> Result<()> {
    let metadata = read_metadata(path)?;
    let content = read_content(path)?;
    validate_content(&content)?;
    store_file(&metadata, &content).await?;
    Ok(())
}

// 错误上下文使用 anyhow 或自定义错误
// 在应用层可以使用 anyhow::<Error>(e).context("...")
```

### 异步代码规范

```rust
// 异步函数命名不加 async_ 前缀
// 好的例子
pub async fn sync_files() -> Result<()> { }

// 而非
pub async fn async_sync_files() -> Result<()> { }

// 使用 tokio::sync 而非 std::sync 用于异步代码
use tokio::sync::{Mutex, RwLock, mpsc};

// 避免在异步代码中使用阻塞操作
// 不好
let file = std::fs::read_to_string(path)?;  // 阻塞！

// 好
let file = tokio::fs::read_to_string(path).await?;
```

## Rust 知识点注释规范

本项目在关键代码处添加**带编号的知识点注释**，格式如下：

```rust
// [知识点 #001] 所有权与借用
// ----------------------------------------
// 题目：为什么这里需要 .clone()？
// 
// 讲解：
// Rust 的所有权规则：每个值都有一个所有者，同一时刻只能有一个所有者。
// 当我们将值传递给函数时，所有权会转移，除非类型实现了 Copy trait。
// 
// String 没有实现 Copy，所以 transfer_ownership(s) 后 s 不能再使用。
// 使用 clone() 可以创建深拷贝，保留原有所有权。
// 
// 思考：什么类型实现了 Copy？什么时候应该用 clone？
// ----------------------------------------

let s = String::from("hello");
let s_clone = s.clone();  // 深拷贝
take_ownership(s);        // s 的所有权转移
println!("{}", s_clone);  // OK：s_clone 仍然有效
// println!("{}", s);     // 编译错误：s 已被移动
```

### 知识点编号规则

| 编号范围 | 知识点类别 | 示例 |
|---------|-----------|------|
| #001-#020 | 所有权与借用 | 移动、克隆、生命周期 |
| #021-#040 | 类型系统 | 泛型、Trait、类型推断 |
| #041-#060 | 错误处理 | Result、Option、?运算符 |
| #061-#080 | 异步编程 | async/await、Future、Tokio |
| #081-#100 | 并发安全 | Mutex、Arc、Send/Sync |
| #101-#120 | 宏与元编程 | derive、声明宏、过程宏 |
| #121-#140 | 内存与性能 | 内存布局、零成本抽象 |
| #141-#160 | 高级特性 | unsafe、FFI、Pin |

### RCore 相关知识点重点

为准备 RCore 项目，重点关注以下知识点：

1. **所有权与借用**：理解 Rust 内存模型，操作系统需要精确控制内存
2. **unsafe Rust**：内核代码不可避免使用 unsafe，理解其安全性边界
3. **零成本抽象**：理解编译器如何优化高层抽象
4. **Trait 与泛型**：理解静态分发与动态分发
5. **生命周期**：理解引用的有效范围，避免悬垂指针
6. **Pin 与自引用类型**：异步运行时和内核中常见
7. **内存布局**：repr(C)、packed、对齐

## 文件索引

| 文件 | 功能 | 核心知识点 |
|------|------|-----------|
| src/main.rs | 程序入口 | 异步运行时、状态管理 |
| src/error.rs | 错误定义 | thiserror、错误传播 |
| src/config.rs | 配置管理 | serde、文件 I/O |
| src/service/storage.rs | 文件存储 | 异步 I/O、路径处理 |
| src/service/sync.rs | 同步引擎 | 并发、状态机 |
| src/service/version.rs | 版本控制 | 数据库操作、哈希 |

## 开发工作流

1. **开始开发前**：运行 `cargo check` 确保编译通过
2. **提交代码前**：运行 `cargo fmt && cargo clippy && cargo test`
3. **添加新模块**：在 src/ 下创建目录，添加 mod.rs 导出
4. **学习知识点**：搜索 `[知识点 #` 查看相关代码和讲解
