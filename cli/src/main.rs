use clap::{Parser, Subcommand};
use anyhow::Result;

mod client;
mod commands;
mod config;
mod sync;

#[derive(Parser)]
#[command(name = "rcloud")]
#[command(about = "RustCloud CLI - File sync client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    server: Option<String>,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Sync files with server")]
    Sync {
        #[arg(short, long)]
        path: Option<String>,
        
        #[arg(short, long)]
        dry_run: bool,
    },

    #[command(about = "Show sync status")]
    Status {
        #[arg(short, long)]
        path: Option<String>,
    },

    #[command(about = "Configure client")]
    Config {
        #[arg(short, long)]
        server: Option<String>,
        
        #[arg(short, long)]
        device_name: Option<String>,
    },

    #[command(about = "List remote files")]
    Ls {
        #[arg(short, long)]
        path: Option<String>,
    },

    #[command(about = "Upload a file")]
    Upload {
        #[arg(short, long)]
        path: String,
        
        #[arg(short, long)]
        remote_path: Option<String>,
    },

    #[command(about = "Download a file")]
    Download {
        #[arg(short, long)]
        remote_path: String,
        
        #[arg(short, long)]
        local_path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt::init();
    }

    let config = config::load()?;
    let server = cli.server.unwrap_or(config.server);

    match cli.command {
        Commands::Sync { path, dry_run } => {
            commands::sync::run(&server, path.as_deref(), dry_run).await?;
        }
        Commands::Status { path } => {
            commands::status::run(&server, path.as_deref()).await?;
        }
        Commands::Config { server: new_server, device_name } => {
            commands::config::run(new_server.as_deref(), device_name.as_deref())?;
        }
        Commands::Ls { path } => {
            commands::ls::run(&server, path.as_deref()).await?;
        }
        Commands::Upload { path, remote_path } => {
            commands::upload::run(&server, &path, remote_path.as_deref()).await?;
        }
        Commands::Download { remote_path, local_path } => {
            commands::download::run(&server, &remote_path, local_path.as_deref()).await?;
        }
    }

    Ok(())
}
