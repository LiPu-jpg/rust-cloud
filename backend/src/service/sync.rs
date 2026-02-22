// [知识点 #125] 同步引擎设计
// ----------------------------------------
// 题目：文件同步的核心逻辑是什么？
//
// 讲解：
// 同步引擎负责：
// 1. 状态检测：比较本地与远程文件
// 2. 冲突检测：同一文件在多处被修改
// 3. 变更传播：上传/下载变更
// 4. 状态管理：记录同步状态
//
// 简单策略：最后写入者胜出
// 复杂策略：三路合并、操作转换
//
// 思考：如何处理网络中断和部分失败？
// ----------------------------------------

use std::sync::Arc;

use crate::db::{DeviceRecord, FileRecord, NewDeviceRecord, NewSyncRecord, Repository, SyncStatus};
use crate::error::Result;

// TODO: Phase 2 集成 - 将在实现客户端同步协议时使用
// 预留 API 端点: POST /api/sync/plan, POST /api/sync/execute
#[allow(dead_code)]
// [知识点 #135] 简化结构设计
// ----------------------------------------
// 题目：为什么移除 storage 字段？
//
// 讲解：
// SyncEngine 当前职责是设备管理和同步状态追踪。
// 实际的文件传输（上传/下载）由 API 层直接调用 StorageService。
//
// 如果未来需要实现客户端同步协议，可以重新添加：
// - storage: 用于读取文件内容发送到远程
// - client: HTTP 客户端用于远程通信
//
// 思考：服务边界如何划分？什么时候拆分服务？
// ----------------------------------------
pub struct SyncEngine {
    repository: Arc<Repository>,
}

// [知识点 #126] 同步状态机
// ----------------------------------------
// 题目：为什么需要 SyncStatus 枚举？
//
// 讲解：
// 同步是一个异步过程，需要状态跟踪：
// Pending -> Syncing -> Completed
//                  \-> Failed
//
// 状态转换：
// - Pending：检测到变更，等待同步
// - Syncing：正在传输数据
// - Completed：同步完成
// - Failed：同步失败，需要重试
//
// 思考：如何实现自动重试和指数退避？
// ----------------------------------------

#[derive(Debug, Clone)]
pub struct SyncPlan {
    pub file_id: uuid::Uuid,
    pub path: String,
    pub action: SyncAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    Upload,
    Download,
    Delete,
    Skip,
}

#[derive(Debug, Default)]
pub struct SyncReport {
    pub uploaded: usize,
    pub downloaded: usize,
    pub deleted: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl SyncEngine {
    pub fn new(repository: Arc<Repository>) -> Self {
        SyncEngine { repository }
    }

    pub async fn register_device(&self, name: &str) -> Result<DeviceRecord> {
        self.repository
            .create_device(NewDeviceRecord {
                name: name.to_string(),
            })
            .await
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceRecord>> {
        self.repository.list_devices().await
    }

    pub async fn heartbeat(&self, device_id: uuid::Uuid) -> Result<DeviceRecord> {
        self.repository.update_device_last_seen(device_id).await
    }

    // [知识点 #127] 同步计划生成
    // ----------------------------------------
    // 题目：如何判断文件需要上传还是下载？
    //
    // 讲解：
    // 简单同步策略：
    // 1. 本地有、远程无 -> 上传
    // 2. 本地无、远程有 -> 下载
    // 3. 都有但不同 -> 比较版本/时间戳
    //
    // 复杂策略考虑：
    // - 向量时钟：追踪因果关系
    // - 内容寻址：用 hash 判断是否相同
    //
    // 思考：如何实现真正的双向同步？
    // ----------------------------------------
    pub async fn create_sync_plan(&self, local_files: &[FileRecord]) -> Result<Vec<SyncPlan>> {
        let remote_files = self.repository.list_files().await?;
        let mut plans = Vec::new();

        let remote_paths: std::collections::HashSet<_> =
            remote_files.iter().map(|f| f.path.as_str()).collect();

        for local in local_files {
            if !remote_paths.contains(local.path.as_str()) {
                plans.push(SyncPlan {
                    file_id: local.id,
                    path: local.path.clone(),
                    action: SyncAction::Upload,
                });
            } else if let Ok(remote) = self.repository.get_file_by_path(&local.path).await {
                if remote.hash != local.hash {
                    let action = if remote.version > local.version {
                        SyncAction::Download
                    } else {
                        SyncAction::Upload
                    };
                    plans.push(SyncPlan {
                        file_id: local.id,
                        path: local.path.clone(),
                        action,
                    });
                } else {
                    plans.push(SyncPlan {
                        file_id: local.id,
                        path: local.path.clone(),
                        action: SyncAction::Skip,
                    });
                }
            }
        }

        Ok(plans)
    }

    pub async fn sync_file(
        &self,
        file_id: uuid::Uuid,
        device_id: uuid::Uuid,
        action: SyncAction,
    ) -> Result<()> {
        let new_sync = NewSyncRecord {
            device_id,
            file_id,
            sync_status: SyncStatus::Syncing,
        };
        let sync_record = self.repository.create_sync(new_sync).await;

        let result = match action {
            SyncAction::Upload | SyncAction::Download | SyncAction::Skip => {
                Ok::<(), crate::error::Error>(())
            }
            SyncAction::Delete => self.repository.delete_file(file_id).await.map(|_| ()),
        };

        match result {
            Ok(_) => {
                let _ = self
                    .repository
                    .update_sync_status(sync_record?.id, SyncStatus::Completed)
                    .await;
                Ok(())
            }
            Err(e) => {
                let _ = self
                    .repository
                    .update_sync_status(sync_record?.id, SyncStatus::Failed)
                    .await;
                Err(e)
            }
        }
    }

    pub async fn get_sync_status(&self, file_id: uuid::Uuid) -> Result<Vec<crate::db::SyncRecord>> {
        self.repository.list_syncs_by_file(file_id).await
    }
}
