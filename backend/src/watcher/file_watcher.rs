// [知识点 #128] 文件系统监控
// ----------------------------------------
// 题目：如何实时检测文件变化？
//
// 讲解：
// 操作系统提供文件系统事件通知机制：
// - Linux: inotify
// - macOS: FSEvents
// - Windows: ReadDirectoryChangesW
//
// notify 库提供了跨平台抽象，统一处理这些差异
//
// 典型事件类型：
// - Create: 新建文件/目录
// - Modify: 内容修改
// - Remove: 删除
// - Rename: 重命名/移动
//
// 思考：如何处理事件风暴（短时间内大量事件）？
// ----------------------------------------

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

// [知识点 #065] 通道通信
// ----------------------------------------
// 题目：为什么用 mpsc 通道传递文件事件？
//
// 讲解：
// 文件监控在独立线程运行，事件需要传递到主线程处理。
// mpsc (Multi-Producer Single-Consumer) 通道：
// - 多个发送者，一个接收者
// - 异步发送，不阻塞发送者
// - 接收者可以 await 消息
//
// 这是 Rust 并发编程的核心模式：
// "不要通过共享内存来通信，而是通过通信来共享内存"
//
// 思考：如果需要多个接收者怎么办？
// ----------------------------------------

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(std::path::PathBuf),
    Modified(std::path::PathBuf),
    Deleted(std::path::PathBuf),
    Renamed {
        from: std::path::PathBuf,
        to: std::path::PathBuf,
    },
}

pub struct FileWatcher {
    watcher: RecommendedWatcher,
}

// [知识点 #084] 闭包与 move 关键字
// ----------------------------------------
// 题目：为什么闭包需要 move 关键字？
//
// 讲解：
// 闭包会捕获外部变量，默认是引用捕获。
// 但这里的 tx 需要发送者所有权转移到闭包中，
// 因为闭包会在另一个线程中长期运行。
//
// move 关键字强制所有权转移，而非借用
//
// 思考：如果闭包中需要修改捕获的变量怎么办？
// ----------------------------------------
impl FileWatcher {
    pub fn new<F>(path: &Path, callback: F) -> Result<Self, notify::Error>
    where
        F: Fn(FileEvent) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<FileEvent>(100);

        // 创建回调任务的运行时
        let handle_event = move |event: Result<Event, notify::Error>| {
            if let Ok(event) = event {
                if let Some(file_event) = Self::convert_event(event) {
                    let _ = tx.blocking_send(file_event);
                }
            }
        };

        let mut watcher = RecommendedWatcher::new(handle_event, Config::default())?;
        watcher.watch(path, RecursiveMode::Recursive)?;

        // 在后台任务中处理事件
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                callback(event);
            }
        });

        Ok(FileWatcher { watcher })
    }

    fn convert_event(event: Event) -> Option<FileEvent> {
        use notify::EventKind;

        let path = event.paths.first()?.clone();

        match event.kind {
            EventKind::Create(_) => Some(FileEvent::Created(path)),
            EventKind::Modify(_) => Some(FileEvent::Modified(path)),
            EventKind::Remove(_) => Some(FileEvent::Deleted(path)),
            EventKind::Any | EventKind::Access(_) | EventKind::Other => None,
        }
    }

    pub fn stop(&mut self) {
        let _ = self.watcher.unwatch(std::path::Path::new("."));
    }
}

// [知识点 #129] 异步服务集成
// ----------------------------------------
// 题目：FileWatcher 如何与 VersionService 集成？
//
// 讲解：
// FileWatcher 接收事件后，可以：
// 1. 直接调用 VersionService（同步）
// 2. 发送到任务队列，由工作线程处理（异步）
// 3. 使用 Actor 模式封装服务
//
// 这里的 WatcherService 封装了完整的监控逻辑
//
// 思考：如何保证事件处理的顺序性？
// ----------------------------------------
pub struct WatcherService {
    watcher: Option<FileWatcher>,
    storage: Arc<crate::service::storage::StorageService>,
    repository: Arc<crate::db::Repository>,
}

impl WatcherService {
    pub fn new(
        storage: Arc<crate::service::storage::StorageService>,
        repository: Arc<crate::db::Repository>,
    ) -> Self {
        WatcherService {
            watcher: None,
            storage,
            repository,
        }
    }

    pub fn start(&mut self, path: &Path) -> Result<(), notify::Error> {
        let storage = self.storage.clone();
        let repository = self.repository.clone();

        let watcher = FileWatcher::new(path, move |event| {
            let storage = storage.clone();
            let repository = repository.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_event(event, &storage, &repository).await {
                    tracing::error!("Failed to handle file event: {}", e);
                }
            });
        })?;

        self.watcher = Some(watcher);
        Ok(())
    }

    async fn handle_event(
        event: FileEvent,
        storage: &crate::service::storage::StorageService,
        repository: &crate::db::Repository,
    ) -> crate::error::Result<()> {
        match event {
            FileEvent::Created(path) | FileEvent::Modified(path) => {
                if path.is_file() {
                    let (hash, size) = storage.store_file(&path).await?;
                    tracing::info!("File stored: {:?} (hash: {}, size: {})", path, hash, size);
                }
            }
            FileEvent::Deleted(path) => {
                tracing::info!("File deleted: {:?}", path);
                if let Ok(record) = repository.get_file_by_path(&path.to_string_lossy()).await {
                    repository.delete_file(record.id).await?;
                }
            }
            FileEvent::Renamed { from, to } => {
                tracing::info!("File renamed: {:?} -> {:?}", from, to);
                if let Ok(record) = repository.get_file_by_path(&from.to_string_lossy()).await {
                    repository.delete_file(record.id).await?;
                }
                if to.is_file() {
                    let (hash, size) = storage.store_file(&to).await?;
                    tracing::info!("File stored: {:?} (hash: {}, size: {})", to, hash, size);
                }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(watcher) = &mut self.watcher {
            watcher.stop();
        }
        self.watcher = None;
    }
}
