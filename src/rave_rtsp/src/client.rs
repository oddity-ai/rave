use crate::error::Result;
use crate::io::AsClient;
use crate::message::Uri;
use crate::request::Request;
use crate::tokio_codec::Codec;

use futures::SinkExt;

use tokio_stream::StreamExt;

type FramedRead = tokio_util::codec::FramedRead<tokio::net::tcp::OwnedReadHalf, Codec<AsClient>>;
type FramedWrite = tokio_util::codec::FramedWrite<tokio::net::tcp::OwnedWriteHalf, Codec<AsClient>>;

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
            scheme,
            authority,
            path_and_query,
            ..
        } = uri.into_parts();
        if let Some("rtsp") = scheme {
            todo!()
        } else {
            todo!() // ERR
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
            read,
            write,
            sequencer: Sequencer::new(),
            session: None,
        })
    }

    pub async fn options(&mut self) {
        // TODO: cseq sequencer
        // TODO: let request = Request::options(uri, self.sequencer.sequence())
        todo!()
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
