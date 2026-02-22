// [知识点 #124] 版本控制设计
// ----------------------------------------
// 题目：版本控制需要存储哪些信息？
//
// 讲解：
// 文件版本控制的核心数据：
// 1. 文件路径：定位文件
// 2. 版本号：递增标识
// 3. 内容哈希：指向实际内容
// 4. 时间戳：变更时间
// 5. 变更者：谁做的修改（可选）
//
// Git 使用 DAG（有向无环图）存储版本历史
// 简化版本可以用线性版本号
//
// 思考：如何实现分支和合并？
// ----------------------------------------

use std::path::Path;
use std::sync::Arc;

use crate::db::{FileRecord, NewFileRecord, Repository};
use crate::error::Result;
use crate::service::storage::StorageService;

// TODO: Phase 2 集成 - 将在实现版本历史功能时使用
// 预留 API 端点: GET /api/files/{path}/versions, POST /api/files/{path}/rollback
#[allow(dead_code)]
// [知识点 #083] 组合优于继承
// ----------------------------------------
// 题目：VersionService 如何访问 StorageService 和 Repository？
//
// 讲解：
// Rust 没有继承，使用组合模式：
// - 持有其他服务的引用（Arc）
// - 通过方法调用委托操作
//
// Arc 允许多个所有者共享同一服务实例
// 避免重复创建和状态不同步
//
// 思考：如果服务之间有循环依赖怎么办？
// ----------------------------------------
pub struct VersionService {
    storage: Arc<StorageService>,
    repository: Arc<Repository>,
}

impl VersionService {
    pub fn new(storage: Arc<StorageService>, repository: Arc<Repository>) -> Self {
        VersionService {
            storage,
            repository,
        }
    }

    pub async fn create_version(&self, path: &Path) -> Result<FileRecord> {
        let (hash, size) = self.storage.store_file(path).await?;

        let new_file = NewFileRecord {
            path: path.to_string_lossy().to_string(),
            hash: Some(hash),
            size,
        };

        self.repository.create_file(new_file).await
    }

    pub async fn update_version(&self, path: &Path) -> Result<FileRecord> {
        let existing = self
            .repository
            .get_file_by_path(&path.to_string_lossy())
            .await
            .ok();

        let (hash, size) = self.storage.store_file(path).await?;

        match existing {
            Some(record) => {
                // 检查内容是否变化
                if record.hash.as_deref() == Some(hash.as_str()) {
                    return Ok(record);
                }
                self.repository
                    .update_file(record.id, Some(hash), size)
                    .await
            }
            None => {
                let new_file = NewFileRecord {
                    path: path.to_string_lossy().to_string(),
                    hash: Some(hash),
                    size,
                };
                self.repository.create_file(new_file).await
            }
        }
    }

    pub async fn get_version(&self, path: &str) -> Result<FileRecord> {
        self.repository.get_file_by_path(path).await
    }

    pub async fn get_content(&self, record: &FileRecord) -> Result<Vec<u8>> {
        match &record.hash {
            Some(hash) => self.storage.retrieve_chunked(hash).await,
            None => Err(crate::error::Error::NotFound(std::path::PathBuf::from(
                &record.path,
            ))),
        }
    }

    pub async fn delete_version(&self, path: &str) -> Result<()> {
        let record = self.repository.get_file_by_path(path).await?;
        self.repository.delete_file(record.id).await
    }

    pub async fn list_versions(&self) -> Result<Vec<FileRecord>> {
        self.repository.list_files().await
    }

    pub async fn has_changes(&self, path: &Path) -> Result<bool> {
        let current_hash = self.storage.compute_hash(path).await?;
        let existing = self
            .repository
            .get_file_by_path(&path.to_string_lossy())
            .await
            .ok();

        Ok(existing.is_none_or(|r| r.hash.as_deref() != Some(current_hash.as_str())))
    }
}
