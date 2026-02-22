# AGENTS.md - RustCloud 项目指南

本项目为 AI 编码代理提供开发上下文，包含 Rust 后端和 React 前端。

## 项目结构

```
/Users/jiaoziang/project_test/
├── backend/           # Rust 后端 (Axum)
│   ├── src/
│   │   ├── api/       # HTTP API 层
│   │   ├── service/   # 业务逻辑 (storage, version, sync)
│   │   ├── db/        # 数据访问层
│   │   └── watcher/   # 文件监控
│   └── tests/         # 集成测试
├── cli/               # CLI 客户端 (rcloud)
│   └── src/
│       ├── commands/  # 子命令实现
│       ├── client.rs  # API 客户端
│       └── sync.rs    # 同步引擎
├── web/               # React 前端 (Vite + TypeScript)
│   └── src/
│       ├── pages/     # 页面组件
│       ├── components/# 通用组件
│       ├── hooks/     # React Query hooks
│       └── api/       # API 客户端
└── Makefile           # 一键启动
```

## 构建与测试命令

### 后端 (Rust)

```bash
# 构建
cargo build --release

# 开发模式运行
cargo run

# 运行所有测试
cargo test

# 运行单个测试
cargo test test_name              # 按名称过滤
cargo test --test integration_test  # 运行特定测试文件
cargo test test_api_health_check -- --nocapture  # 显示 println 输出

# 代码检查
cargo check          # 快速检查
cargo clippy         # lint 检查
cargo clippy --fix   # 自动修复
cargo fmt            # 格式化
cargo fmt -- --check # 检查格式

# 添加依赖
cargo add package_name
cargo add --dev package_name
```

### 前端 (React/TypeScript)

```bash
cd web

# 开发模式
npm run dev

# 构建
npm run build

# Lint 检查
npm run lint

# 预览生产构建
npm run preview
```

### 一键启动

```bash
make dev        # 同时启动前后端
make backend    # 仅后端 :3000
make frontend   # 仅前端 :5173
make test       # 运行所有测试
```

## 代码风格指南

### Rust (后端)

**Imports 顺序**: 标准库 → 外部 crate → 项目模块 → 父/兄弟模块

```rust
use std::path::PathBuf;
use std::sync::Arc;

use axum::{Router, routing::get};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::Result;

use super::storage::StorageService;
```

**命名规范**:
- 结构体/枚举: `CamelCase` (如 `FileRecord`, `SyncStatus`)
- 函数/变量: `snake_case` (如 `sync_files`, `file_path`)
- 常量: `SCREAMING_SNAKE_CASE` (如 `MAX_FILE_SIZE`)

**错误处理**: 使用 `Result<T>` 和 `?` 运算符，避免 `unwrap/expect`（测试除外）

**异步代码**: 使用 `tokio::fs` 而非 `std::fs`，函数名不加 `async_` 前缀

### TypeScript (前端)

**Imports 顺序**: React → 外部库 → 内部模块 → 类型

```typescript
import { useState, useEffect } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { fileApi } from '../api';
import type { FileInfo } from '../types';
```

**命名规范**:
- 组件: `PascalCase` (如 `FileManager`)
- 函数/变量: `camelCase` (如 `handleUpload`)
- 类型/接口: `PascalCase` (如 `FileInfo`)
- 常量: `SCREAMING_SNAKE_CASE`

**Hooks**: 使用 React Query 封装 API 调用，返回类型明确

## 访问地址

- 前端: http://localhost:5173
- 后端: http://127.0.0.1:3000
- API 文档: http://127.0.0.1:3000/swagger-ui

## 开发工作流

1. 开始前: `cargo check && cd web && npm run lint`
2. 提交前: `cargo fmt && cargo clippy && cargo test && cd web && npm run build`
3. 添加后端模块: 在 `backend/src/` 下创建目录，添加 `mod.rs`
4. 添加前端组件: 在 `web/src/components/` 下创建文件

## 环境变量

```bash
# 后端
RUSTCLOUD_NO_WATCH=true  # 禁用文件监控（默认启用）
RUSTCLOUD_LOG_FILE=true  # 启用文件日志
RUST_LOG=debug           # 日志级别
```

## 知识点注释

代码中包含带编号的知识点注释，格式: `// [知识点 #XXX] 标题`
- 搜索 `[知识点 #` 可查看相关代码和讲解
