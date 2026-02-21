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

use crate::net_err;

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
        async fn try_connect<T: Tokio1ToSocketAddrs>(
            server: T,
            timeout: Option<Duration>,
            local_addr: Option<IpAddr>,
        ) -> Result<TokioTcpStream, Error> {
            let addrs = tokio::net::lookup_host(server)
                .await
                .map_err(crate::net_err::connection)?
                .filter(|resolved_addr| resolved_address_filter(resolved_addr, local_addr));

            let mut last_err = None;

            for addr in addrs {
                let socket = match addr.ip() {
                    IpAddr::V4(_) => TokioTcpSocket::new_v4(),
                    IpAddr::V6(_) => TokioTcpSocket::new_v6(),
                }
                .map_err(crate::net_err::connection)?;
                if let Some(local_addr) = local_addr {
                    socket
                        .bind(SocketAddr::new(local_addr, 0))
                        .map_err(crate::net_err::connection)?;
                }

                let connect_future = socket.connect(addr);
                if let Some(timeout) = timeout {
                    match tokio::time::timeout(timeout, connect_future).await {
                        Ok(Ok(stream)) => return Ok(stream),
                        Ok(Err(err)) => last_err = Some(err),
                        Err(_) => {
                            last_err = Some(io::Error::new(
                                io::ErrorKind::TimedOut,
                                "connection timed out",
                            ));
                        }
                    }
                } else {
                    match connect_future.await {
                        Ok(stream) => return Ok(stream),
                        Err(err) => last_err = Some(err),
                    }
                }
            }

            Err(match last_err {
                Some(last_err) => crate::net_err::connection(last_err),
                None => crate::net_err::connection("could not resolve to any supported address"),
            })
        }

        let tcp_stream = try_connect(server, timeout, local_addr).await?;
        let mut stream =
            AsyncNetworkStream::new(InnerAsyncNetworkStream::Tokio1Tcp(Box::new(tcp_stream)));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
}
