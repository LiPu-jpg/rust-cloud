# RustCloud 学习指引

本文档为 Rust 学习者提供阅读 RustCloud 源码的最佳路径。

## 推荐学习顺序

### 阶段一：基础概念 (1-2 天)

按照以下顺序阅读，建立基础认知：

| 序号 | 文件 | 知识点 | 核心概念 |
|------|------|--------|----------|
| 1 | `src/error.rs` | #041 | 自定义错误类型、thiserror |
| 2 | `src/config.rs` | #021 | 结构体、Serde 序列化、环境变量 |
| 3 | `src/db/models.rs` | #023-#025 | 数据模型设计、Option、枚举 |

### 阶段二：所有权与借用 (2-3 天)

理解 Rust 最核心的概念：

| 序号 | 文件 | 知识点 | 核心概念 |
|------|------|--------|----------|
| 4 | `src/api/routes.rs` | #001 | Arc 与 RwLock 的组合 |
| 5 | `src/api/routes.rs` | #003-#005 | 路径处理、Option 方法链 |
| 6 | `src/service/storage.rs` | #006 | 路径规范化、Hash 存储 |

### 阶段三：异步编程 (2-3 天)

掌握 Tokio 异步运行时：

| 序号 | 文件 | 知识点 | 核心概念 |
|------|------|--------|----------|
| 7 | `src/main.rs` | #063 | #[tokio::main] 宏展开 |
| 8 | `src/api/routes.rs` | #061 | async fn 与 IntoResponse |
| 9 | `src/api/routes.rs` | #062 | tokio::fs vs std::fs |
| 10 | `src/service/storage.rs` | #122 | 异步文件读取与哈希 |

### 阶段四：并发与安全 (2-3 天)

理解并发编程模式：

| 序号 | 文件 | 知识点 | 核心概念 |
|------|------|--------|----------|
| 11 | `src/db/repository.rs` | #081 | Arc<Mutex> 与内部可变性 |
| 12 | `src/db/repository.rs` | #043 | 锁的作用域管理 |
| 13 | `src/watcher/file_watcher.rs` | #065 | mpsc 通道通信 |
| 14 | `src/watcher/file_watcher.rs` | #084 | 闭包与 move 关键字 |

### 阶段五：服务设计 (1-2 天)

学习服务层架构：

| 序号 | 文件 | 知识点 | 核心概念 |
|------|------|--------|----------|
| 15 | `src/service/storage.rs` | #082 | 服务配置模式 |
| 16 | `src/service/version.rs` | #083 | 组合优于继承 |
| 17 | `src/api/routes.rs` | #085 | 应用状态设计 |

### 阶段六：高级特性 (2-3 天)

深入理解 Rust 高级特性：

| 序号 | 文件 | 知识点 | 核心概念 |
|------|------|--------|----------|
| 18 | `src/service/storage.rs` | #121 | SHA-256 哈希计算 |
| 19 | `src/service/storage.rs` | #123 | 分块存储设计 |
| 20 | `src/service/sync.rs` | #126 | 同步状态机 |
| 21 | `src/watcher/file_watcher.rs` | #128 | 文件系统监控 |

### 阶段七：前端开发 (1-2 天)

学习 React + TypeScript 全栈开发：

| 序号 | 文件/目录 | 核心概念 |
|------|----------|----------|
| 22 | `web/src/api/` | Axios 配置、API 客户端封装 |
| 23 | `web/src/hooks/` | React Query 数据获取与缓存 |
| 24 | `web/src/components/` | React 组件设计、状态管理 |
| 25 | `web/src/pages/` | 页面组织、路由逻辑 |

---

## 同步原理详解

### 为什么需要同步？

想象你有三台设备：
- MacBook（工作）
- Windows PC（家里）
- Linux 服务器（备份）

你希望在任何一台设备上修改文件，其他设备自动同步。

### 核心概念

#### 1. 内容寻址存储 (Content-Addressable Storage)

```
文件内容 ──> SHA-256 哈希 ──> 存储路径
   "hello"    ──> 2cf24dba... ──> objects/2c/f24dba...
```

**好处**：
- 相同内容只存一份（去重）
- 通过 hash 判断文件是否变化

#### 2. 分块存储 (Chunked Storage)

大文件（如视频）分成小块：

```
file.mp4 (100MB)
  ├── chunk_0 (4MB) ──> hash_a
  ├── chunk_1 (4MB) ──> hash_b
  ├── chunk_2 (4MB) ──> hash_c
  └── ... (重复块只存一次)
```

**好处**：
- 增量同步（只传变化的块）
- 断点续传
- 节省带宽

#### 3. 同步状态机

```
┌─────────┐    检测变化     ┌─────────┐    传输数据     ┌───────────┐
│  Idle   │ ─────────────> │ Pending │ ─────────────> │ Syncing   │
└─────────┘                └─────────┘                └───────────┘
      ^                         │                           │
      │                         │ 成功                      │ 失败
      └─────────────────────────┴───────────────────────────┘
                          回退到 Idle
```

### 同步流程图

```
设备 A                    服务器                    设备 B
  │                         │                         │
  │─ 修改文件 ─────────────>│                         │
  │                         │─ 计算 hash ─────────────>│
  │                         │<─ 比较 hash ─────────────│
  │                         │                         │
  │<─ 同步完成 ────────────│<─ 下载变更 ────────────│
  │                         │                         │
```

### 冲突处理

当同一文件在多设备同时修改：

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| 最后写入胜出 | 以时间戳为准 | 简单场景 |
| 三路合并 | 自动合并无冲突部分 | 代码文件 |
| 手动解决 | 提示用户选择 | 重要文档 |

---

## 知识点索引

### 所有权与借用 (#001-#020)

| 编号 | 位置 | 标题 |
|------|------|------|
| #001 | `api/routes.rs:32` | Arc 与 RwLock 的组合 |
| #003 | `api/routes.rs:133` | 路径处理与安全 |
| #004 | `api/routes.rs:279` | 安全性检查：路径遍历防护 |
| #005 | `api/routes.rs:328` | Option 与错误处理 |
| #006 | `service/storage.rs:106` | hash_to_path 目录结构 |

### 类型系统 (#021-#040)

| 编号 | 位置 | 标题 |
|------|------|------|
| #021 | `config.rs:1` | 结构体与 Serde 序列化 |
| #022 | `api/routes.rs:1` | 泛型结构体与提取器 |
| #023 | `db/models.rs:1` | 数据模型设计 |
| #024 | `db/models.rs:42` | 新建记录与完整记录分离 |
| #025 | `db/models.rs:72` | 枚举与数据库映射 |

### 错误处理 (#041-#060)

| 编号 | 位置 | 标题 |
|------|------|------|
| #041 | `error.rs:1` | 自定义错误类型 |
| #042 | `api/routes.rs:162` | Result 错误处理模式 |
| #043 | `db/repository.rs:55` | async 方法与锁的作用域 |

### 异步编程 (#061-#080)

| 编号 | 位置 | 标题 |
|------|------|------|
| #061 | `api/routes.rs:123` | async fn 与 axum handler |
| #062 | `api/routes.rs:234` | 异步文件操作 |
| #063 | `main.rs:1` | 异步入口函数 |
| #064 | `main.rs:86` | 绑定地址与启动服务 |
| #065 | `watcher/file_watcher.rs:31` | 通道通信 |

### 并发安全 (#081-#100)

| 编号 | 位置 | 标题 |
|------|------|------|
| #081 | `db/repository.rs:1` | Arc<Mutex> 与内部可变性 |
| #082 | `service/storage.rs:41` | 异步服务的设计模式 |
| #083 | `service/version.rs:26` | 组合优于继承 |
| #084 | `watcher/file_watcher.rs:52` | 闭包与 move 关键字 |
| #085 | `api/routes.rs:45` | 应用状态设计 |

### 存储与哈希 (#121-#140)

| 编号 | 位置 | 标题 |
|------|------|------|
| #121 | `service/storage.rs:1` | SHA-256 哈希计算 |
| #122 | `service/storage.rs:69` | 异步文件读取与哈希 |
| #123 | `service/storage.rs:155` | 分块存储 |
| #124 | `service/version.rs:1` | 版本控制设计 |
| #125 | `service/sync.rs:1` | 同步引擎设计 |
| #126 | `service/sync.rs:29` | 同步状态机 |
| #127 | `service/sync.rs:94` | 同步计划生成 |
| #128 | `watcher/file_watcher.rs:1` | 文件系统监控 |

### 前端开发 (#201-#210)

| 编号 | 位置 | 标题 |
|------|------|------|
| #201 | `web/src/api/index.ts` | Axios 客户端封装 |
| #202 | `web/src/hooks/index.ts` | React Query 数据获取 |
| #203 | `web/src/components/FileList.tsx` | React 组件设计 |
| #204 | `web/src/App.tsx` | QueryClientProvider 全局状态 |

---

## 学习建议

### 对于 Rust 初学者

1. **先读 `error.rs`** - 最简单的模块，理解 thiserror
2. **再读 `config.rs`** - 理解 Serde 和环境变量
3. **然后读 `db/models.rs`** - 理解数据建模

### 对于有后端经验的开发者

1. **从 `main.rs` 开始** - 理解程序入口和初始化
2. **阅读 `api/routes.rs`** - 理解 Axum 路由设计
3. **深入 `service/` 目录** - 理解业务逻辑层

### 对于前端开发者

1. **从 `web/src/App.tsx`** - 理解 React Query 集成
2. **阅读 `api/index.ts`** - 理解 API 客户端封装
3. **查看 `hooks/index.ts`** - 理解数据获取模式
4. **浏览组件代码** - 理解 React 组件设计

### 对于 RCore 准备者

重点学习以下知识点：

| 知识点 | RCore 相关性 |
|--------|-------------|
| 所有权与借用 | 内核需要精确控制内存 |
| 异步编程 | 理解 Future 和调度 |
| 并发安全 | Mutex、Arc 的使用 |
| 内存布局 | repr(C)、对齐 |

---

## 测试驱动学习

### 后端测试

```bash
# 运行所有测试
cd backend && cargo test

# 运行特定测试
cargo test test_storage_compute_hash

# 显示 println 输出
cargo test -- --nocapture
```

推荐测试阅读顺序：

1. `test_repository_create_and_get_file` - 基本 CRUD
2. `test_storage_compute_hash` - 文件哈希
3. `test_api_upload_file` - HTTP API 测试
4. `test_file_watcher_detects_creation` - 异步事件测试

### 前端测试

```bash
# 进入前端目录
cd web

# 运行测试
npm test

# 构建检查
npm run build
```

---

## 常见问题

### Q: 为什么用 Arc<Mutex> 而不是直接传参？

参见知识点 #001 和 #081。

### Q: 为什么用 tokio::fs 而不是 std::fs？

参见知识点 #062。

### Q: Option 的 map 和 and_then 有什么区别？

参见知识点 #005。

### Q: 如何理解 async/await？

参见知识点 #061 和 #063。

### Q: React Query 相比 useEffect 有什么优势？

- 自动缓存和去重
- 后台刷新
- 错误重试
- 加载状态管理

参见 #202。

### Q: 为什么前端也用 TypeScript？

- 前后端类型共享（通过 API 定义）
- 编辑器智能提示
- 编译时错误检测

---

## 扩展资源

### Rust
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Axum Documentation](https://docs.rs/axum)
- [RCore Tutorial](https://rcore-os.cn/rCore-Tutorial-Book-v3/)

### 前端
- [React 文档](https://react.dev)
- [TanStack Query](https://tanstack.com/query)
- [TypeScript 手册](https://www.typescriptlang.org/docs/)
- [Vite 指南](https://vitejs.dev/guide/)

---

*本学习指引与代码中的知识点注释配合使用，效果更佳。*
