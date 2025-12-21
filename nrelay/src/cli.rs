use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "nrelay")]
#[command(about = "NRelay - Reverse tunnel management CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage origins (servers and services)
    Origin {
        #[command(subcommand)]
        command: OriginCommands,
    },
    /// Create TCP tunnel
    Tcp {
        /// Target address (port or addr:port)
        target: String,
        /// Origin to use (default: configured default)
        #[arg(long)]
        origin: Option<String>,
    },
    /// Create UDP tunnel
    Udp {
        /// Target address (port or addr:port)
        target: String,
        /// Origin to use (default: configured default)
        #[arg(long)]
        origin: Option<String>,
    },
    /// Create HTTP tunnel
    Http {
        /// Target address (port or addr:port)
        target: String,
        /// Origin to use (default: configured default)
        #[arg(long)]
        origin: Option<String>,
    },
    /// Create HTTPS tunnel
    Https {
        /// Target address (port or addr:port)
        target: String,
        /// Origin to use (default: configured default)
        #[arg(long)]
        origin: Option<String>,
    },
    /// Create Minecraft tunnel
    Mc {
        /// Target address (port or addr:port)
        target: String,
        /// Origin to use (default: configured default)
        #[arg(long)]
        origin: Option<String>,
    },
    /// Create TLS SNI tunnel
    Sni {
        /// Target address (port or addr:port)
        target: String,
        /// Origin to use (default: configured default)
        #[arg(long)]
        origin: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum OriginCommands {
    /// List all configured origins
    List,
    /// Add a new origin
    Add {
        /// Origin identifier
        id: String,
        /// Origin URL
        #[arg(long)]
        url: String,
        /// Access token
        #[arg(long)]
        token: String,
        /// Origin kind (server or service)
        #[arg(long, default_value = "service")]
        kind: OriginKind,
        /// Optional relay URL to use instead of server-provided relay address
        #[arg(long)]
        relay_url: Option<String>,
    },
    /// Set origin configuration value
    Set {
        /// Origin identifier
        id: String,
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get origin configuration value
    Get {
        /// Origin identifier
        id: String,
        /// Configuration key
        key: String,
    },
    /// Remove an origin
    Rm {
        /// Origin identifier
        id: String,
    },
    /// Set default origin
    Use {
        /// Origin identifier
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OriginKind {
    Server,
    Service,
}

impl std::str::FromStr for OriginKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "server" => Ok(OriginKind::Server),
            "service" => Ok(OriginKind::Service),
            _ => Err(format!(
                "Invalid kind: {}. Must be 'server' or 'service'",
                s
            )),
        }
    }
}

impl std::fmt::Display for OriginKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginKind::Server => write!(f, "server"),
            OriginKind::Service => write!(f, "service"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TunnelProtocol {
    TcpRaw,
    UdpRaw,
    Http,
    Https,
    Minecraft,
    TlsSni,
}

impl TunnelProtocol {
    pub fn as_str(&self) -> &str {
        match self {
            TunnelProtocol::TcpRaw => "tcp_raw",
            TunnelProtocol::UdpRaw => "udp_raw",
            TunnelProtocol::Http => "http",
            TunnelProtocol::Https => "https",
            TunnelProtocol::Minecraft => "minecraft",
            TunnelProtocol::TlsSni => "tls_sni",
        }
    }
}
