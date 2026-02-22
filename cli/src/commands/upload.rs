use anyhow::Result;
use std::path::Path;

use crate::client::Client;

pub async fn run(server: &str, local_path: &str, remote_path: Option<&str>) -> Result<()> {
    let client = Client::new(server);
    
    let path = Path::new(local_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", local_path);
    }
    
    let content = tokio::fs::read(path).await?;
    let remote = remote_path.unwrap_or(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
    );
    
    println!("Uploading {} -> {}...", local_path, remote);
    
    let info = client.upload_file(remote, &content).await?;
    
    println!("Uploaded successfully!");
    println!("  Path: {}", info.path);
    println!("  Size: {} bytes", info.size);
    if let Some(hash) = info.hash {
        println!("  Hash: {}...", &hash[..12]);
    }
    
    Ok(())
}
