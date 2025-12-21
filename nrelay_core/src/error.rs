use thiserror::Error;

#[derive(Error, Debug)]
pub enum NRelayError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Tunnel not found: {0}")]
    TunnelNotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Decode error: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("Encode error: {0}")]
    Encode(#[from] prost::EncodeError),
}
