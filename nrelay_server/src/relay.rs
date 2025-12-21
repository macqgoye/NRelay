use crate::admin::{create_admin_router, AdminState};
use crate::config::Config;
use crate::control::handle_client_connection;
use crate::tunnel::{PendingConnection, TunnelRegistry};
use anyhow::Result;
use nrelay_core::TunnelKind;
use nrelay_proto_http::HttpSniffer;
use nrelay_proto_tcp::proxy_bidirectional;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct RelayServer {
    config: Config,
    tunnels: TunnelRegistry,
}

impl RelayServer {
    pub async fn new(config: Config) -> Result<Self> {
        config.admin_token()?;

        Ok(Self {
            config,
            tunnels: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(self) -> Result<()> {
        let client_addr = format!("{}:{}", self.config.bind_addr, self.config.client_port);
        let admin_addr = format!("{}:{}", self.config.bind_addr, self.config.admin_port);

        let admin_state = AdminState {
            tunnels: self.tunnels.clone(),
            admin_token: self.config.admin_token()?,
            public_domain: self.config.public_domain.clone(),
            http_port: 80,
            https_port: 443,
            relay_domain: self.config.relay_domain.clone(),
            client_port: self.config.client_port,
        };

        let admin_router = create_admin_router(admin_state);

        let tunnels_for_control = self.tunnels.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(&client_addr).await.unwrap();
            info!("Client control listening on {}", client_addr);

            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        debug!("New client connection from {}", addr);
                        let tunnels = tunnels_for_control.clone();
                        tokio::spawn(async move {
                            handle_client_connection(socket, tunnels).await;
                        });
                    }
                    Err(e) => error!("Accept error: {}", e),
                }
            }
        });

        let tunnels_for_http = self.tunnels.clone();
        tokio::spawn(async move {
            if let Err(e) = run_http_listener(tunnels_for_http).await {
                error!("HTTP listener error: {}", e);
            }
        });

        info!("Admin API listening on {}", admin_addr);
        let listener = tokio::net::TcpListener::bind(&admin_addr).await?;
        axum::serve(listener, admin_router).await?;

        Ok(())
    }
}

async fn run_http_listener(tunnels: TunnelRegistry) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:80").await?;
    info!("HTTP listener on port 80");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                debug!("HTTP connection from {}", addr);
                let tunnels = tunnels.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_http_connection(socket, tunnels).await {
                        error!("HTTP handler error: {}", e);
                    }
                });
            }
            Err(e) => error!("HTTP accept error: {}", e),
        }
    }
}

async fn handle_http_connection(mut socket: TcpStream, tunnels: TunnelRegistry) -> Result<()> {
    let mut sniffer = HttpSniffer::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }

        sniffer.feed(&buf[..n]);

        if let Some(host) = sniffer.extract_host() {
            debug!("Extracted host: {}", host);

            let tunnel_id = extract_tunnel_id(&host);

            let (request_tx, _control_exists) = {
                let registry = tunnels.read().await;
                registry
                    .values()
                    .find(|s| {
                        s.info.kind == TunnelKind::Http
                            && s.info
                                .public_hostname
                                .as_ref()
                                .map(|h| h.contains(&tunnel_id))
                                .unwrap_or(false)
                    })
                    .and_then(|state| {
                        state
                            .control_conn
                            .as_ref()
                            .map(|c| (c.request_tx.clone(), true))
                    })
                    .unzip()
            };

            if let Some(tx) = request_tx {
                let connection_id = Uuid::new_v4().to_string();
                let (tunnel_tx, mut tunnel_rx) = mpsc::channel(1);

                {
                    let mut registry = tunnels.write().await;
                    if let Some(state) = registry.values_mut().find(|s| {
                        s.info
                            .public_hostname
                            .as_ref()
                            .map(|h| h.contains(&tunnel_id))
                            .unwrap_or(false)
                    }) {
                        state.enqueue_pending(PendingConnection {
                            connection_id: connection_id.clone(),
                            tunnel_tx,
                        });
                    }
                }

                if tx.send(connection_id.clone()).await.is_ok() {
                    if let Some((conn_id, mut tunnel_socket)) = tunnel_rx.recv().await {
                        info!(connection_id = %conn_id, "Starting HTTP proxy");

                        // Write initial data to tunnel socket
                        if let Err(e) = tunnel_socket.write_all(sniffer.consumed_bytes()).await {
                            error!("Failed to write initial data: {}", e);
                            return Ok(());
                        }

                        // Now do bidirectional proxy
                        let (mut client_read, mut client_write) = socket.split();
                        let (mut tunnel_read, mut tunnel_write) = tunnel_socket.split();

                        if let Err(e) = proxy_bidirectional(
                            &mut client_read,
                            &mut client_write,
                            &mut tunnel_read,
                            &mut tunnel_write,
                        )
                        .await
                        {
                            error!("Proxy error: {}", e);
                        }
                    }
                }
            } else {
                warn!("No active tunnel for host: {}", host);
            }

            return Ok(());
        }

        if sniffer.consumed_bytes().len() > 8192 {
            return Ok(());
        }
    }
}

fn extract_tunnel_id(host: &str) -> String {
    host.split('.').next().unwrap_or(host).to_string()
}

pub async fn start_tcp_listener(
    port: u16,
    tunnel_id: String,
    tunnels: TunnelRegistry,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!(tunnel_id = %tunnel_id, port = port, "TCP listener started on port {}", port);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                debug!(tunnel_id = %tunnel_id, "TCP connection from {}", addr);
                let tunnels = tunnels.clone();
                let tunnel_id = tunnel_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_tcp_connection(socket, tunnel_id, tunnels).await {
                        error!("TCP handler error: {}", e);
                    }
                });
            }
            Err(e) => error!("TCP accept error: {}", e),
        }
    }
}

async fn handle_tcp_connection(
    mut socket: TcpStream,
    tunnel_id: String,
    tunnels: TunnelRegistry,
) -> Result<()> {
    let (request_tx, _control_exists) = {
        let registry = tunnels.read().await;
        registry
            .get(&tunnel_id)
            .and_then(|state| {
                state
                    .control_conn
                    .as_ref()
                    .map(|c| (c.request_tx.clone(), true))
            })
            .map(|(tx, exists)| (Some(tx), exists))
            .unwrap_or((None, false))
    };

    if let Some(tx) = request_tx {
        let connection_id = Uuid::new_v4().to_string();
        let (tunnel_tx, mut tunnel_rx) = mpsc::channel(1);

        {
            let mut registry = tunnels.write().await;
            if let Some(state) = registry.get_mut(&tunnel_id) {
                state.enqueue_pending(PendingConnection {
                    connection_id: connection_id.clone(),
                    tunnel_tx,
                });
            }
        }

        if tx.send(connection_id.clone()).await.is_ok() {
            if let Some((conn_id, mut tunnel_socket)) = tunnel_rx.recv().await {
                info!(connection_id = %conn_id, "Starting TCP proxy");

                // Do bidirectional proxy
                let (mut client_read, mut client_write) = socket.split();
                let (mut tunnel_read, mut tunnel_write) = tunnel_socket.split();

                if let Err(e) = proxy_bidirectional(
                    &mut client_read,
                    &mut client_write,
                    &mut tunnel_read,
                    &mut tunnel_write,
                )
                .await
                {
                    error!("Proxy error: {}", e);
                }
            }
        }
    } else {
        warn!(tunnel_id = %tunnel_id, "No active control connection for tunnel");
    }

    Ok(())
}
