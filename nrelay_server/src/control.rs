use nrelay_core::{
    codec::{read_control_message, write_control_message},
    proto::{
        control_message::Payload, AuthMode, AuthResult, ClientAuth, ControlMessage,
        OpenTunnelRequest, TunnelOk,
    },
    NRelayError,
};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::tunnel::{ControlConnection, TunnelRegistry};

pub async fn handle_client_connection(
    socket: TcpStream,
    tunnels: TunnelRegistry,
) {
    if let Err(e) = handle_client_inner(socket, tunnels).await {
        error!("Control connection error: {}", e);
    }
}

async fn handle_client_inner(
    mut socket: TcpStream,
    tunnels: TunnelRegistry,
) -> Result<(), NRelayError> {
    let msg = read_control_message(&mut socket).await?;
    
    let auth = match msg.payload {
        Some(Payload::Auth(auth)) => auth,
        _ => {
            return Err(NRelayError::Protocol(
                "Expected auth message".to_string(),
            ))
        }
    };
    
    let tunnel_id = validate_auth(&auth, &tunnels).await?;
    
    match auth.mode() {
        AuthMode::AuthControl => {
            handle_control_connection(&mut socket, tunnel_id, tunnels).await
        }
        AuthMode::AuthTunnel => {
            handle_tunnel_connection(socket, tunnel_id, tunnels).await
        }
    }
}

async fn validate_auth(
    auth: &ClientAuth,
    tunnels: &TunnelRegistry,
) -> Result<String, NRelayError> {
    let registry = tunnels.read().await;
    
    for (tunnel_id, state) in registry.iter() {
        if state.info.access_token == auth.tunnel_token {
            return Ok(tunnel_id.clone());
        }
    }
    
    Err(NRelayError::Auth("Invalid token".to_string()))
}

async fn handle_control_connection(
    socket: &mut TcpStream,
    tunnel_id: String,
    tunnels: TunnelRegistry,
) -> Result<(), NRelayError> {
    write_control_message(
        socket,
        &ControlMessage {
            payload: Some(Payload::AuthResult(AuthResult {
                success: true,
                message: "Control authenticated".to_string(),
                tunnel_id: tunnel_id.clone(),
            })),
        },
    )
    .await?;
    
    info!(tunnel_id = %tunnel_id, "Control connection authenticated");
    
    let (request_tx, mut request_rx) = mpsc::channel::<String>(32);
    
    {
        let mut registry = tunnels.write().await;
        if let Some(state) = registry.get_mut(&tunnel_id) {
            state.set_control_connection(ControlConnection {
                tunnel_id: tunnel_id.clone(),
                request_tx,
            });
        }
    }
    
    while let Some(connection_id) = request_rx.recv().await {
        debug!(tunnel_id = %tunnel_id, connection_id = %connection_id, "Sending open tunnel request");
        
        if let Err(e) = write_control_message(
            socket,
            &ControlMessage {
                payload: Some(Payload::OpenTunnelRequest(OpenTunnelRequest {
                    tunnel_id: tunnel_id.clone(),
                    connection_id,
                })),
            },
        )
        .await
        {
            error!("Failed to send open tunnel request: {}", e);
            break;
        }
    }
    
    tunnels.write().await.get_mut(&tunnel_id).map(|s| {
        s.remove_control_connection();
    });
    
    Ok(())
}

async fn handle_tunnel_connection(
    mut socket: TcpStream,
    tunnel_id: String,
    tunnels: TunnelRegistry,
) -> Result<(), NRelayError> {
    let connection_id = Uuid::new_v4().to_string();
    
    write_control_message(
        &mut socket,
        &ControlMessage {
            payload: Some(Payload::TunnelOk(TunnelOk {
                connection_id: connection_id.clone(),
            })),
        },
    )
    .await?;
    
    debug!(tunnel_id = %tunnel_id, connection_id = %connection_id, "Tunnel connection authenticated");
    
    let pending_socket = {
        let mut registry = tunnels.write().await;
        let state = registry
            .get_mut(&tunnel_id)
            .ok_or_else(|| NRelayError::TunnelNotFound(tunnel_id.clone()))?;
        
        state.dequeue_pending()
    };
    
    if let Some(pending) = pending_socket {
        info!(
            tunnel_id = %tunnel_id,
            connection_id = %connection_id,
            "Proxying connection"
        );
        
        if pending.tunnel_tx.send((connection_id, socket)).await.is_err() {
            warn!("Failed to send socket to tunnel handler");
        }
    } else {
        warn!(tunnel_id = %tunnel_id, "No pending connection found");
    }
    
    Ok(())
}