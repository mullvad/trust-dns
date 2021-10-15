//! Runtime abstraction component for DNS clients
use crate::{error::ProtoError, tcp::DnsTcpStream, udp::UdpSocket, Time};
use std::{future::Future, io, net::SocketAddr};

/// RuntimeProvider defines which async runtime that handles IO and timers.
#[async_trait::async_trait]
pub trait RuntimeProvider: Clone + 'static + Send + Sync + Unpin {
    /// Time implementation used for this type
    type Time: Time + Unpin + Send;
    /// Type of socket that would be bound by the trait implementation. E.g. for tokio, it would be
    /// `tokio::net::UdpSocket`.
    type UdpSocket: UdpSocket + Send + 'static;

    /// Socket type that is returned after a successful connection.
    type TcpConnection: DnsTcpStream;

    /// Bind an UDP socket to the given socket address.
    async fn bind_udp(&self, addr: SocketAddr) -> io::Result<Self::UdpSocket>;

    /// Create a socket and connect to the specified socket address.
    async fn connect_tcp(self, addr: SocketAddr) -> io::Result<Self::TcpConnection>;

    /// Spawn a future on the given runtime.
    fn spawn_bg<F>(&mut self, future: F)
    where
        F: Future<Output = Result<(), ProtoError>> + Send + 'static;
}

#[cfg(feature = "tokio-runtime")]
pub use tokio_runtime::TokioRuntime;

#[cfg(feature = "tokio-runtime")]
mod tokio_runtime {
    use super::*;
    use crate::iocompat::AsyncIoTokioAsStd;

    /// An implementation of a runtime provider using the tokio runtime.
    #[derive(Clone, Default, Copy)]
    pub struct TokioRuntime;

    #[async_trait::async_trait]
    impl RuntimeProvider for TokioRuntime {
        type Time = crate::TokioTime;
        type UdpSocket = tokio::net::UdpSocket;
        type TcpConnection = AsyncIoTokioAsStd<tokio::net::TcpStream>;

        async fn bind_udp(&self, addr: std::net::SocketAddr) -> std::io::Result<Self::UdpSocket> {
            Self::UdpSocket::bind(addr).await
        }

        async fn connect_tcp(
            self,
            addr: std::net::SocketAddr,
        ) -> std::io::Result<Self::TcpConnection> {
            tokio::net::TcpStream::connect(addr)
                .await
                .map(AsyncIoTokioAsStd)
        }

        fn spawn_bg<F>(&mut self, future: F)
        where
            F: Future<Output = Result<(), ProtoError>> + Send + 'static,
        {
            let _join = tokio::spawn(future);
        }
    }
}
