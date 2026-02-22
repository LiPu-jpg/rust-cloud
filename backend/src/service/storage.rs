// [知识点 #121] SHA-256 哈希计算
// ----------------------------------------
// 题目：为什么用 SHA-256 而不是简单比较文件内容？
//
// 讲解：
// 文件同步中，哈希是判断文件是否变化的关键：
// 1. 固定长度：无论文件多大，哈希值都是 32 字节
// 2. 快速比较：比较 32 字节比比较整个文件快得多
// 3. 唯一标识：可以用 hash 作为存储文件名
//
// SHA-256 是加密哈希函数，碰撞概率极低
// 对于文件同步场景，足以保证唯一性
//
// 思考：哈希碰撞时会发生什么？如何处理？
// ----------------------------------------

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;

use crate::error::{Error, Result};

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub storage_path: PathBuf,
    pub chunk_size: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            storage_path: PathBuf::from("./storage"),
            chunk_size: CHUNK_SIZE,
        }
    }
}

// [知识点 #082] 异步服务的设计模式
// ----------------------------------------
// 题目：为什么 StorageService 持有 Config 而不是每次传入？
//
// 讲解：
// 服务通常有固定的配置，不需要每次调用都传入。
// 这种模式：
// 1. 配置在创建时确定，运行时不可变
// 2. 方法签名更简洁
// 3. 便于依赖注入和测试
//
// 如果配置需要动态更新，可以用 Arc<RwLock<Config>>
//
// 思考：如何让 StorageService 支持运行时配置更新？
// ----------------------------------------
#[derive(Debug, Clone)]
pub struct StorageService {
    config: StorageConfig,
}

impl StorageService {
    pub fn new(config: StorageConfig) -> Self {
        StorageService { config }
    }

    pub fn storage_path(&self) -> &Path {
        &self.config.storage_path
    }

    // [知识点 #122] 异步文件读取与哈希
    // ----------------------------------------
    // 题目：为什么用 async 函数处理文件？
    //
    // 讲解：
    // 文件 I/O 是阻塞操作，但在 tokio 中：
    // - tokio::fs 在独立线程池执行，不阻塞调度器
    // - 对于大文件，异步读取允许其他任务并发执行
    //
    // update 方法增量更新哈希，避免一次性读入内存
    // 这对于大文件很重要
    //
    // 思考：如何在读取大文件时显示进度？
    // ----------------------------------------
    pub async fn compute_hash(&self, path: &Path) -> Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; self.config.chunk_size];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    // [知识点 #006] 路径规范化与安全
    // ----------------------------------------
    // 题目：hash_to_path 的目录结构有什么好处？
    //
    // 讲解：
    // 使用 hash 前两个字符作为子目录：
    // storage/ab/cdef1234...
    //
    // 好处：
    // 1. 避免单个目录文件过多（文件系统性能）
    // 2. 便于备份和迁移
    // 3. 天然的负载均衡（hash 分布均匀）
    //
    // 这种模式在 Git、Docker 等系统中广泛使用
    //
    // 思考：为什么取前两个字符而不是更多？
    // ----------------------------------------
    fn hash_to_path(&self, hash: &str) -> PathBuf {
        let (prefix, rest) = hash.split_at(2);
        self.config
            .storage_path
            .join("objects")
            .join(prefix)
            .join(rest)
    }

    pub async fn store_file(&self, source: &Path) -> Result<(String, u64)> {
        let hash = self.compute_hash(source).await?;
        let target = self.hash_to_path(&hash);

        if !target.exists() {
            if let Some(parent) = target.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::copy(source, &target).await?;
        }

        let metadata = tokio::fs::metadata(source).await?;
        Ok((hash, metadata.len()))
    }

    pub async fn store_content(&self, content: &[u8]) -> Result<(String, u64)> {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());

        let target = self.hash_to_path(&hash);

        if !target.exists() {
            if let Some(parent) = target.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&target, content).await?;
        }

        Ok((hash, content.len() as u64))
    }

    pub async fn retrieve_file(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.hash_to_path(hash);
        if !path.exists() {
            return Err(Error::NotFound(path));
        }
        let content = tokio::fs::read(&path).await?;
        Ok(content)
    }

    pub async fn file_exists(&self, hash: &str) -> bool {
        self.hash_to_path(hash).exists()
    }

    pub async fn delete_file(&self, hash: &str) -> Result<()> {
        let path = self.hash_to_path(hash);
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        Ok(())
    }

    // [知识点 #123] 分块存储
    // ----------------------------------------
    // 题目：为什么大文件需要分块存储？
    //
    // 讲解：
    // 分块存储的好处：
    // 1. 增量同步：只传输变化的块
    // 2. 断点续传：网络中断后可继续
    // 3. 内存友好：不需要一次性加载整个文件
    // 4. 去重：相同内容的块只存储一次
    //
    // 云存储服务（如 Dropbox、S3）都使用分块
    //
    // 思考：如何确定最优的块大小？
    // ----------------------------------------
    pub async fn store_chunked(&self, source: &Path) -> Result<(String, u64, Vec<String>)> {
        let metadata = tokio::fs::metadata(source).await?;
        let file_size = metadata.len();

        if file_size <= self.config.chunk_size as u64 {
            let (hash, size) = self.store_file(source).await?;
            return Ok((hash.clone(), size, vec![hash]));
        }

        let mut file = tokio::fs::File::open(source).await?;
        let mut buffer = vec![0u8; self.config.chunk_size];
        let mut chunks = Vec::new();
        let mut file_hasher = Sha256::new();

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk_data = &buffer[..bytes_read];
            file_hasher.update(chunk_data);

            let (chunk_hash, _) = self.store_content(chunk_data).await?;
            chunks.push(chunk_hash);
        }

        let file_hash = format!("{:x}", file_hasher.finalize());

        let manifest = ChunkManifest {
            file_hash: file_hash.clone(),
            file_size,
            chunks: chunks.clone(),
        };
        let manifest_path = self.hash_to_path(&format!("manifest-{}", file_hash));
        if let Some(parent) = manifest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let manifest_content = serde_json::to_vec(&manifest)?;
        tokio::fs::write(&manifest_path, manifest_content).await?;

        Ok((file_hash, file_size, chunks))
    }

    pub async fn retrieve_chunked(&self, hash: &str) -> Result<Vec<u8>> {
        let manifest_path = self.hash_to_path(&format!("manifest-{}", hash));

        if manifest_path.exists() {
            let manifest_content = tokio::fs::read(&manifest_path).await?;
            let manifest: ChunkManifest = serde_json::from_slice(&manifest_content)?;

            let mut result = Vec::with_capacity(manifest.file_size as usize);
            for chunk_hash in &manifest.chunks {
                let chunk_data = self.retrieve_file(chunk_hash).await?;
                result.extend_from_slice(&chunk_data);
            }
            Ok(result)
        } else {
            self.retrieve_file(hash).await
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChunkManifest {
    file_hash: String,
    file_size: u64,
    chunks: Vec<String>,
}
