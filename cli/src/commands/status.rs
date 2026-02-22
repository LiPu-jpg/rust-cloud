use anyhow::Result;

use crate::client::Client;
use crate::config;
use crate::sync::SyncEngine;

pub async fn run(server: &str, path: Option<&str>) -> Result<()> {
    let client = Client::new(server);
    
    if !client.health().await? {
        anyhow::bail!("Cannot connect to server at {}", server);
    }
    
    let cfg = config::load()?;
    let sync_path = path
        .map(std::path::PathBuf::from)
        .unwrap_or(cfg.sync_path);
    
    let engine = SyncEngine::new(client, sync_path);
    let status = engine.status().await?;
    
    println!("Sync Status:");
    println!("  Local path:  {:?}", status.local_path);
    println!("  Local files: {}", status.local_count);
    println!("  Remote files: {}", status.remote_count);
    
    Ok(())
}
