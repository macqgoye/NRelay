use anyhow::Result;
use nrelay_core::{
    codec::{read_control_message, write_control_message},
    proto::{control_message::Payload, AuthMode, ClientAuth, ControlMessage},
};
use nrelay_proto_tcp::proxy_bidirectional;
use tokio::net::TcpStream;
use tracing::{debug, error, info};

pub async fn handle_tunnel_request(
    server_addr: &str,
    tunnel_token: &str,
    local_addr: &str,
    local_port: u16,
) -> Result<()> {
    debug!("Opening tunnel connection to relay");
    let mut relay_socket = TcpStream::connect(server_addr).await?;

    write_control_message(
        &mut relay_socket,
        &ControlMessage {
            payload: Some(Payload::Auth(ClientAuth {
                mode: AuthMode::AuthTunnel as i32,
                tunnel_token: tunnel_token.to_string(),
            })),
        },
    )
    .await?;

    let tunnel_ok = read_control_message(&mut relay_socket).await?;

    let connection_id = match tunnel_ok.payload {
        Some(Payload::TunnelOk(ok)) => ok.connection_id,
        _ => return Err(anyhow::anyhow!("Expected TUNNEL_OK")),
    };

    info!(connection_id = %connection_id, "Tunnel connection established");

    let local_target = format!("{}:{}", local_addr, local_port);
    debug!("Connecting to local service: {}", local_target);

    let mut local_socket = TcpStream::connect(&local_target).await?;

    info!(connection_id = %connection_id, "Proxying traffic");

    let (relay_read, relay_write) = relay_socket.split();
    let (local_read, local_write) = local_socket.split();

    if let Err(e) = proxy_bidirectional(relay_read, relay_write, local_read, local_write).await {
        error!(connection_id = %connection_id, "Proxy error: {}", e);
    }

    info!(connection_id = %connection_id, "Tunnel closed");

    Ok(())
}
