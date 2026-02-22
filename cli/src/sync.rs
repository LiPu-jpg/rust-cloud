use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use anyhow::Result;

use crate::client::Client;

pub struct SyncEngine {
    client: Client,
    local_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LocalFile {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

impl SyncEngine {
    pub fn new(client: Client, local_path: PathBuf) -> Self {
        SyncEngine { client, local_path }
    }

    pub async fn sync(&self, dry_run: bool) -> Result<SyncReport> {
        println!("Scanning local files...");
        let _local_files = self.scan_local_files()?;
        
        println!("Fetching remote versions...");
        let remote_files = self.client.list_versions().await?;
        
        println!("Creating sync plan...");
        let plan = self.client.create_sync_plan(&remote_files).await?;
        
        let mut report = SyncReport::default();
        
        for item in plan {
            match item.action.as_str() {
                "upload" => {
                    println!("[UPLOAD] {}", item.path);
                    if !dry_run {
                        let local_path = self.local_path.join(&item.path);
                        if local_path.exists() {
                            let content = tokio::fs::read(&local_path).await?;
                            self.client.upload_file(&item.path, &content).await?;
                            report.uploaded += 1;
                        }
                    } else {
                        report.uploaded += 1;
                    }
                }
                "download" => {
                    println!("[DOWNLOAD] {}", item.path);
                    if !dry_run {
                        let content = self.client.download_file(&item.path).await?;
                        let local_path = self.local_path.join(&item.path);
                        if let Some(parent) = local_path.parent() {
                            tokio::fs::create_dir_all(parent).await?;
                        }
                        tokio::fs::write(&local_path, content).await?;
                        report.downloaded += 1;
                    } else {
                        report.downloaded += 1;
                    }
                }
                "delete" => {
                    println!("[DELETE] {}", item.path);
                    if !dry_run {
                        let local_path = self.local_path.join(&item.path);
                        if local_path.exists() {
                            if local_path.is_dir() {
                                tokio::fs::remove_dir_all(&local_path).await?;
                            } else {
                                tokio::fs::remove_file(&local_path).await?;
                            }
                        }
                        report.deleted += 1;
                    } else {
                        report.deleted += 1;
                    }
                }
                "skip" => {
                    report.skipped += 1;
                }
                _ => {}
            }
        }
        
        Ok(report)
    }

    fn scan_local_files(&self) -> Result<Vec<LocalFile>> {
        let mut files = Vec::new();
        self.scan_dir(&self.local_path, &mut files)?;
        Ok(files)
    }

    fn scan_dir(&self, dir: &Path, files: &mut Vec<LocalFile>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.scan_dir(&path, files)?;
            } else {
                let relative = path.strip_prefix(&self.local_path)?
                    .to_string_lossy()
                    .replace('\\', "/");
                
                let content = std::fs::read(&path)?;
                let hash = format!("{:x}", Sha256::digest(&content));
                let size = content.len() as u64;
                
                files.push(LocalFile {
                    path: relative,
                    hash,
                    size,
                });
            }
        }

        Ok(())
    }

    pub async fn status(&self) -> Result<SyncStatus> {
        let local_files = self.scan_local_files()?;
        let remote_files = self.client.list_files(None).await?;
        
        let local_count = local_files.len();
        let remote_count = remote_files.len();
        
        Ok(SyncStatus {
            local_count,
            remote_count,
            local_path: self.local_path.clone(),
        })
    }
}

#[derive(Debug, Default)]
pub struct SyncReport {
    pub uploaded: usize,
    pub downloaded: usize,
    pub deleted: usize,
    pub skipped: usize,
}

pub struct SyncStatus {
    pub local_count: usize,
    pub remote_count: usize,
    pub local_path: PathBuf,
}
