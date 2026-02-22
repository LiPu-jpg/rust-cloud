use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub hash: Option<String>,
    pub version: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub last_seen: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub path: String,
    pub hash: Option<String>,
    pub size: u64,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPlanItem {
    pub file_id: String,
    pub path: String,
    pub action: String,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Client {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/api/health", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let result: ApiResponse<serde_json::Value> = resp.json().await?;
        Ok(result.success)
    }

    pub async fn list_files(&self, path: Option<&str>) -> Result<Vec<FileInfo>> {
        let url = format!("{}/api/files", self.base_url);
        let mut req = self.http.get(&url);
        
        if let Some(p) = path {
            req = req.query(&[("path", p)]);
        }
        
        let resp = req.send().await?;
        let result: ApiResponse<Vec<FileInfo>> = resp.json().await?;
        result.data.ok_or_else(|| anyhow::anyhow!("No data in response"))
    }

    pub async fn register_device(&self, name: &str) -> Result<Device> {
        let url = format!("{}/api/devices", self.base_url);
        let resp = self.http
            .post(&url)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await?;
        let result: ApiResponse<Device> = resp.json().await?;
        result.data.ok_or_else(|| anyhow::anyhow!("Failed to register device"))
    }

    pub async fn upload_file(&self, path: &str, content: &[u8]) -> Result<FileInfo> {
        let url = format!("{}/api/files/{}", self.base_url, path);
        let resp = self.http
            .put(&url)
            .body(content.to_vec())
            .send()
            .await?;
        let result: ApiResponse<FileInfo> = resp.json().await?;
        result.data.ok_or_else(|| anyhow::anyhow!("Failed to upload file"))
    }

    pub async fn download_file(&self, path: &str) -> Result<Vec<u8>> {
        let url = format!("{}/api/files/{}", self.base_url, path);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn create_folder(&self, path: &str) -> Result<FileInfo> {
        let url = format!("{}/api/files", self.base_url);
        let resp = self.http
            .post(&url)
            .json(&serde_json::json!({ "path": path }))
            .send()
            .await?;
        let result: ApiResponse<FileInfo> = resp.json().await?;
        result.data.ok_or_else(|| anyhow::anyhow!("Failed to create folder"))
    }

    pub async fn delete_file(&self, path: &str) -> Result<bool> {
        let url = format!("{}/api/files/{}", self.base_url, path);
        let resp = self.http.delete(&url).send().await?;
        let result: ApiResponse<bool> = resp.json().await?;
        Ok(result.success)
    }

    pub async fn create_sync_plan(&self, local_files: &[FileRecord]) -> Result<Vec<SyncPlanItem>> {
        let url = format!("{}/api/sync/plan", self.base_url);
        let resp = self.http
            .post(&url)
            .json(&serde_json::json!({ "local_files": local_files }))
            .send()
            .await?;
        let result: ApiResponse<Vec<SyncPlanItem>> = resp.json().await?;
        result.data.ok_or_else(|| anyhow::anyhow!("Failed to create sync plan"))
    }

    pub async fn execute_sync(&self, file_id: &str, device_id: &str, action: &str) -> Result<bool> {
        let url = format!("{}/api/sync/execute", self.base_url);
        let resp = self.http
            .post(&url)
            .json(&serde_json::json!({
                "file_id": file_id,
                "device_id": device_id,
                "action": action
            }))
            .send()
            .await?;
        let result: ApiResponse<bool> = resp.json().await?;
        Ok(result.success)
    }

    pub async fn list_versions(&self) -> Result<Vec<FileRecord>> {
        let url = format!("{}/api/versions", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let result: ApiResponse<Vec<FileRecord>> = resp.json().await?;
        result.data.ok_or_else(|| anyhow::anyhow!("No data in response"))
    }
}
