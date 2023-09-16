use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

pub async fn handle_encryption(
    rstream: OwnedReadHalf,
    wstream: OwnedWriteHalf,
) -> Result<
    (
        Box<dyn AsyncRead + Unpin + Send + Sync>,
        Box<dyn AsyncWrite + Unpin + Send + Sync>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    Ok((Box::new(rstream), Box::new(wstream)))
}
