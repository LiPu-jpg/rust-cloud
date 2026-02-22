use anyhow::Result;

use crate::client::Client;
use crate::config;
use crate::sync::SyncEngine;

pub async fn run(server: &str, path: Option<&str>, dry_run: bool) -> Result<()> {
    let client = Client::new(server);
    
    if !client.health().await? {
        anyhow::bail!("Cannot connect to server at {}", server);
    }
    
    let cfg = config::load()?;
    let sync_path = path
        .map(std::path::PathBuf::from)
        .unwrap_or(cfg.sync_path);
    
    if !sync_path.exists() {
        std::fs::create_dir_all(&sync_path)?;
        println!("Created sync directory: {:?}", sync_path);
    }
    
    let engine = SyncEngine::new(client, sync_path);
    
    println!("Starting sync{}...", if dry_run { " (dry run)" } else { "" });
    let report = engine.sync(dry_run).await?;
    
    println!("\nSync completed:");
    println!("  Uploaded:  {}", report.uploaded);
    println!("  Downloaded: {}", report.downloaded);
    println!("  Deleted:    {}", report.deleted);
    println!("  Skipped:    {}", report.skipped);
    
    Ok(())
}
