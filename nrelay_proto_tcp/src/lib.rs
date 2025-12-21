use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::io;

pub async fn proxy_bidirectional<R1, W1, R2, W2>(
    mut read1: R1,
    mut write1: W1,
    mut read2: R2,
    mut write2: W2,
) -> io::Result<()>
where
    R1: AsyncRead + Unpin,
    W1: AsyncWrite + Unpin,
    R2: AsyncRead + Unpin,
    W2: AsyncWrite + Unpin,
{
    let (mut r1, mut w1) = (&mut read1, &mut write1);
    let (mut r2, mut w2) = (&mut read2, &mut write2);
    
    tokio::select! {
        res = copy_with_backpressure(&mut r1, &mut w2) => res,
        res = copy_with_backpressure(&mut r2, &mut w1) => res,
    }
}

async fn copy_with_backpressure<R, W>(reader: &mut R, writer: &mut W) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; 8192];
    
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        
        writer.write_all(&buf[..n]).await?;
        writer.flush().await?;
    }
}