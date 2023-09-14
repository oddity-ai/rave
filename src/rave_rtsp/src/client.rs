use std::str::FromStr;

use crate::error::Error;
use crate::interleaved::{MaybeInterleaved, RequestMaybeInterleaved};
use crate::io::AsClient;
use crate::message::{status_from_code, Headers, Message, Method, StatusCategory, Uri};
use crate::request::{Request, RequestMetadata};
use crate::response::Response;
use crate::tokio_codec::Codec;

use rave_sdp::Sdp;

use futures::SinkExt;

use tokio_stream::StreamExt;

type Result<T> = std::result::Result<T, ClientError>;

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
        let authority = authority.ok_or(ClientError::UriMissingAuthority)?;
        match scheme {
            Some(scheme) if scheme.as_str() == "rtsp" => {
                let host = authority.host();
                let port = authority.port_u16().unwrap_or(554);
                let addrs = tokio::net::lookup_host((host, port))
                    .await?
                    .collect::<Vec<_>>();
                if !addrs.is_empty() {
                    let mut errors = Vec::new();
                    for addr in addrs {
                        match Self::connect_inner(addr, uri.clone()).await {
                            Ok(client) => return Ok(client),
                            Err(err) => errors.push(err),
                        }
                    }
                    Err(ClientError::Connect { errors })
                } else {
                    Err(ClientError::Resolve {
                        name: host.to_string(),
                    })
                }
            }
            Some(scheme) => Err(ClientError::UriUnsupportedProtocolScheme {
                scheme: scheme.to_string(),
            }),
            None => Err(ClientError::UriMissingProtocolScheme),
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
        let stream = tokio::net::TcpStream::connect(addr).await?;
        let (read, write) = stream.into_split();
        let read = FramedRead::new(read, Codec::<AsClient>::new());
        let write = FramedWrite::new(write, Codec::<AsClient>::new());
        Ok(Self {
            uri,
            read,
            write,
            sequencer: Sequencer::new(),
            session: None,
        })
    }

    pub async fn options(&mut self) -> Result<Vec<Method>> {
        let response = self.request(Method::Options, Headers::new()).await?;
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

    pub async fn describe(&mut self) -> Result<Sdp> {
        let response = self.request(Method::Describe, Headers::new()).await?;
        if let Some(body) = response.body {
            // sdp is always UTF-8 (RFC 2327, 6)
            Ok(Sdp::parse(&String::from_utf8_lossy(&body))?)
        } else {
            Err(ClientError::MissingSdp)
        }
    }

    // TODO: other client calls

    async fn request(&mut self, method: Method, headers: Headers) -> Result<Response> {
        for _request_count in 0..20 {
            let response = self
                .request_without_redirect_handling(method, headers.clone())
                .await?;
            match response.status() {
                StatusCategory::Success => return Ok(response),
                StatusCategory::Redirection => {
                    let location = response
                        .headers
                        .get("Location")
                        .ok_or(ClientError::InvalidRedirect)?;
                    // replace path and query in request URI with contents of location
                    // header (assuming it parses correctly)
                    let mut request_uri_parts = self.uri.clone().into_parts();
                    request_uri_parts.path_and_query = Some(
                        location
                            .parse::<http::uri::PathAndQuery>()
                            .map_err(|_| ClientError::InvalidRedirect)?,
                    );
                    self.uri = Uri::from_parts(request_uri_parts)
                        .map_err(|_| ClientError::InvalidRedirect)?;
                    continue;
                }
                _ => return Err(ClientError::Status(response)),
            }
        }
        Err(ClientError::MaximumNumberOfRedirectsReached)
    }

    async fn request_without_redirect_handling(
        &mut self,
        method: Method,
        additional_headers: Headers,
    ) -> Result<Response> {
        let cseq = self.sequencer.sequence();
        let mut headers = match self.session.as_ref() {
            Some(session) => Headers::with_cseq_and_session(cseq, session),
            None => Headers::with_cseq(cseq),
        };
        headers.extend(additional_headers);
        let request = Request::new(
            RequestMetadata::new_v1(method, self.uri.clone()),
            headers,
            None,
        );
        self.write
            .send(RequestMaybeInterleaved::Message(request))
            .await?;
        match self.read.next().await {
            Some(Ok(MaybeInterleaved::Message(response))) => Ok(response),
            Some(Ok(MaybeInterleaved::Interleaved { .. })) => {
                Err(ClientError::UnexpectedInterleavedMessage)
            }
            Some(Err(err)) => Err(err.into()),
            None => Err(ClientError::ConnectionClosed),
        }
    }
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

#[derive(Debug)]
pub enum ClientError {
    /// URI missing authority.
    UriMissingAuthority,
    /// URI unsupported protocol scheme.
    UriUnsupportedProtocolScheme { scheme: String },
    /// URI missing protocol scheme.
    UriMissingProtocolScheme,
    /// Could not connect.
    Connect { errors: Vec<ClientError> },
    /// Could not resolve server.
    Resolve { name: String },
    /// Non-successful status code.
    Status(Response),
    /// Protocol error.
    Protocol(Error),
    /// Missing SDP content.
    MissingSdp,
    /// Invalid SDP content.
    InvalidSdp(rave_sdp::Error),
    /// Connection unexpectedly closed.
    ConnectionClosed,
    /// Received unexpected interleaved data response from server.
    UnexpectedInterleavedMessage,
    /// Server issued redirection with missing or invalid "Location" header.
    InvalidRedirect,
    /// Server issued to many consecutive redirects.
    MaximumNumberOfRedirectsReached,
    /// I/O error occurred.
    Io(std::io::Error),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClientError::UriMissingAuthority => write!(f, "uri missing authority"),
            ClientError::UriUnsupportedProtocolScheme { scheme } => {
                write!(f, "uri has unsupported protocol scheme: {scheme}")
            }
            ClientError::UriMissingProtocolScheme => write!(f, "uri missing protocol scheme"),
            ClientError::Connect { errors } => {
                write!(
                    f,
                    "failed to connect: {}",
                    errors
                        .iter()
                        .map(|err| format!("{}", err))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ClientError::Resolve { name } => write!(f, "failed to resolve server name: {name}"),
            ClientError::Status(response) => write!(
                f,
                "response status code: {}",
                match status_from_code(response.status) {
                    Some(status) => format!("{}", status),
                    None => response.status.to_string(),
                }
            ),
            ClientError::Protocol(error) => write!(f, "{}", error),
            ClientError::MissingSdp => write!(
                f,
                "expected request to carry a session description but it does not"
            ),
            ClientError::InvalidSdp(error) => write!(f, "{}", error),
            ClientError::ConnectionClosed => write!(f, "connection closed"),
            ClientError::UnexpectedInterleavedMessage => {
                write!(
                    f,
                    "received unexpected interleaved data response from server"
                )
            }
            ClientError::InvalidRedirect => write!(
                f,
                "server issued redirect with missing or invalid location header"
            ),
            ClientError::MaximumNumberOfRedirectsReached => {
                write!(f, "server issued too many consecutive redirects")
            }
            ClientError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::convert::From<Error> for ClientError {
    fn from(error: Error) -> Self {
        ClientError::Protocol(error)
    }
}

impl std::convert::From<rave_sdp::Error> for ClientError {
    fn from(error: rave_sdp::Error) -> Self {
        ClientError::InvalidSdp(error)
    }
}

impl std::convert::From<std::io::Error> for ClientError {
    fn from(error: std::io::Error) -> Self {
        ClientError::Io(error)
    }
}

impl std::error::Error for ClientError {}
