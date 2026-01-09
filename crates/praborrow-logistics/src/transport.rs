use async_trait::async_trait;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::raw::RawResource;
use std::net::SocketAddr;

/// Abstract Transport Layer.
/// In Year 4, this abstracts over IOCP (Windows) and io_uring (Linux).
#[async_trait]
pub trait Transport {
    async fn send(&mut self, data: &[u8]) -> std::io::Result<()>;
    async fn recv(&mut self, buffer: &mut [u8]) -> std::io::Result<usize>;
}

/// A real Tokio-based Transport.
pub struct TokioTransport {
    stream: TcpStream,
}

impl TokioTransport {
    pub async fn connect(addr: SocketAddr) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl Transport for TokioTransport {
    async fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(data).await
    }

    async fn recv(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buffer).await
    }
}

/// Zero-Copy extension for transporting RawResources.
/// This mock implementation just extracts the data, but real impl would use gather/scatter IO.
impl TokioTransport {
    pub async fn send_raw<T: AsRef<[u8]>>(&mut self, resource: RawResource<T>) -> std::io::Result<()> {
        let inner = resource.into_inner(); 
        self.stream.write_all(inner.as_ref()).await
    }
}
