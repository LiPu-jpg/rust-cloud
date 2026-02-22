// [知识点 #044] 集成测试
// ----------------------------------------
// 题目：集成测试和单元测试有什么区别？
//
// 讲解：
// Rust 测试分为两类：
// 1. 单元测试：在 src/ 中，#[cfg(test)] mod tests
// 2. 集成测试：在 tests/ 中，作为外部 crate 使用
//
// 集成测试的特点：
// - 只能访问 pub API
// - 每个文件是独立的 crate
// - 适合测试模块间交互
//
// 思考：什么情况下应该用集成测试而非单元测试？
// ----------------------------------------

use http_body_util::BodyExt;
use rustcloud::config::Config;
use rustcloud::db::{NewFileRecord, Repository};
use rustcloud::service::storage::{StorageConfig, StorageService};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

async fn setup() -> (TempDir, Arc<Repository>, Arc<StorageService>) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("db.json");
    let storage_path = temp_dir.path().join("storage");

    let repository = Arc::new(Repository::new(db_path).await.unwrap());
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path,
        chunk_size: 1024,
    }));

    (temp_dir, repository, storage)
}

fn make_config(temp_dir: &TempDir) -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        storage_path: temp_dir.path().join("storage"),
        max_file_size: 100 * 1024 * 1024,
    }
}

#[tokio::test]
async fn test_repository_create_and_get_file() {
    let (_temp_dir, repository, _storage) = setup().await;

    let new_file = NewFileRecord {
        path: "test.txt".to_string(),
        hash: Some("abc123".to_string()),
        size: 100,
    };

    let created = repository.create_file(new_file.clone()).await.unwrap();
    assert_eq!(created.path, "test.txt");
    assert_eq!(created.size, 100);
    assert_eq!(created.version, 1);

    let fetched = repository.get_file_by_path("test.txt").await.unwrap();
    assert_eq!(fetched.id, created.id);
}

#[tokio::test]
async fn test_repository_update_file() {
    let (_temp_dir, repository, _storage) = setup().await;

    let new_file = NewFileRecord {
        path: "test.txt".to_string(),
        hash: Some("abc123".to_string()),
        size: 100,
    };

    let created = repository.create_file(new_file).await.unwrap();

    let updated = repository
        .update_file(created.id, Some("def456".to_string()), 200)
        .await
        .unwrap();

    assert_eq!(updated.hash, Some("def456".to_string()));
    assert_eq!(updated.size, 200);
    assert_eq!(updated.version, 2);
}

#[tokio::test]
async fn test_repository_delete_file() {
    let (_temp_dir, repository, _storage) = setup().await;

    let new_file = NewFileRecord {
        path: "test.txt".to_string(),
        hash: Some("abc123".to_string()),
        size: 100,
    };

    let created = repository.create_file(new_file).await.unwrap();
    repository.delete_file(created.id).await.unwrap();

    let result = repository.get_file_by_path("test.txt").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_compute_hash() {
    let (_temp_dir, _repository, storage) = setup().await;

    let test_file = _temp_dir.path().join("test.txt");
    tokio::fs::write(&test_file, b"hello world").await.unwrap();

    let hash = storage.compute_hash(&test_file).await.unwrap();
    assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn test_storage_store_and_retrieve() {
    let (_temp_dir, _repository, storage) = setup().await;

    let test_file = _temp_dir.path().join("test.txt");
    tokio::fs::write(&test_file, b"hello world").await.unwrap();

    let (hash, size) = storage.store_file(&test_file).await.unwrap();
    assert_eq!(size, 11);

    let content = storage.retrieve_file(&hash).await.unwrap();
    assert_eq!(content, b"hello world");
}

#[tokio::test]
async fn test_storage_deduplication() {
    let (_temp_dir, _repository, storage) = setup().await;

    let file1 = _temp_dir.path().join("file1.txt");
    let file2 = _temp_dir.path().join("file2.txt");
    tokio::fs::write(&file1, b"same content").await.unwrap();
    tokio::fs::write(&file2, b"same content").await.unwrap();

    let (hash1, _) = storage.store_file(&file1).await.unwrap();
    let (hash2, _) = storage.store_file(&file2).await.unwrap();

    // 相同内容应该产生相同的 hash
    assert_eq!(hash1, hash2);
}

#[tokio::test]
async fn test_storage_store_content() {
    let (_temp_dir, _repository, storage) = setup().await;

    let content = b"test content";
    let (hash, size) = storage.store_content(content).await.unwrap();
    assert_eq!(size, 12);

    let retrieved = storage.retrieve_file(&hash).await.unwrap();
    assert_eq!(retrieved, content);
}

// [知识点 #134] API 集成测试
// ----------------------------------------
// 题目：如何测试 HTTP API？
//
// 讲解：
// 测试 HTTP API 的方法：
// 1. 使用 tower::ServiceExt::oneshot 模拟单个请求
// 2. 不需要真正启动服务器
// 3. 可以直接检查响应状态码和内容
//
// 这比端到端测试更快更可靠
//
// 思考：如何测试需要认证的 API？
// ----------------------------------------

#[tokio::test]
async fn test_api_health_check() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_config(&temp_dir);
    std::fs::create_dir_all(&config.storage_path).unwrap();

    let db_path = config.storage_path.join("db.json");
    let repository = Arc::new(Repository::new(db_path).await.unwrap());
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 1024,
    }));

    let app = rustcloud::api::create_router_with_services(config, repository, storage).await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp["success"], true);
}

#[tokio::test]
async fn test_api_register_device() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_config(&temp_dir);
    std::fs::create_dir_all(&config.storage_path).unwrap();

    let db_path = config.storage_path.join("db.json");
    let repository = Arc::new(Repository::new(db_path).await.unwrap());
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 1024,
    }));

    let app = rustcloud::api::create_router_with_services(config, repository, storage).await;

    let body = serde_json::json!({ "name": "test-device" }).to_string();
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/devices")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let resp_body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(resp["success"], true);
    assert_eq!(resp["data"]["name"], "test-device");
}

#[tokio::test]
async fn test_api_upload_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = make_config(&temp_dir);
    std::fs::create_dir_all(&config.storage_path).unwrap();

    let db_path = config.storage_path.join("db.json");
    let repository = Arc::new(Repository::new(db_path).await.unwrap());
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 1024,
    }));

    let app = rustcloud::api::create_router_with_services(config, repository, storage).await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/api/files/test.txt")
                .header("Content-Type", "text/plain")
                .body(axum::body::Body::from("hello world"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let resp_body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(resp["success"], true);
    assert_eq!(resp["data"]["path"], "test.txt");
    assert_eq!(resp["data"]["version"], 1);
}

// [知识点 #137] 文件大小限制测试
// ----------------------------------------
// 题目：如何测试文件大小限制？
//
// 讲解：
// 测试边界条件是测试的重要部分：
// - 恰好等于限制（应该通过）
// - 超过限制（应该失败）
// - 空文件（边界情况）
//
// 思考：如何测试超大文件而不实际创建它？
// ----------------------------------------
#[tokio::test]
async fn test_api_upload_file_too_large() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = make_config(&temp_dir);
    config.max_file_size = 10; // 设置很小的限制
    std::fs::create_dir_all(&config.storage_path).unwrap();

    let db_path = config.storage_path.join("db.json");
    let repository = Arc::new(Repository::new(db_path).await.unwrap());
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 1024,
    }));

    let app = rustcloud::api::create_router_with_services(config, repository, storage).await;

    // 上传超过限制的文件
    let large_content = "this is more than 10 bytes";
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/api/files/large.txt")
                .header("Content-Type", "text/plain")
                .body(axum::body::Body::from(large_content))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::PAYLOAD_TOO_LARGE);

    let resp_body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(resp["success"], false);
    assert!(resp["error"].as_str().unwrap().contains("too large"));
}

#[tokio::test]
async fn test_api_upload_file_within_limit() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = make_config(&temp_dir);
    config.max_file_size = 100; // 设置足够大的限制
    std::fs::create_dir_all(&config.storage_path).unwrap();

    let db_path = config.storage_path.join("db.json");
    let repository = Arc::new(Repository::new(db_path).await.unwrap());
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 1024,
    }));

    let app = rustcloud::api::create_router_with_services(config, repository, storage).await;

    // 上传小于限制的文件
    let small_content = "hello";
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/api/files/small.txt")
                .header("Content-Type", "text/plain")
                .body(axum::body::Body::from(small_content))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

// [知识点 #138] 文件监控测试
// ----------------------------------------
// 题目：如何测试异步事件驱动的代码？
//
// 讲解：
// 文件监控是异步事件驱动的，测试时需要：
// 1. 使用 AtomicBool 等同步原语共享状态
// 2. 等待足够时间让事件传播
// 3. 注意文件系统事件可能有延迟
//
// 思考：如何避免测试中的竞态条件？
// ----------------------------------------
#[tokio::test]
async fn test_file_watcher_detects_creation() {
    use rustcloud::watcher::file_watcher::{FileEvent, FileWatcher};
    use std::sync::atomic::{AtomicBool, Ordering};

    let temp_dir = TempDir::new().unwrap();
    let detected = Arc::new(AtomicBool::new(false));
    let detected_clone = detected.clone();

    let _watcher = FileWatcher::new(temp_dir.path(), move |event| {
        if matches!(event, FileEvent::Created(_)) {
            detected_clone.store(true, Ordering::SeqCst);
        }
    });

    // 创建文件
    let test_file = temp_dir.path().join("watched.txt");
    std::fs::write(&test_file, "content").unwrap();

    // 等待事件处理
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    assert!(detected.load(Ordering::SeqCst));
}
