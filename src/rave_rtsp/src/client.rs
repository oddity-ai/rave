use crate::error::Result;
use crate::io::AsClient;
use crate::request::Request;
use crate::tokio_codec::Codec;

use futures::SinkExt;

use tokio_stream::StreamExt;

type FramedRead = tokio_util::codec::FramedRead<tokio::net::tcp::OwnedReadHalf, Codec<AsClient>>;
type FramedWrite = tokio_util::codec::FramedWrite<tokio::net::tcp::OwnedWriteHalf, Codec<AsClient>>;

pub struct Client {
    addr: std::net::SocketAddr,
    read: FramedRead,
    write: FramedWrite,
}

impl Client {
    pub async fn connect(addr: std::net::SocketAddr) -> Result<Client> {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        let (read, write) = stream.into_split();
        let read = FramedRead::new(read, Codec::<AsClient>::new());
        let write = FramedWrite::new(write, Codec::<AsClient>::new());
        Ok(Self { addr, read, write })
    }

    #[inline]
    pub async fn connect_with_default_port(ip: std::net::IpAddr) -> Result<Client> {
        Self::connect(std::net::SocketAddr::new(ip, 554)).await
    }

    pub async fn options(&mut self) {
        todo!()
    }

    // TODO: other client calls
}
