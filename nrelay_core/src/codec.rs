use crate::error::NRelayError;
use crate::proto::ControlMessage;
use bytes::BytesMut;
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::debug;

pub async fn read_control_message<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<ControlMessage, NRelayError> {
    let len = reader.read_u32().await? as usize;
    debug!("Reading control message, size: {} bytes", len);

    // Validate message size - reasonable limit for control messages
    const MAX_MESSAGE_SIZE: usize = 64 * 1024; // 64KB should be more than enough for control messages
    if len > MAX_MESSAGE_SIZE {
        return Err(NRelayError::Protocol(format!(
            "Message too large: {} bytes (max: {})",
            len, MAX_MESSAGE_SIZE
        )));
    }

    if len == 0 {
        return Err(NRelayError::Protocol("Message size is zero".to_string()));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;

    ControlMessage::decode(&buf[..]).map_err(Into::into)
}

pub async fn write_control_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &ControlMessage,
) -> Result<(), NRelayError> {
    let mut buf = BytesMut::new();
    msg.encode(&mut buf)?;

    let len = buf.len() as u32;
    debug!("Writing control message, size: {} bytes", len);
    writer.write_u32(len).await?;
    writer.write_all(&buf).await?;
    writer.flush().await?;

    Ok(())
}
