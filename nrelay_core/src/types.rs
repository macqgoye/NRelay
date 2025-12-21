use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelKind {
    Http,
    Https,
    TcpRaw,
    UdpRaw,
    Minecraft,
    Ssh,
    TlsSni,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub kind: TunnelKind,
    pub local_port: u16,
    pub fixed_public_port: Option<u16>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub tunnel_id: String,
    pub access_token: String,
    pub kind: TunnelKind,
    pub public_hostname: Option<String>,
    pub public_port: Option<u16>,
    pub exposure_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_addr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_port: Option<u16>,
}

impl TunnelInfo {
    pub fn new(kind: TunnelKind) -> Self {
        Self {
            tunnel_id: Uuid::new_v4().to_string(),
            access_token: Uuid::new_v4().to_string(),
            kind,
            public_hostname: None,
            public_port: None,
            exposure_mode: String::new(),
            relay_addr: None,
            relay_port: None,
        }
    }
}
