# RustCloud

一个用 Rust + React 实现的文件同步与存储服务，展示现代化的全栈开发实践。

## 项目结构

```
rustcloud/
├── backend/               # Rust 后端
│   ├── src/              # 源代码
│   ├── tests/            # 集成测试
│   └── Cargo.toml        # Rust 配置
├── web/                   # React 前端
│   ├── src/              # 源代码
│   └── package.json      # Node 配置
├── Makefile               # 构建脚本
├── start.sh              # 启动脚本
├── README.md             # 本文件
└── Learn.md              # 学习指引
```

## 快速开始

### 方式一：使用 Makefile（推荐）

```bash
# 安装依赖
make install

# 开发环境（同时启动前后端）
make dev

# 只启动后端
make backend

# 只启动前端
make frontend

# 构建生产版本
make build

# 运行测试
make test
```

### 方式二：使用启动脚本

```bash
# 安装依赖并启动所有服务
./start.sh all

# 或分别启动
./start.sh backend   # 只启动后端
./start.sh frontend  # 只启动前端
```

### 方式三：手动启动

```bash
# 终端 1: 启动后端
cd backend && cargo run

# 终端 2: 启动前端
cd web && npm install && npm run dev
```

### 访问服务

启动后访问：

- **前端界面**: http://localhost:5173
- **后端 API**: http://127.0.0.1:3000
- **API 文档**: http://127.0.0.1:3000/swagger-ui

## 功能特性

**后端 (Rust)**
- ✅ RESTful API (Axum)
- ✅ 文件存储 (SHA-256 去重)
- ✅ 分块存储 (>4MB 文件)
- ✅ 设备管理
- ✅ 文件监控 (notify)
- ✅ OpenAPI 文档 (Swagger UI)
- ✅ 文件大小限制
- 🔄 版本控制 (预留)
- 🔄 同步引擎 (预留)

**前端 (React + TypeScript)**
- ✅ 现代化 UI (Tailwind CSS)
- ✅ 文件管理 (浏览、上传、删除)
- ✅ 拖拽上传
- ✅ 设备管理
- ✅ 版本历史查看
- ✅ React Query 数据缓存

## 技术栈

**后端**
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Tokio](https://tokio.rs) - 异步运行时
- [Serde](https://serde.rs) - 序列化
- [Tracing](https://docs.rs/tracing) - 日志
- [Utoipa](https://github.com/juhaku/utoipa) - OpenAPI
- [Notify](https://docs.rs/notify) - 文件监控

**前端**
- [React 18](https://react.dev) - UI 框架
- [TypeScript](https://www.typescriptlang.org) - 类型安全
- [Vite](https://vitejs.dev) - 构建工具
- [Tailwind CSS](https://tailwindcss.com) - 样式
- [TanStack Query](https://tanstack.com/query) - 数据管理

## 开发命令

```bash
# 代码格式化
make fmt

# 代码检查
make lint

# 清理构建产物
make clean

# 创建必要目录
make setup
```

## 配置

通过环境变量配置后端:

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RUSTCLOUD_HOST` | 127.0.0.1 | 监听地址 |
| `RUSTCLOUD_PORT` | 3000 | 监听端口 |
| `RUSTCLOUD_STORAGE_PATH` | ./storage | 存储目录 |
| `RUSTCLOUD_MAX_FILE_SIZE` | 104857600 | 最大文件大小 (100MB) |
| `RUSTCLOUD_WATCH` | false | 启用文件监控 |
| `RUSTCLOUD_LOG_FILE` | false | 启用文件日志 |

## API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/files` | 列出文件 |
| PUT | `/api/files/{path}` | 上传文件 |
| DELETE | `/api/files/{path}` | 删除文件 |
| GET | `/api/devices` | 设备列表 |
| POST | `/api/devices` | 注册设备 |
| GET | `/api/versions` | 版本列表 |
| GET | `/api/syncs/{file_id}` | 同步状态 |

## 测试

```bash
# 运行后端测试
cd backend && cargo test

# 或
make test
```

## 学习资源

本项目包含丰富的 Rust 学习资源：

- **[Learn.md](./Learn.md)** - 详细学习指引
- **30+ 知识点注释** - 代码中标注 (#001-#140)
- **架构设计** - 分层架构、Repository 模式

涵盖主题：
- 所有权与借用
- 异步编程 (Tokio)
- 错误处理
- 并发安全
- Web 开发 (Axum)

## 开发计划

查看 [todo.md](./todo.md) 了解当前开发进度。

## License

MIT
