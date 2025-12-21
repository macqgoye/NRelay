mod config;
mod control;
mod tunnel;

use anyhow::Result;
use clap::Parser;
use config::Config;
use control::ControlClient;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nrelay_client=debug".into()),
        )
        .init();
    
    let config = Config::parse();
    
    info!("Starting NRelay client");
    info!("Connecting to: {}", config.server_addr);
    info!("Tunnel token: {}...", &config.tunnel_token[..8]);
    info!("Local target: {}:{}", config.local_addr, config.local_port);
    
    let client = ControlClient::new(config);
    client.run().await?;
    
    Ok(())
}