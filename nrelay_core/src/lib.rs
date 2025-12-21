pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/nrelay.control.rs"));
}

pub mod codec;
pub mod error;
pub mod types;

pub use error::NRelayError;
pub use types::*;
