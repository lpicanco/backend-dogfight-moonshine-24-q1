use std::io;
use std::io::{Error, IoSlice};
use std::pin::Pin;
use std::task::{Context, Poll};
use async_trait::async_trait;
use deadpool::managed;
use deadpool::managed::{Metrics, Object, RecycleError};
use tokio::io::{AsyncRead, AsyncWrite, Interest, ReadBuf};
use tokio::net::TcpStream;

pub struct Manager(String);

pub type Pool = managed::Pool<Manager>;

impl Manager {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self(url.into())
    }
}


pub(crate) struct TcpConnWrapper {
    conn: Object<Manager>,
}

impl TcpConnWrapper {
    pub(crate) fn new(conn: Object<Manager>) -> Self {
        Self { conn }
    }
}

impl AsyncRead for TcpConnWrapper {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.conn).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpConnWrapper {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.conn).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.conn).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut *self.conn).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut *self.conn).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.conn.is_write_vectored()
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = TcpStream;
    type Error = Error;

    async fn create(&self) -> Result<TcpStream, Error> {
        let stream = TcpStream::connect(&self.0).await.unwrap();
        return Ok(stream);

    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if let Ok(ready) = conn.ready(Interest::WRITABLE | Interest::READABLE).await {
            if ready.is_writable() && ready.is_readable() {
                return Ok(());
            }
        }
        Err(RecycleError::Message("Connection is closed".into()).into())
    }
}
