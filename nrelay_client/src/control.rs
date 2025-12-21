use crate::config::Config;
use crate::tunnel::handle_tunnel_request;
use anyhow::Result;
use nrelay_core::{
    codec::{read_control_message, write_control_message},
    proto::{control_message::Payload, AuthMode, ClientAuth, ControlMessage},
};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

pub struct ControlClient {
    config: Config,
}

impl ControlClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn run(&self) -> Result<()> {
        loop {
            if let Err(e) = self.run_control_connection().await {
                error!("Control connection error: {}", e);
                warn!("Reconnecting in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
    
    async fn run_control_connection(&self) -> Result<()> {
        info!("Connecting to relay server...");
        let mut socket = TcpStream::connect(&self.config.server_addr).await?;
        
        write_control_message(
            &mut socket,
            &ControlMessage {
                payload: Some(Payload::Auth(ClientAuth {
                    mode: AuthMode::AuthControl as i32,
                    tunnel_token: self.config.tunnel_token.clone(),
                })),
            },
        )
        .await?;
        
        let auth_result = read_control_message(&mut socket).await?;
        
        match auth_result.payload {
            Some(Payload::AuthResult(result)) => {
                if result.success {
                    info!(tunnel_id = %result.tunnel_id, "Control connection authenticated");
                } else {
                    return Err(anyhow::anyhow!("Auth failed: {}", result.message));
                }
            }
            _ => return Err(anyhow::anyhow!("Unexpected auth response")),
        }
        
        loop {
            let msg = read_control_message(&mut socket).await?;
            
            match msg.payload {
                Some(Payload::OpenTunnelRequest(req)) => {
                    debug!(
                        tunnel_id = %req.tunnel_id,
                        connection_id = %req.connection_id,
                        "Received tunnel request"
                    );
                    
                    let config = self.config.clone();
                    let server_addr = self.config.server_addr.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = handle_tunnel_request(
                            &server_addr,
                            &config.tunnel_token,
                            &config.local_addr,
                            config.local_port,
                        )
                        .await
                        {
                            error!("Tunnel handler error: {}", e);
                        }
                    });
                }
                _ => {
                    warn!("Unexpected message on control connection");
                }
            }
        }
    }
}