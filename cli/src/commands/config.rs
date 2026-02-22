use anyhow::Result;

use crate::config;

pub fn run(server: Option<&str>, device_name: Option<&str>) -> Result<()> {
    let mut cfg = config::load()?;

    if let Some(s) = server {
        cfg.server = s.to_string();
        println!("Server set to: {}", s);
    }

    if let Some(name) = device_name {
        cfg.device_name = Some(name.to_string());
        println!("Device name set to: {}", name);
    }

    config::save(&cfg)?;
    println!("Configuration saved.");

    Ok(())
}
