use anyhow::Result;
use std::path::PathBuf;

use crate::client::Client;

pub async fn run(server: &str, remote_path: &str, local_path: Option<&str>) -> Result<()> {
    let client = Client::new(server);
    
    println!("Downloading {}...", remote_path);
    
    let content = client.download_file(remote_path).await?;
    
    let local = local_path
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(remote_path.rsplit('/').next().unwrap_or(remote_path))
        });
    
    if let Some(parent) = local.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    tokio::fs::write(&local, &content).await?;
    
    println!("Downloaded successfully!");
    println!("  Saved to: {:?}", local);
    println!("  Size: {} bytes", content.len());
    
    Ok(())
}
