// [知识点 #023] 数据模型设计
// ----------------------------------------
// 题目：为什么字段用 Option 包装？
//
// 讲解：
// 数据库中的字段可能为 NULL，Rust 用 Option 表示可空值。
// Option<T> 是一个枚举：Some(T) 或 None。
// 这是 Rust 避免空指针异常的核心机制。
//
// 数据库字段设计原则：
// - 必填字段用 T
// - 可选字段用 Option<T>
// - 有默认值用 #[serde(default)]
//
// 思考：Option 的内存布局是怎样的？为什么没有开销？
// ----------------------------------------

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub path: String,
    pub hash: Option<String>,
    pub size: u64,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// [知识点 #024] 新建记录与完整记录分离
// ----------------------------------------
// 题目：为什么需要 NewFileRecord 和 FileRecord 两个结构体？
//
// 讲解：
// 这是 Rust 中常见的 "输入类型" 与 "存储类型" 分离模式：
// - NewFileRecord：创建时需要的字段（不含 id, created_at 等自动生成的）
// - FileRecord：完整记录，包含所有字段
//
// 这样设计的好处：
// 1. 类型系统强制调用者提供必需字段
// 2. 自动生成的字段不会被误设置
// 3. API 更清晰，不易出错
//
// 思考：有没有办法用一个结构体实现两种用途？
// ----------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFileRecord {
    pub path: String,
    pub hash: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub id: Uuid,
    pub device_id: Uuid,
    pub file_id: Uuid,
    pub sync_status: SyncStatus,
    pub last_sync_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSyncRecord {
    pub device_id: Uuid,
    pub file_id: Uuid,
    pub sync_status: SyncStatus,
}

// [知识点 #025] 枚举与数据库映射
// ----------------------------------------
// 题目：如何将 Rust 枚举存储到数据库？
//
// 讲解：
// 数据库通常没有枚举类型（或各数据库实现不同），
// 常见方案：
// 1. 存储为字符串（可读性好，空间稍大）
// 2. 存储为整数（空间小，可读性差）
//
// serde 的 rename_all 可以控制序列化格式：
// - "SCREAMING_SNAKE_CASE" -> "SYNCING"
// - "lowercase" -> "syncing"
//
// 思考：如何处理枚举值变更（数据库迁移）？
// ----------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncStatus {
    Pending,
    Syncing,
    Completed,
    Failed,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncStatus::Pending => "PENDING",
            SyncStatus::Syncing => "SYNCING",
            SyncStatus::Completed => "COMPLETED",
            SyncStatus::Failed => "FAILED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub id: Uuid,
    pub name: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDeviceRecord {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub files: Vec<FileRecord>,
    pub syncs: Vec<SyncRecord>,
    pub devices: Vec<DeviceRecord>,
}

impl FileRecord {
    pub fn new(new_record: NewFileRecord) -> Self {
        let now = Utc::now();
        FileRecord {
            id: Uuid::new_v4(),
            path: new_record.path,
            hash: new_record.hash,
            size: new_record.size,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

impl SyncRecord {
    pub fn new(new_record: NewSyncRecord) -> Self {
        SyncRecord {
            id: Uuid::new_v4(),
            device_id: new_record.device_id,
            file_id: new_record.file_id,
            sync_status: new_record.sync_status,
            last_sync_at: Utc::now(),
        }
    }
}

impl DeviceRecord {
    pub fn new(new_record: NewDeviceRecord) -> Self {
        DeviceRecord {
            id: Uuid::new_v4(),
            name: new_record.name,
            last_seen: Utc::now(),
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }
}
