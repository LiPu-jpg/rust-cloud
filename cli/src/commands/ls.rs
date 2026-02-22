use anyhow::Result;

use crate::client::Client;

pub async fn run(server: &str, path: Option<&str>) -> Result<()> {
    let client = Client::new(server);
    
    let files = client.list_files(path).await?;
    
    if files.is_empty() {
        println!("No files found.");
        return Ok(());
    }
    
    println!("{:<40} {:<10} {:<20}", "Name", "Size", "Type");
    println!("{}", "-".repeat(70));
    
    for file in files {
        let file_type = if file.is_dir { "DIR" } else { "FILE" };
        let size = format_size(file.size);
        println!("{:<40} {:<10} {:<20}", file.name, size, file_type);
    }
    
    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 { return "0 B".to_string(); }
    const K: u64 = 1024;
    const SIZES: [&str; 4] = ["B", "KB", "MB", "GB"];
    let i = (bytes as f64).log(K as f64).floor() as usize;
    let i = i.min(SIZES.len() - 1);
    format!("{:.1} {}", bytes as f64 / K.pow(i as u32) as f64, SIZES[i])
}
