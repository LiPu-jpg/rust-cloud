# 代码改进建议清单（已更新 - GLM修改后）

**文件**: `kimiwrite_only/improvements.md`  
**状态**: GLM 修改后更新  
**更新时间**: 2026-02-22

---

## 已完成的改进 ✅

### 1. 文件大小限制功能

**状态**: ✅ 已完成（GLM 实现）  
**修改文件**: `src/api/routes.rs`

**实现内容**:
- AppData 新增 `max_file_size: u64` 字段
- `upload_file` handler 添加大小校验逻辑
- 超大文件返回 413 Payload Too Large

**新增测试**: `test_api_upload_file_within_limit`

### 2. 文件监控测试

**状态**: ✅ 已完成（GLM 实现）  
**修改文件**: `tests/integration_test.rs`

**实现内容**:
- 添加 `test_file_watcher_detects_creation` 测试
- 验证文件创建事件检测
- 使用临时目录和原子标志进行异步测试

### 3. OpenAPI 准备

**状态**: ✅ 已引入依赖  
**修改文件**: `Cargo.toml`, `src/api/routes.rs`

**实现内容**:
- 添加 `utoipa` 依赖
- 为 `FileInfo` 和 `ApiResponse` 添加 `ToSchema` derive

---

## 待处理事项 📋

### P1 - 编译警告处理

**当前状态**: 21个警告  
**类型**: 主要是"从未使用"的代码

**建议处理方式**:

```rust
// 1. 对于预留的功能模块，添加标记
#[allow(dead_code)]
pub struct SyncEngine { ... }

// 2. 对于将来要使用的代码，添加 TODO
// TODO: 集成到主流程（将在 Phase 2 实现）
pub struct VersionService { ... }

// 3. 对于真正不需要的代码，直接删除
```

**具体需要处理的文件**:
- [ ] `src/api/routes.rs` - `UploadRequest`, `create_router`
- [ ] `src/config.rs` - `from_file` 方法
- [ ] `src/db/models.rs` - `NewSyncRecord`, `SyncStatus::as_str`
- [ ] `src/db/repository.rs` - 多个未使用的方法
- [ ] `src/service/sync.rs` - `SyncEngine` 整体
- [ ] `src/service/version.rs` - `VersionService` 整体
- [ ] `src/service/storage.rs` - 多个未使用的方法
- [ ] `src/watcher/file_watcher.rs` - `Renamed` 事件, `stop` 方法

### P2 - 功能集成

#### 2.1 集成 VersionService

**说明**: 版本控制服务代码完整，但未在 API 中使用

**建议 API 端点**:
```
POST   /api/files/{path}/versions      # 创建新版本
GET    /api/files/{path}/versions      # 获取版本列表
GET    /api/files/{path}/versions/{id} # 获取特定版本
POST   /api/files/{path}/rollback      # 回滚到指定版本
```

#### 2.2 集成 SyncEngine

**说明**: 同步引擎代码完整，但未在 API 中使用

**建议 API 端点**:
```
POST   /api/sync/plan                   # 生成同步计划
POST   /api/sync/execute                # 执行同步
GET    /api/sync/status/{file_id}       # 查看同步状态
```

#### 2.3 完成 OpenAPI 集成

**说明**: utoipa 已引入，但需要完成配置

**示例代码**:
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        list_files,
        upload_file,
        // ... 其他 handler
    ),
    components(
        schemas(FileInfo, ApiResponse, RegisterDeviceRequest)
    ),
    tags(
        (name = "files", description = "文件管理 API"),
        (name = "devices", description = "设备管理 API"),
    )
)]
pub struct ApiDoc;

// 在 main.rs 中添加文档路由
let app = api::create_router_with_services(...)
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
```

### P3 - 可选增强

#### 3.1 配置文件热重载

**说明**: 使用已引入的 `notify` 库监控配置文件

```rust
pub async fn watch_config<F>(path: &Path, mut callback: F) -> Result<()>
where
    F: FnMut(Config) + Send + 'static,
{
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(_) = res {
            let _ = tx.blocking_send(());
        }
    })?;
    
    watcher.watch(path, notify::RecursiveMode::NonRecursive)?;
    
    tokio::spawn(async move {
        while rx.recv().await.is_some() {
            if let Ok(config) = Config::from_file(path.to_str().unwrap()) {
                callback(config);
            }
        }
    });
    
    Ok(())
}
```

#### 3.2 数据库迁移（JSON → SQLite）

**说明**: 当前 JSON 文件存储适合开发，但生产环境需要真实数据库

**建议依赖**:
```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "migrate"] }
```

---

## 实施追踪

| 编号 | 改进项 | 状态 | 负责人 | 备注 |
|------|--------|------|--------|------|
| 1 | 修复编译警告 | ⬜ 待办 | - | 21个警告待处理 |
| 2 | max_file_size 校验 | ✅ 完成 | GLM | 已测试 |
| 3 | SyncEngine 集成 | ⬜ 待办 | - | 代码已就绪 |
| 4 | 文件监控测试 | ✅ 完成 | GLM | 13个测试通过 |
| 5 | OpenAPI 文档 | ⬜ 待办 | - | utoipa 已引入 |
| 6 | VersionService 集成 | ⬜ 待办 | - | 代码已就绪 |
| 7 | 配置热重载 | ⬜ 待办 | - | 可选功能 |
| 8 | SQLite 迁移 | ⬜ 待办 | - | 生产准备 |

---

## GLM 贡献记录

### 修改的文件统计
```
Cargo.lock                  | 756 +++++++-
Cargo.toml                  |  10 ++
src/api/mod.rs              |   3 +-
src/api/routes.rs           | 428 +++++++++
src/db/mod.rs               |   5 +
src/db/models.rs            | 152 ++++
src/db/repository.rs        | 242 +++++
src/main.rs                 | 144 +++++
src/service/mod.rs          |   2 +
src/service/storage.rs      | 263 +++++
src/service/sync.rs         | 200 +++++
src/service/version.rs      | 127 ++++
src/watcher/file_watcher.rs | 208 ++++++
tests/integration_test.rs   |  40 +++++
```

### 新增功能
1. ✅ 文件大小限制校验
2. ✅ 文件监控功能测试
3. ✅ utoipa OpenAPI 支持

### 新增测试
1. ✅ `test_api_upload_file_within_limit`
2. ✅ `test_file_watcher_detects_creation`

---

*最后更新: 2026-02-22（GLM 修改后）*
