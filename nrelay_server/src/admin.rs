use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    Json, Router,
};
use nrelay_core::{TunnelConfig, TunnelInfo, TunnelKind};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::relay::start_tcp_listener;
use crate::tunnel::{TunnelRegistry, TunnelState};

#[derive(Clone)]
pub struct AdminState {
    pub tunnels: TunnelRegistry,
    pub admin_token: String,
    pub public_domain: String,
    pub http_port: u16,
    pub https_port: u16,
    pub relay_domain: String,
    pub relay_port: u16,
}

pub fn create_admin_router(state: AdminState) -> Router {
    Router::new()
        .route("/tunnels", axum::routing::post(create_tunnel))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn auth_middleware(
    State(state): State<AdminState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    if let Some(token) = auth_header.and_then(|s| s.strip_prefix("Bearer ")) {
        if token == state.admin_token {
            return Ok(next.run(req).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn create_tunnel(
    State(state): State<AdminState>,
    Json(config): Json<TunnelConfig>,
) -> Result<Json<TunnelInfo>, StatusCode> {
    let mut info = TunnelInfo::new(config.kind);

    match config.kind {
        TunnelKind::Http => {
            info.public_hostname = Some(format!("{}.{}", info.tunnel_id, state.public_domain));
            info.public_port = Some(state.http_port);
            info.exposure_mode = "hostname".to_string();
        }
        TunnelKind::Https | TunnelKind::TlsSni => {
            info.public_hostname = Some(format!("{}.{}", info.tunnel_id, state.public_domain));
            info.public_port = Some(state.https_port);
            info.exposure_mode = "hostname".to_string();
        }
        TunnelKind::Minecraft => {
            info.public_port = config.fixed_public_port.or(Some(25565));
            info.public_hostname = Some(state.public_domain.clone());
            info.exposure_mode = "port".to_string();
        }
        TunnelKind::TcpRaw | TunnelKind::Ssh => {
            info.public_port = config
                .fixed_public_port
                .or_else(|| Some(rand::random::<u16>() % 10000 + 20000));
            info.exposure_mode = "port".to_string();
        }
        TunnelKind::UdpRaw => {
            info.public_port = config
                .fixed_public_port
                .or_else(|| Some(rand::random::<u16>() % 10000 + 30000));
            info.exposure_mode = "port".to_string();
        }
    }

    let tunnel_kind = config.kind;
    let tunnel_state = TunnelState::new(info.clone(), config);

    state
        .tunnels
        .write()
        .await
        .insert(info.tunnel_id.clone(), tunnel_state);

    // Set relay_addr and relay_port
    info.relay_addr = Some(state.relay_domain.clone());
    info.relay_port = Some(state.relay_port);

    info!(
        tunnel_id = %info.tunnel_id,
        kind = ?info.kind,
        hostname = ?info.public_hostname,
        port = ?info.public_port,
        relay_addr = ?info.relay_addr,
        relay_port = ?info.relay_port,
        "Tunnel created"
    );

    // Start TCP listener for TCP-based tunnels
    if matches!(
        tunnel_kind,
        TunnelKind::TcpRaw | TunnelKind::Ssh | TunnelKind::Minecraft
    ) {
        if let Some(port) = info.public_port {
            let tunnels = state.tunnels.clone();
            let tunnel_id = info.tunnel_id.clone();
            tokio::spawn(async move {
                if let Err(e) = start_tcp_listener(port, tunnel_id, tunnels).await {
                    tracing::error!("TCP listener error: {}", e);
                }
            });
        }
    }

    Ok(Json(info))
}
