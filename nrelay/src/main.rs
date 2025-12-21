mod cli;
mod config;
mod origin;
mod tunnel;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, OriginCommands, TunnelProtocol};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Origin { command } => match command {
            OriginCommands::List => origin::list_origins().await?,
            OriginCommands::Add {
                id,
                url,
                token,
                kind,
            } => origin::add_origin(&id, &url, &token, kind).await?,
            OriginCommands::Set { id, key, value } => origin::set_origin(&id, &key, &value).await?,
            OriginCommands::Get { id, key } => origin::get_origin(&id, &key).await?,
            OriginCommands::Rm { id } => origin::remove_origin(&id).await?,
            OriginCommands::Use { id } => origin::use_origin(&id).await?,
        },
        Commands::Tcp { target, origin } => {
            tunnel::start_tunnel(TunnelProtocol::TcpRaw, &target, origin.as_deref()).await?
        }
        Commands::Udp { target, origin } => {
            tunnel::start_tunnel(TunnelProtocol::UdpRaw, &target, origin.as_deref()).await?
        }
        Commands::Http { target, origin } => {
            tunnel::start_tunnel(TunnelProtocol::Http, &target, origin.as_deref()).await?
        }
        Commands::Https { target, origin } => {
            tunnel::start_tunnel(TunnelProtocol::Https, &target, origin.as_deref()).await?
        }
        Commands::Mc { target, origin } => {
            tunnel::start_tunnel(TunnelProtocol::Minecraft, &target, origin.as_deref()).await?
        }
        Commands::Sni { target, origin } => {
            tunnel::start_tunnel(TunnelProtocol::TlsSni, &target, origin.as_deref()).await?
        }
    }

    Ok(())
}