// [知识点 #139] OpenAPI 文档生成
// ----------------------------------------
// 题目：为什么需要 OpenAPI 文档？
//
// 讲解：
// OpenAPI（Swagger）是 API 文档的标准：
// 1. 自动生成可交互的 API 文档
// 2. 客户端 SDK 生成的基础
// 3. API 测试工具支持
//
// utoipa 通过 Rust 宏从代码生成文档，保持同步
//
// 思考：如何保持文档与实现的一致性？
// ----------------------------------------

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::routes::{ApiResponse, FileInfo};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustCloud API",
        description = "文件同步与存储服务 API",
        version = "0.1.0",
        contact(
            name = "RustCloud Team",
            email = "team@rustcloud.dev"
        )
    ),
    components(
        schemas(FileInfo, ApiResponse)
    ),
    tags(
        (name = "files", description = "文件操作"),
        (name = "devices", description = "设备管理"),
        (name = "sync", description = "同步状态"),
        (name = "health", description = "健康检查")
    )
)]
pub struct ApiDoc;

// [知识点 #140] Swagger UI 集成
// ----------------------------------------
// 题目：如何将 Swagger UI 添加到 Axum？
//
// 讲解：
// Swagger UI 提供交互式 API 文档界面：
// - 可以直接在浏览器中测试 API
// - 显示请求/响应格式
// - 支持认证配置
//
// 思考：如何在生产环境中保护 API 文档？
// ----------------------------------------
pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
