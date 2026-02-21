use std::{
    fmt, io, mem,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::io::{AsyncRead,AsyncWrite};
use futures_io::{
    AsyncRead as FuturesAsyncRead, AsyncWrite as FuturesAsyncWrite, Error as IoError,
    Result as IoResult,
};

enum InnerAsyncNetworkStream {
    #[cfg(feature = "tokio")]
    Tokio1Tcp(Box<dyn AsyncTokioStream>),
}

pub struct AsyncNetworkStream {
    inner: InnerAsyncNetworkStream,
}

#[cfg(feature = "tokio")]
pub trait AsyncTokioStream: AsyncRead + AsyncWrite + Send + Sync + Unpin + fmt::Debug {
    fn peer_addr(&self) -> io::Result<SocketAddr>;
}

#[cfg(feature = "tokio")]
impl AsyncTokioStream for TokioTcpStream {
    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}
use tokio::net::{
    TcpSocket as TokioTcpSocket, TcpStream as TokioTcpStream,
    ToSocketAddrs as Tokio1ToSocketAddrs,
};

pub(crate) enum InnerTlsParameters {
    #[cfg(feature = "native-tls")]
    NativeTls { connector: TlsConnector },
    #[cfg(feature = "rustls")]
    Rustls { config: Arc<ClientConfig> },
    #[cfg(feature = "boring-tls")]
    BoringTls {
        connector: SslConnector,
        accept_invalid_hostnames: bool,
    },
}

pub struct TlsParameters {
    pub(crate) connector: InnerTlsParameters,
    /// The domain name which is expected in the TLS certificate from the server
    pub(super) domain: String,
}

use std::fmt::Error;//заглушка
#[cfg(feature = "tokio")]
pub async fn connect_tokio<T: Tokio1ToSocketAddrs>(
        server: T,
        timeout: Option<Duration>,
        tls_parameters: Option<TlsParameters>,
        local_addr: Option<IpAddr>,
    ) -> Result<AsyncNetworkStream, Error>
{

}