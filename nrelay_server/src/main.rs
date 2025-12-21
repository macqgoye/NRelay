mod admin;
mod config;
mod control;
mod relay;
mod tunnel;

use anyhow::Result;
use clap::Parser;
use config::Config;
use relay::RelayServer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nrelay_server=debug,tower_http=debug".into()),
        )
        .init();
    
    let config = Config::parse();
    
    info!("Starting NRelay server");
    info!("Relay control port: {}", config.relay_port);
    info!("Admin API port: {}", config.admin_port);
    
    let server = RelayServer::new(config).await?;
    server.run().await?;
    
    Ok(())
}