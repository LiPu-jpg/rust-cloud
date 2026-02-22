// [知识点 #081] Arc<Mutex> 与内部可变性
// ----------------------------------------
// 题目：为什么 Repository 用 Arc<Mutex<Database>> 而不是直接持有 Database？
//
// 讲解：
// Repository 需要在多个 handler 之间共享，且需要修改数据。
// Arc<Mutex<T>> 组合：
// - Arc：多所有权，允许 clone 出多个引用
// - Mutex：内部可变性，允许通过不可变引用修改数据
//
// Mutex vs RwLock：
// - Mutex：同一时刻只有一个线程可以访问（读写都互斥）
// - RwLock：允许多个读者或一个写者
//
// 这里用 Mutex 是因为大多数操作都需要写入，RwLock 优势不明显
//
// 思考：什么情况下应该用 RwLock 而非 Mutex？
// ----------------------------------------

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::models::{
    Database, DeviceRecord, FileRecord, NewDeviceRecord, NewFileRecord, NewSyncRecord, SyncRecord,
    SyncStatus,
};
use crate::error::{Error, Result};

#[derive(Clone)]
pub struct Repository {
    data: Arc<Mutex<Database>>,
    db_path: PathBuf,
}

impl Repository {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let database = if db_path.exists() {
            let content = tokio::fs::read_to_string(&db_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Database::default()
        };

        Ok(Repository {
            data: Arc::new(Mutex::new(database)),
            db_path,
        })
    }

    async fn save(&self) -> Result<()> {
        let data = self.data.lock().await;
        let content = serde_json::to_string_pretty(&*data)?;
        tokio::fs::write(&self.db_path, content).await?;
        Ok(())
    }

    // [知识点 #043] async 方法与锁的作用域
    // ----------------------------------------
    // 题目：为什么 lock().await 后要尽快释放锁？
    //
    // 讲解：
    // Mutex::lock().await 会等待获取锁，持有锁期间其他任务无法访问。
    // 如果在持有锁时执行耗时操作或 .await，会阻塞其他任务。
    //
    // 最佳实践：
    // 1. 获取锁后尽快完成操作
    // 2. 避免在持有锁时调用其他 async 函数
    // 3. 如果必须调用，考虑先克隆需要的数据再释放锁
    //
    // 思考：如果必须在持有锁时 .await，有什么解决方案？
    // ----------------------------------------

    pub async fn create_file(&self, new_file: NewFileRecord) -> Result<FileRecord> {
        let mut data = self.data.lock().await;

        // 检查路径是否已存在
        if data.files.iter().any(|f| f.path == new_file.path) {
            return Err(Error::AlreadyExists(PathBuf::from(&new_file.path)));
        }

        let record = FileRecord::new(new_file);
        data.files.push(record.clone());
        drop(data); // 提前释放锁

        self.save().await?;
        Ok(record)
    }

    pub async fn get_file_by_path(&self, path: &str) -> Result<FileRecord> {
        let data = self.data.lock().await;
        data.files
            .iter()
            .find(|f| f.path == path)
            .cloned()
            .ok_or_else(|| Error::NotFound(PathBuf::from(path)))
    }

    pub async fn get_file_by_id(&self, id: uuid::Uuid) -> Result<FileRecord> {
        let data = self.data.lock().await;
        data.files
            .iter()
            .find(|f| f.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound(PathBuf::from(format!("file:{}", id))))
    }

    pub async fn update_file(
        &self,
        id: uuid::Uuid,
        hash: Option<String>,
        size: u64,
    ) -> Result<FileRecord> {
        let mut data = self.data.lock().await;
        let file = data
            .files
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| Error::NotFound(PathBuf::from(format!("file:{}", id))))?;

        file.hash = hash;
        file.size = size;
        file.increment_version();
        let record = file.clone();
        drop(data);

        self.save().await?;
        Ok(record)
    }

    pub async fn delete_file(&self, id: uuid::Uuid) -> Result<()> {
        let mut data = self.data.lock().await;
        let idx = data
            .files
            .iter()
            .position(|f| f.id == id)
            .ok_or_else(|| Error::NotFound(PathBuf::from(format!("file:{}", id))))?;

        data.files.remove(idx);
        // 同时删除相关的同步记录
        data.syncs.retain(|s| s.file_id != id);
        drop(data);

        self.save().await
    }

    pub async fn list_files(&self) -> Result<Vec<FileRecord>> {
        let data = self.data.lock().await;
        Ok(data.files.clone())
    }

    pub async fn create_sync(&self, new_sync: NewSyncRecord) -> Result<SyncRecord> {
        let mut data = self.data.lock().await;

        // 验证 file_id 存在
        if !data.files.iter().any(|f| f.id == new_sync.file_id) {
            return Err(Error::NotFound(PathBuf::from(format!(
                "file:{}",
                new_sync.file_id
            ))));
        }

        let record = SyncRecord::new(new_sync);
        data.syncs.push(record.clone());
        drop(data);

        self.save().await?;
        Ok(record)
    }

    pub async fn update_sync_status(
        &self,
        id: uuid::Uuid,
        status: SyncStatus,
    ) -> Result<SyncRecord> {
        let mut data = self.data.lock().await;
        let sync = data
            .syncs
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| Error::NotFound(PathBuf::from(format!("sync:{}", id))))?;

        sync.sync_status = status;
        sync.last_sync_at = chrono::Utc::now();
        let record = sync.clone();
        drop(data);

        self.save().await?;
        Ok(record)
    }

    pub async fn list_syncs_by_file(&self, file_id: uuid::Uuid) -> Result<Vec<SyncRecord>> {
        let data = self.data.lock().await;
        Ok(data
            .syncs
            .iter()
            .filter(|s| s.file_id == file_id)
            .cloned()
            .collect())
    }

    pub async fn create_device(&self, new_device: NewDeviceRecord) -> Result<DeviceRecord> {
        let mut data = self.data.lock().await;
        let record = DeviceRecord::new(new_device);
        data.devices.push(record.clone());
        drop(data);

        self.save().await?;
        Ok(record)
    }

    pub async fn get_device(&self, id: uuid::Uuid) -> Result<DeviceRecord> {
        let data = self.data.lock().await;
        data.devices
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound(PathBuf::from(format!("device:{}", id))))
    }

    pub async fn update_device_last_seen(&self, id: uuid::Uuid) -> Result<DeviceRecord> {
        let mut data = self.data.lock().await;
        let device = data
            .devices
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or_else(|| Error::NotFound(PathBuf::from(format!("device:{}", id))))?;

        device.update_last_seen();
        let record = device.clone();
        drop(data);

        self.save().await?;
        Ok(record)
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceRecord>> {
        let data = self.data.lock().await;
        Ok(data.devices.clone())
    }
}
