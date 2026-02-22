// [知识点 #022] 泛型结构体与提取器
// ----------------------------------------
// 题目：Json<T> 是如何工作的？
//
// 讲解：
// Axum 的提取器模式：
// - Json<T> 实现了 FromRequest trait
// - 当请求到达时，自动解析 body 为 T 类型
// - 要求 T: DeserializeOwned（可以从字节反序列化）
//
// 这是 Rust 泛型的典型应用：编译期多态
// 编译器会为每个具体类型生成专门的代码，零运行时开销
//
// 思考：提取器模式与中间件有什么区别？
// ----------------------------------------

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::config::Config;
use crate::db::{NewDeviceRecord, Repository};
use crate::error::Error;
use crate::service::storage::{StorageConfig, StorageService};

// [知识点 #001] Arc 与 RwLock 的组合
// ----------------------------------------
// 题目：为什么用 Arc 而不是直接用 T？
//
// 讲解：
// Axum 的 handler 是异步的，可能被多个请求并发调用。
// 需要满足：
// 1. 多所有权：Arc（Atomic Reference Count）允许多个所有者
// 2. 内部可变性：通过 &self 调用 async 方法修改数据
// 3. 线程安全：Arc 是 Send + Sync
//
// Repository 使用内部 Arc<Mutex>，所以这里只需要 Arc
//
// 思考：什么时候需要在外层再加 RwLock？
// ----------------------------------------
pub type AppState = Arc<AppData>;

// [知识点 #085] 应用状态设计
// ----------------------------------------
// 题目：AppData 应该包含哪些内容？
//
// 讲解：
// 应用状态是所有 handler 共享的数据：
// - storage_path: 文件存储根目录
// - repository: 数据库访问层
// - storage: 文件存储服务
// - max_file_size: 最大文件大小限制
//
// 所有服务使用 Arc 共享，避免重复创建
//
// 思考：如何处理需要动态更新的配置？
// ----------------------------------------
pub struct AppData {
    pub storage_path: std::path::PathBuf,
    pub repository: Repository,
    pub storage: StorageService,
    pub max_file_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub hash: Option<String>,
    pub version: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl ApiResponse {
    pub fn success<T: Serialize>(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(serde_json::to_value(data).unwrap_or(serde_json::Value::Null)),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub name: String,
}

// TODO: 未来用于接收二进制文件上传
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub content: String,
}

// [知识点 #061] async fn 与 axum handler
// ----------------------------------------
// 题目：async fn 的返回值如何被 axum 处理？
//
// 讲解：
// axum handler 可以返回任何实现了 IntoResponse 的类型。
// (StatusCode, Json<T>) 元组也实现了 IntoResponse。
//
// async 关键字将函数转换为返回 Future 的函数。
// .await 在运行时挂起当前任务，等待异步操作完成。
// tokio 运行时负责调度和执行这些 Future。
//
// 思考：async 函数的调用和同步函数有什么区别？
// ----------------------------------------
pub async fn create_router(config: Config) -> Router {
    let db_path = config.storage_path.join("db.json");

    let repository = Repository::new(db_path)
        .await
        .expect("Failed to init repository");
    let storage = StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 4 * 1024 * 1024,
    });

    create_router_with_services(config, Arc::new(repository), Arc::new(storage)).await
}

// [知识点 #133] 依赖注入模式
// ----------------------------------------
// 题目：为什么要提供两个 create_router 函数？
//
// 讲解：
// - create_router: 简单场景，自动创建服务
// - create_router_with_services: 测试和高级场景，注入外部服务
//
// 依赖注入的好处：
// 1. 测试时可以注入 mock 服务
// 2. 服务可以在多个组件间共享
// 3. 更灵活的服务生命周期管理
//
// 思考：如何使用 trait 实现更通用的依赖注入？
// ----------------------------------------
pub async fn create_router_with_services(
    config: Config,
    repository: Arc<Repository>,
    storage: Arc<StorageService>,
) -> Router {
    let state: AppState = Arc::new(AppData {
        storage_path: config.storage_path.clone(),
        repository: (*repository).clone(),
        storage: (*storage).clone(),
        max_file_size: config.max_file_size,
    });

    build_router(state)
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/files", get(list_files))
        .route("/api/files/{*path}", get(get_file))
        .route("/api/files/{*path}", put(upload_file))
        .route("/api/files/{*path}", delete(delete_file))
        .route("/api/devices", post(register_device))
        .route("/api/devices", get(list_devices))
        .route("/api/devices/{id}/heartbeat", post(device_heartbeat))
        .route("/api/versions", get(list_versions))
        .route("/api/syncs/{file_id}", get(get_sync_status))
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::success("ok"))
}

async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<ListFilesQuery>,
) -> impl IntoResponse {
    let base_path = &state.storage_path;

    let target_path = if let Some(p) = query.path {
        base_path.join(&p)
    } else {
        base_path.clone()
    };

    match list_directory(&target_path, base_path) {
        Ok(files) => Json(ApiResponse::success(files)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn get_file(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let file_path = state.storage_path.join(&path);

    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("File not found")),
        );
    }

    if file_path.is_dir() {
        match list_directory(&file_path, &state.storage_path) {
            Ok(files) => (StatusCode::OK, Json(ApiResponse::success(files))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&e.to_string())),
            ),
        }
    } else {
        match tokio::fs::read(&file_path).await {
            Ok(content) => {
                let hash = state.storage.compute_hash(&file_path).await.ok();
                let db_record = state.repository.get_file_by_path(&path).await.ok();

                let info = FileInfo {
                    name: file_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path,
                    is_dir: false,
                    size: content.len() as u64,
                    modified: file_path
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let datetime: chrono::DateTime<chrono::Utc> = t.into();
                            datetime.to_rfc3339()
                        }),
                    hash,
                    version: db_record.map(|r| r.version),
                };
                (StatusCode::OK, Json(ApiResponse::success(info)))
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&e.to_string())),
            ),
        }
    }
}

// [知识点 #130] 文件上传与版本控制集成
// ----------------------------------------
// 题目：如何将文件上传与版本控制结合？
//
// 讲解：
// 1. 接收文件内容
// 2. 计算哈希并存储到对象存储
// 3. 创建/更新数据库记录
// 4. 返回文件信息
//
// 这样实现了去重存储和版本追踪
//
// 思考：如何处理大文件上传？
// ----------------------------------------
async fn upload_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
    body: String,
) -> impl IntoResponse {
    // [知识点 #136] 文件大小校验
    // ----------------------------------------
    // 题目：为什么要限制上传文件大小？
    //
    // 讲解：
    // 限制文件大小可以：
    // 1. 防止 DoS 攻击（上传超大文件耗尽服务器资源）
    // 2. 控制存储成本
    // 3. 保证服务稳定性
    //
    // 对于大文件，应该使用分块上传 API
    //
    // 思考：如何实现断点续传？
    // ----------------------------------------
    if body.len() as u64 > state.max_file_size {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ApiResponse::error(&format!(
                "File too large. Max size: {} bytes, got: {} bytes",
                state.max_file_size,
                body.len()
            ))),
        );
    }

    let file_path = state.storage_path.join(&path);

    if let Some(parent) = file_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to create directory: {}",
                    e
                ))),
            );
        }
    }

    // 写入文件
    if let Err(e) = tokio::fs::write(&file_path, &body).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to write file: {}", e))),
        );
    }

    // 存储到对象存储并获取哈希
    let (hash, size) = match state.storage.store_file(&file_path).await {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to store file: {}", e))),
            );
        }
    };

    // 更新数据库记录
    let record = match state.repository.get_file_by_path(&path).await {
        Ok(existing) => {
            state
                .repository
                .update_file(existing.id, Some(hash.clone()), size)
                .await
        }
        Err(_) => {
            state
                .repository
                .create_file(crate::db::NewFileRecord {
                    path: path.clone(),
                    hash: Some(hash.clone()),
                    size,
                })
                .await
        }
    };

    match record {
        Ok(record) => {
            let info = FileInfo {
                name: file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                path,
                is_dir: false,
                size,
                modified: Some(record.updated_at.to_rfc3339()),
                hash: Some(hash),
                version: Some(record.version),
            };
            (StatusCode::OK, Json(ApiResponse::success(info)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to update record: {}",
                e
            ))),
        ),
    }
}

async fn delete_file(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let file_path = state.storage_path.join(&path);

    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("File not found")),
        );
    }

    // 从数据库删除记录
    if let Ok(record) = state.repository.get_file_by_path(&path).await {
        if let Err(e) = state.repository.delete_file(record.id).await {
            tracing::warn!("Failed to delete file record: {}", e);
        }
    }

    let result = if file_path.is_dir() {
        tokio::fs::remove_dir_all(&file_path).await
    } else {
        tokio::fs::remove_file(&file_path).await
    };

    match result {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(true))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to delete: {}", e))),
        ),
    }
}

// [知识点 #131] 设备管理 API
// ----------------------------------------
// 题目：设备注册与心跳的作用？
//
// 讲解：
// 文件同步需要知道有哪些设备：
// - 注册：新设备加入时获取唯一 ID
// - 心跳：设备定期报告在线状态
// - 列表：查看所有已注册设备
//
// 这是分布式系统的基础设施
//
// 思考：如何检测设备离线？
// ----------------------------------------
async fn register_device(
    State(state): State<AppState>,
    Json(req): Json<RegisterDeviceRequest>,
) -> impl IntoResponse {
    match state
        .repository
        .create_device(NewDeviceRecord { name: req.name })
        .await
    {
        Ok(device) => (StatusCode::OK, Json(ApiResponse::success(device))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to register device: {}",
                e
            ))),
        ),
    }
}

async fn list_devices(State(state): State<AppState>) -> impl IntoResponse {
    match state.repository.list_devices().await {
        Ok(devices) => (StatusCode::OK, Json(ApiResponse::success(devices))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to list devices: {}",
                e
            ))),
        ),
    }
}

async fn device_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match state.repository.update_device_last_seen(id).await {
        Ok(device) => (StatusCode::OK, Json(ApiResponse::success(device))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Device not found: {}", e))),
        ),
    }
}

async fn list_versions(State(state): State<AppState>) -> impl IntoResponse {
    match state.repository.list_files().await {
        Ok(files) => (StatusCode::OK, Json(ApiResponse::success(files))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to list versions: {}",
                e
            ))),
        ),
    }
}

async fn get_sync_status(
    State(state): State<AppState>,
    Path(file_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match state.repository.list_syncs_by_file(file_id).await {
        Ok(syncs) => (StatusCode::OK, Json(ApiResponse::success(syncs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to get sync status: {}",
                e
            ))),
        ),
    }
}

fn list_directory(
    target: &std::path::Path,
    base: &std::path::Path,
) -> Result<Vec<FileInfo>, Error> {
    if !target.exists() {
        return Err(Error::NotFound(target.to_path_buf()));
    }

    if !target.is_dir() {
        return Err(Error::InvalidPath("Not a directory".to_string()));
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(target)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        let relative_path = path
            .strip_prefix(base)
            .ok()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        files.push(FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: relative_path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            }),
            hash: None,
            version: None,
        });
    }

    Ok(files)
}
