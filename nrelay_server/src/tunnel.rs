use nrelay_core::{TunnelConfig, TunnelInfo};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

pub type TunnelRegistry = Arc<RwLock<HashMap<String, TunnelState>>>;

#[derive(Debug)]
pub struct TunnelState {
    pub info: TunnelInfo,
    pub config: TunnelConfig,
    pub control_conn: Option<ControlConnection>,
    pub pending_connections: VecDeque<PendingConnection>,
}

#[derive(Debug)]
pub struct ControlConnection {
    pub tunnel_id: String,
    pub request_tx: mpsc::Sender<String>,
}

#[derive(Debug)]
pub struct PendingConnection {
    pub connection_id: String,
    pub tunnel_tx: mpsc::Sender<(String, TcpStream)>,
}

impl TunnelState {
    pub fn new(info: TunnelInfo, config: TunnelConfig) -> Self {
        Self {
            info,
            config,
            control_conn: None,
            pending_connections: VecDeque::new(),
        }
    }

    pub fn set_control_connection(&mut self, conn: ControlConnection) {
        let tunnel_id = conn.tunnel_id.clone();
        if self.control_conn.is_some() {
            warn!(tunnel_id = %tunnel_id, "Replacing existing control connection");
        }
        self.control_conn = Some(conn);
        info!(tunnel_id = %tunnel_id, "Control connection established");
    }

    pub fn remove_control_connection(&mut self) {
        self.control_conn = None;
        debug!("Control connection removed");
    }

    pub fn enqueue_pending(&mut self, conn: PendingConnection) {
        debug!(connection_id = %conn.connection_id, "Enqueueing pending connection");
        self.pending_connections.push_back(conn);
    }

    pub fn dequeue_pending(&mut self) -> Option<PendingConnection> {
        self.pending_connections.pop_front()
    }
}
