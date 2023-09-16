use std::{pin::Pin, task::Poll};

use common::common::{DynamicRead, DynamicWrite};
use tokio::io::{AsyncRead, ReadBuf};

pub async fn handle_encryption<'a>(
    rstream: Box<DynamicRead<'a>>,
    wstream: Box<DynamicWrite<'a>>,
) -> Result<(Box<DynamicRead<'a>>, Box<DynamicWrite<'a>>), Box<dyn std::error::Error + Send + Sync>>
{
    Ok((
        Box::new(EncryptedReadStream::new(rstream)),
        Box::new(wstream),
    ))
}

struct EncryptedReadStream<'a> {
    stream: Box<DynamicRead<'a>>,
}

impl<'a> EncryptedReadStream<'a> {
    pub fn new(stream: Box<DynamicRead<'a>>) -> Self {
        Self { stream }
    }
}

impl<'a> AsyncRead for EncryptedReadStream<'a> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut rbuf = vec![];
        let mut rbuf = ReadBuf::new(&mut rbuf);
        let status = AsyncRead::poll_read(Pin::new(&mut self.stream), cx, &mut rbuf)?;
        return match status {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => {
                buf.put_slice(rbuf.filled());
                Poll::Ready(std::io::Result::Ok(()))
            }
        };
    }
}
