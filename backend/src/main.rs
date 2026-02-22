// [知识点 #063] 异步入口函数
// ----------------------------------------
// 题目：#[tokio::main] 宏展开后是什么样子？
//
// 讲解：
// #[tokio::main] 是一个属性宏，将 async fn main() 展开为：
// fn main() {
//     tokio::runtime::Runtime::new()
//         .unwrap()
//         .block_on(async {
//             // 原来的 async body
//         })
// }
//
// tokio 运行时负责：
// 1. 创建任务调度器
// 2. 管理 I/O 事件循环
// 3. 在多线程上执行异步任务
//
// 思考：为什么 Rust 的 async 不像 Go 一样内置运行时？
// ----------------------------------------

mod api;
mod config;
mod db;
mod error;
mod service;
mod watcher;

use std::sync::Arc;

use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::Repository;
use crate::service::storage::{StorageConfig, StorageService};
use crate::watcher::file_watcher::WatcherService;

// [知识点 #081] 初始化与副作用
// ----------------------------------------
// 题目：为什么 main 函数返回 Result？
//
// 讲解：
// 生产代码应返回 Result：
// - 允许使用 ? 运算符传播错误
// - 返回非零退出码表示失败
// - 打印错误信息更规范
//
// Box<dyn std::error::Error> 可以容纳任何错误类型
//
// 思考：如何自定义 main 函数的错误处理？
// ----------------------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // [知识点 #142] 结构化日志配置
    // ----------------------------------------
    // 题目：为什么要配置日志输出到文件？
    //
    // 讲解：
    // 生产环境需要将日志持久化：
    // - 控制台日志用于开发调试
    // - 文件日志用于生产审计和问题排查
    // - 日志轮转避免单个文件过大
    //
    // tracing-appender 提供日志轮转功能
    //
    // 思考：如何实现日志的集中收集？
    // ----------------------------------------
    let enable_file_logging = std::env::var("RUSTCLOUD_LOG_FILE")
        .map(|v| v == "true")
        .unwrap_or(false);

    if enable_file_logging {
        let log_dir = std::path::PathBuf::from("./logs");
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)?;
        }

        let file_appender = tracing_appender::rolling::daily(&log_dir, "rustcloud.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "rustcloud=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "rustcloud=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let config = Config::from_env_or_default();
    tracing::info!("Loaded config: {:?}", config);

    if !config.storage_path.exists() {
        std::fs::create_dir_all(&config.storage_path)?;
        tracing::info!("Created storage directory: {:?}", config.storage_path);
    }

    // [知识点 #132] 服务初始化顺序
    // ----------------------------------------
    // 题目：为什么先初始化 Repository 和 Storage？
    //
    // 讲解：
    // 依赖关系：
    // - API routes 需要 Repository 和 Storage
    // - FileWatcher 需要 Repository 和 Storage
    // - SyncEngine 需要 Repository 和 Storage
    //
    // 使用 Arc 共享实例，避免重复创建
    //
    // 思考：如何处理循环依赖？
    // ----------------------------------------
    let db_path = config.storage_path.join("db.json");
    let repository = Arc::new(Repository::new(db_path).await?);
    let storage = Arc::new(StorageService::new(StorageConfig {
        storage_path: config.storage_path.clone(),
        chunk_size: 4 * 1024 * 1024,
    }));

    // 可选：启用文件监控
    let _watcher = if std::env::var("RUSTCLOUD_WATCH")
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        let mut watcher = WatcherService::new(storage.clone(), repository.clone());
        watcher.start(&config.storage_path)?;
        tracing::info!("File watcher started for: {:?}", config.storage_path);
        Some(watcher)
    } else {
        None
    };

    let app: Router = api::create_router_with_services(config.clone(), repository, storage).await;

    // [知识点 #141] Swagger UI 集成
    // ----------------------------------------
    // 题目：如何合并多个 Router？
    //
    // 讲解：
    // Axum 的 Router 可以通过 merge 方法合并：
    // - 主 API 路由
    // - Swagger UI 文档路由
    // - 其他静态资源路由
    //
    // Swagger UI 访问地址：http://host:port/swagger-ui
    //
    // 思考：如何在生产环境禁用 Swagger UI？
    // ----------------------------------------
    let app = app.merge(api::doc::swagger_ui());

    let listener = tokio::net::TcpListener::bind(&config.addr()).await?;
    tracing::info!("Server running at http://{}", config.addr());
    tracing::info!("API docs available at http://{}/swagger-ui", config.addr());

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
