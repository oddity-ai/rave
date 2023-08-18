use std::str::FromStr;

use crate::error::Result;
use crate::io::AsClient;
use crate::message::{Method, StatusCategory, Uri};
use crate::request::Request;
use crate::response::Response;
use crate::tokio_codec::Codec;
use crate::MaybeInterleaved;

use futures::SinkExt;

use tokio_stream::StreamExt;

type FramedRead = tokio_util::codec::FramedRead<tokio::net::tcp::OwnedReadHalf, Codec<AsClient>>;
type FramedWrite = tokio_util::codec::FramedWrite<tokio::net::tcp::OwnedWriteHalf, Codec<AsClient>>;

/// RTSP client.
///
/// Communicate with RTSP servers. The [`Client`] handles request and response,
/// serialization and deserialization, redirection and error handling.
///
/// # Example
///
/// ```
/// let client = Client::connect("rtsp://localhost/stream").await.unwrap();
/// println!("{:?}", client.options().await.unwrap());
/// ```
pub struct Client {
    addr: std::net::SocketAddr,
    uri: Uri,
    read: FramedRead,
    write: FramedWrite,
    sequencer: Sequencer,
    session: Option<String>,
}

impl Client {
    pub async fn connect(uri: &Uri) -> Result<Client> {
        let http::uri::Parts {
            scheme, authority, ..
        } = uri.clone().into_parts();
        // TODO: handle error
        let authority = authority.unwrap();
        match scheme {
            Some(scheme) if scheme.as_str() == "rtsp" => {
                let host = authority.host();
                let port = authority.port_u16().unwrap_or(554);
                let mut addrs = tokio::net::lookup_host((host, port)).await?;
                let addr = addrs.next().unwrap(); // TODO: handle error
                Self::connect_inner(addr, uri.clone()).await
            }
            _ => {
                // TODO: error
                todo!()
            }
        }
    }

    pub async fn connect_with_host(
        socket_addr: std::net::SocketAddr,
        path: &str,
    ) -> Result<Client> {
        let uri = format!("rtsp://{}/{}", socket_addr, path)
            .parse::<Uri>()
            .unwrap();
        Self::connect_inner(socket_addr, uri).await
    }

    pub async fn connect_with_host_and_default_port(
        ip: std::net::IpAddr,
        path: &str,
    ) -> Result<Client> {
        let uri = format!("rtsp://{}/{}", ip, path).parse::<Uri>().unwrap();
        Self::connect_inner(std::net::SocketAddr::new(ip, 554), uri).await
    }

    async fn connect_inner(addr: std::net::SocketAddr, uri: Uri) -> Result<Client> {
        // TODO: url path
        let stream = tokio::net::TcpStream::connect(addr).await?;
        let (read, write) = stream.into_split();
        let read = FramedRead::new(read, Codec::<AsClient>::new());
        let write = FramedWrite::new(write, Codec::<AsClient>::new());
        Ok(Self {
            addr,
            uri,
            read,
            write,
            sequencer: Sequencer::new(),
            session: None,
        })
    }

    pub async fn options(&mut self) -> Result<Vec<Method>> {
        let cseq = self.sequencer.sequence();
        self.send(Request::options(&self.uri, cseq)).await?;
        let response = self.receive().await?;
        Ok(response
            .headers
            .get("Public")
            .unwrap_or("")
            .split(',')
            // parse methods, trimming each method string, and leaving out
            // invalid methods that could not be parsed
            .filter_map(|method| Method::from_str(method.trim()).ok())
            .collect())
    }

    #[inline]
    async fn send(&mut self, request: Request) -> Result<()> {
        self.write.send(request.into()).await
    }

    #[inline]
    async fn receive(&mut self) -> Result<Response> {
        // TODO: impl is quite easy: just handle unexpected None and interleaved as errors
        let response = match self.read.next().await {
            Some(Ok(MaybeInterleaved::Message(response))) => Ok(response),
            Some(Ok(MaybeInterleaved::Interleaved { .. })) => Err(todo!()),
            Some(Err(err)) => Err(err),
            None => Err(todo!()),
        }?;
        // TODO: handle redirection
        if response.status() == StatusCategory::Success {
            Ok(response)
        } else {
            Err(todo!())
        }
    }

    // TODO: other client calls
}

pub struct Sequencer {
    sequence_number: usize,
}

impl Sequencer {
    pub fn new() -> Self {
        Self { sequence_number: 0 }
    }

    #[inline]
    pub fn sequence(&mut self) -> usize {
        let sequence_number = self.sequence_number;
        self.sequence_number += 1;
        sequence_number
    }
}

impl Default for Sequencer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
