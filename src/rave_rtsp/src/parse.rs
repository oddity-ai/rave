use crate::buffer::{Buf, ReadLine};
use crate::error::{Error, Result};
use crate::message::{Bytes, Headers, Message, StatusCode, Uri, Version};
use crate::request::{Request, RequestMetadata};
use crate::response::{Response, ResponseMetadata};

pub type RequestParser = Parser<Request>;
pub type ResponseParser = Parser<Response>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Hungry,
    Done,
}

#[derive(Debug)]
pub struct Parser<M: Message> {
    state: State,
    metadata: Option<M::Metadata>,
    headers: Headers,
    body: Option<Bytes>,
}

impl<M: Message> Parser<M> {
    pub fn new() -> Self {
        Self {
            state: State::Head(Head::FirstLine),
            metadata: None,
            headers: Headers::new(),
            body: None,
        }
    }

    pub fn into_message(self) -> Result<M> {
        match self.state {
            State::Body(Body::Complete) => Ok(M::new(
                self.metadata.ok_or(Error::MetadataNotParsed)?,
                self.headers,
                self.body,
            )),
            _ => Err(Error::NotDone),
        }
    }

    pub fn parse(&mut self, buffer: &mut impl Buf) -> Result<Status> {
        self.parse_loop(buffer)?;

        match &self.state {
            State::Body(Body::Complete) => Ok(Status::Done),
            State::Body(Body::Incomplete) => Ok(Status::Hungry),
            State::Head(_) => Ok(Status::Hungry),
        }
    }

    fn parse_loop(&mut self, buffer: &mut impl Buf) -> Result<()> {
        let mut again = true;
        while again {
            (self.state, again) = self.parse_inner(buffer)?;
        }

        Ok(())
    }

    fn parse_inner(&mut self, buffer: &mut impl Buf) -> Result<(State, Again)> {
        match self.state {
            State::Head(head) => {
                let next_head = self.parse_inner_head(buffer, head)?;
                match next_head {
                    Head::Done => {
                        if self.have_content_length() {
                            Ok((State::Body(Body::Incomplete), true))
                        } else {
                            Ok((State::Body(Body::Complete), false))
                        }
                    }
                    _ => Ok((State::Head(next_head), false)),
                }
            }
            State::Body(Body::Incomplete) => {
                let need = self
                    .find_content_length()?
                    .ok_or_else(|| Error::ContentLengthMissing)?;
                let got = buffer.remaining();

                if got >= need {
                    self.body = Some(buffer.copy_to_bytes(need));
                    Ok((State::Body(Body::Complete), false))
                } else {
                    Ok((State::Body(Body::Incomplete), false))
                }
            }
            State::Body(Body::Complete) => Err(Error::BodyAlreadyDone),
        }
    }

    fn parse_inner_head(&mut self, buffer: &mut impl Buf, mut head: Head) -> Result<Head> {
        while head != Head::Done {
            let line = match buffer.read_line() {
                Some(line) => line.map_err(|_| Error::Encoding)?,
                None => break,
            };

            head = Self::parse_inner_head_line(&mut self.metadata, &mut self.headers, line, head)?;
        }

        Ok(head)
    }

    fn parse_inner_head_line(
        metadata: &mut Option<M::Metadata>,
        headers: &mut Headers,
        line: String,
        head: Head,
    ) -> Result<Head> {
        let line = line.trim();
        match head {
            Head::FirstLine => {
                *metadata = Some(Self::parse_metadata(line)?);
                Ok(Head::Header)
            }
            Head::Header => {
                Ok(if !line.is_empty() {
                    let (var, val) = parse_header(line)?;
                    headers.insert(var, val);
                    Head::Header
                } else {
                    // The line is empty, so we got CRLF, which signals end of headers for this
                    // request.
                    Head::Done
                })
            }
            Head::Done => Err(Error::HeadAlreadyDone),
        }
    }

    fn parse_metadata(line: &str) -> Result<M::Metadata> {
        M::Metadata::parse(line)
    }

    fn have_content_length(&self) -> bool {
        self.headers.contains("Content-Length")
    }

    fn find_content_length(&self) -> Result<Option<usize>> {
        if let Some(content_length) = self.headers.get("Content-Length") {
            Ok(Some(content_length.parse::<usize>().map_err(|_| {
                Error::ContentLengthNotInteger {
                    value: content_length.to_string(),
                }
            })?))
        } else {
            Ok(None)
        }
    }

    fn parse_and_into(mut self, mut buffer: impl Buf) -> Result<M> {
        self.parse(&mut buffer)?;
        self.into_message()
    }
}

impl<M: Message> Default for Parser<M> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Parser<Request> {
    #[inline]
    pub fn parse_and_into_request(self, buffer: impl Buf) -> Result<Request> {
        self.parse_and_into(buffer)
    }

    #[inline]
    pub fn into_request(self) -> Result<Request> {
        self.into_message()
    }
}

impl Parser<Response> {
    #[inline]
    pub fn parse_and_into_response(self, buffer: impl Buf) -> Result<Response> {
        self.parse_and_into(buffer)
    }

    #[inline]
    pub fn into_response(self) -> Result<Response> {
        self.into_message()
    }
}

type Again = bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Head(Head),
    Body(Body),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Head {
    FirstLine,
    Header,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Body {
    Incomplete,
    Complete,
}

pub trait Parse: Sized {
    fn parse(line: &str) -> Result<Self>;
}

impl Parse for RequestMetadata {
    fn parse(line: &str) -> Result<RequestMetadata> {
        let mut parts = line.split(' ');

        let method = parts
            .next()
            .ok_or_else(|| Error::RequestLineMalformed {
                line: line.to_string(),
            })?
            .parse()?;

        let uri = parts
            .next()
            .ok_or_else(|| Error::UriMissing {
                line: line.to_string(),
            })?
            .to_string();

        let uri = uri.parse::<Uri>().map_err(|_| Error::UriMalformed {
            line: line.to_string(),
            uri: uri.to_string(),
        })?;

        let uri = if uri.authority().is_some() || uri.path() == "*" {
            Ok(uri)
        } else {
            // Relative URI's are not allowed in RTSP.
            Err(Error::UriNotAbsolute { uri })
        }?;

        let version = parts.next().ok_or_else(|| Error::VersionMissing {
            line: line.to_string(),
        })?;

        let version = parse_version(version, line)?;

        Ok(RequestMetadata::new(method, uri, version))
    }
}

impl Parse for ResponseMetadata {
    fn parse(line: &str) -> Result<ResponseMetadata> {
        let (version, rest) = line
            .split_once(' ')
            .ok_or_else(|| Error::StatusCodeMissing {
                line: line.to_string(),
            })?;

        let version = parse_version(version.trim(), line)?;

        let (status_code, rest) =
            rest.split_once(' ')
                .ok_or_else(|| Error::ReasonPhraseMissing {
                    line: line.to_string(),
                })?;

        let status_code =
            status_code
                .trim()
                .parse::<StatusCode>()
                .map_err(|_| Error::StatusCodeNotInteger {
                    line: line.to_string(),
                    status_code: status_code.to_string(),
                })?;

        let reason = rest.trim().to_string();

        Ok(ResponseMetadata::new(version, status_code, reason))
    }
}

fn parse_version(part: &str, line: &str) -> Result<Version> {
    if let Some(version) = part.strip_prefix("RTSP/") {
        Ok(match version {
            "1.0" => Version::V1,
            "2.0" => Version::V2,
            _ => Version::Unknown,
        })
    } else {
        Err(Error::VersionMalformed {
            line: line.to_string(),
            version: part.to_string(),
        })
    }
}

fn parse_header(line: &str) -> Result<(String, String)> {
    let (var, val) = line.split_once(':').ok_or_else(|| Error::HeaderMalformed {
        line: line.to_string(),
    })?;

    Ok((var.trim().to_string(), val.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::message::{Method, StatusCategory};

    use super::*;

    #[test]
    fn parse_options_request() {
        let request = br###"OPTIONS rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 1
Require: implicit-play
Proxy-Require: gzipped-messages

"###;

        let request = RequestParser::new()
            .parse_and_into_request(request.as_slice())
            .unwrap();
        assert_eq!(request.method, Method::Options);
        assert_eq!(request.uri, "rtsp://example.com/media.mp4");
        assert_eq!(request.version, Version::V1);
        assert_eq!(request.headers.get("CSeq"), Some("1"));
        assert_eq!(request.headers.get("Require"), Some("implicit-play"));
        assert_eq!(
            request.headers.get("Proxy-Require"),
            Some("gzipped-messages")
        );
    }

    #[test]
    fn parse_options_request_any() {
        let request = br###"OPTIONS * RTSP/1.0
CSeq: 1

"###;

        let request = RequestParser::new()
            .parse_and_into_request(request.as_slice())
            .unwrap();
        assert_eq!(request.method, Method::Options);
        assert_eq!(request.uri, "*");
        assert_eq!(request.version, Version::V1);
        assert_eq!(request.headers.get("CSeq"), Some("1"));
    }

    #[test]
    fn parse_options_response() {
        let response = br###"RTSP/1.0 200 OK
CSeq: 1
Public: DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE

"###;

        let response = ResponseParser::new()
            .parse_and_into_response(response.as_slice())
            .unwrap();
        assert_eq!(response.version, Version::V1);
        assert_eq!(response.status, 200);
        assert_eq!(response.status(), StatusCategory::Success);
        assert_eq!(response.reason, "OK");
        assert_eq!(response.headers.get("CSeq"), Some("1"));
        assert_eq!(
            response.headers.get("Public"),
            Some("DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE")
        );
    }

    #[test]
    fn parse_options_response_error() {
        let response = br###"RTSP/1.0 404 Stream Not Found
CSeq: 1

"###;

        let response = ResponseParser::new()
            .parse_and_into_response(response.as_slice())
            .unwrap();
        assert_eq!(response.version, Version::V1);
        assert_eq!(response.status, 404);
        assert_eq!(response.status(), StatusCategory::ClientError);
        assert_eq!(response.reason, "Stream Not Found");
        assert_eq!(response.headers.get("CSeq"), Some("1"));
    }

    #[test]
    fn parse_describe_request() {
        let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 2

"###;

        let request = RequestParser::new()
            .parse_and_into_request(request.as_slice())
            .unwrap();
        assert_eq!(request.method, Method::Describe);
        assert_eq!(request.uri, "rtsp://example.com/media.mp4");
        assert_eq!(request.version, Version::V1);
        assert_eq!(request.headers.get("CSeq"), Some("2"));
    }

    #[test]
    fn parse_describe_request_v2() {
        let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/2.0
CSeq: 2

"###;

        let request = RequestParser::new()
            .parse_and_into_request(request.as_slice())
            .unwrap();
        assert_eq!(request.method, Method::Describe);
        assert_eq!(request.uri, "rtsp://example.com/media.mp4");
        assert_eq!(request.version, Version::V2);
        assert_eq!(request.headers.get("CSeq"), Some("2"));
    }

    #[test]
    fn parse_describe_request_v3() {
        let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/3.0
CSeq: 2

"###;

        let request = RequestParser::new()
            .parse_and_into_request(request.as_slice())
            .unwrap();
        assert_eq!(request.method, Method::Describe);
        assert_eq!(request.uri, "rtsp://example.com/media.mp4");
        assert_eq!(request.version, Version::Unknown);
        assert_eq!(request.headers.get("CSeq"), Some("2"));
    }

    #[test]
    fn parse_describe_response() {
        let response = br###"RTSP/1.0 200 OK
CSeq: 2
Content-Base: rtsp://example.com/media.mp4
Content-Type: application/sdp
Content-Length: 443

m=video 0 RTP/AVP 96
a=control:streamid=0
a=range:npt=0-7.741000
a=length:npt=7.741000
a=rtpmap:96 MP4V-ES/5544
a=mimetype:string;"video/MP4V-ES"
a=AvgBitRate:integer;304018
a=StreamName:string;"hinted video track"
m=audio 0 RTP/AVP 97
a=control:streamid=1
a=range:npt=0-7.712000
a=length:npt=7.712000
a=rtpmap:97 mpeg4-generic/32000/2
a=mimetype:string;"audio/mpeg4-generic"
a=AvgBitRate:integer;65790
a=StreamName:string;"hinted audio track""###;

        let response = ResponseParser::new()
            .parse_and_into_response(response.as_slice())
            .unwrap();
        assert_eq!(response.version, Version::V1);
        assert_eq!(response.status, 200);
        assert_eq!(response.reason, "OK");
        assert_eq!(response.headers.get("CSeq"), Some("2"));
        assert_eq!(
            response.headers.get("Content-Base"),
            Some("rtsp://example.com/media.mp4")
        );
        assert_eq!(
            response.headers.get("Content-Type"),
            Some("application/sdp")
        );
        assert_eq!(response.headers.get("Content-Length"), Some("443"));
    }

    const EXAMPLE_PIPELINED_REQUESTS: &[u8] = br###"RECORD rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 6
Session: 12345678

ANNOUNCE rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 7
Date: 23 Jan 1997 15:35:06 GMT
Session: 12345678
Content-Type: application/sdp
Content-Length: 305

v=0
o=mhandley 2890844526 2890845468 IN IP4 126.16.64.4
s=SDP Seminar
i=A Seminar on the session description protocol
u=http://www.cs.ucl.ac.uk/staff/M.Handley/sdp.03.ps
e=mjh@isi.edu (Mark Handley)
c=IN IP4 224.2.17.12/127
t=2873397496 2873404696
a=recvonly
m=audio 3456 RTP/AVP 0
m=video 2232 RTP/AVP 31TEARDOWN rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 8
Session: 12345678

"###;

    #[test]
    fn parse_pipelined_requests() {
        let mut buffer = Bytes::from_static(EXAMPLE_PIPELINED_REQUESTS);
        let mut parser = RequestParser::new();

        let mut requests = Vec::new();
        for _ in 0..3 {
            if parser.parse(&mut buffer).unwrap() == Status::Done {
                requests.push(parser.into_request().unwrap());
                parser = RequestParser::new();
            }
        }

        test_example_piplined_requests(&requests);
    }

    #[test]
    fn parse_pipelined_requests_pieces1() {
        let mut buffer = BytesMut::new();
        let mut parser = RequestParser::new();

        let mut requests = Vec::new();
        for i in 0..EXAMPLE_PIPELINED_REQUESTS.len() {
            buffer.extend_from_slice(&EXAMPLE_PIPELINED_REQUESTS[i..i + 1]);
            if parser.parse(&mut buffer).unwrap() == Status::Done {
                requests.push(parser.into_request().unwrap());
                parser = RequestParser::new();
            }
        }

        test_example_piplined_requests(&requests);
    }

    #[test]
    fn parse_pipelined_requests_pieces_varying() {
        let mut buffer = BytesMut::new();
        let mut parser = RequestParser::new();

        let mut requests = Vec::new();
        let mut start = 0;
        let mut size = 1;
        loop {
            let piece_range = start..(start + size).min(EXAMPLE_PIPELINED_REQUESTS.len());
            buffer.extend_from_slice(&EXAMPLE_PIPELINED_REQUESTS[piece_range]);
            if let Status::Done = parser.parse(&mut buffer).unwrap() {
                requests.push(parser.into_request().unwrap());
                parser = RequestParser::new();
            }
            start += size;
            size = (size * 2) % 9;
            if start >= EXAMPLE_PIPELINED_REQUESTS.len() {
                break;
            }
        }

        test_example_piplined_requests(&requests);
    }

    fn test_example_piplined_requests(requests: &[Request]) {
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].method, Method::Record);
        assert_eq!(requests[0].uri, "rtsp://example.com/media.mp4");
        assert_eq!(requests[0].version, Version::V1);
        assert_eq!(requests[0].headers.get("CSeq"), Some("6"));
        assert_eq!(requests[0].headers.get("Session"), Some("12345678"));
        assert_eq!(requests[0].body, None);
        assert_eq!(requests[1].method, Method::Announce);
        assert_eq!(requests[1].uri, "rtsp://example.com/media.mp4");
        assert_eq!(requests[1].version, Version::V1);
        assert_eq!(requests[1].headers.get("CSeq"), Some("7"));
        assert_eq!(requests[1].headers.get("Session"), Some("12345678"));
        assert_eq!(
            requests[1].headers.get("Date"),
            Some("23 Jan 1997 15:35:06 GMT")
        );
        assert_eq!(
            requests[1].headers.get("Content-Type"),
            Some("application/sdp")
        );
        assert_eq!(requests[1].headers.get("Content-Length"), Some("305"));
        assert_eq!(requests[1].body.as_ref().unwrap().len(), 305);
        assert_eq!(requests[2].method, Method::Teardown);
        assert_eq!(requests[2].uri, "rtsp://example.com/media.mp4");
        assert_eq!(requests[2].version, Version::V1);
        assert_eq!(requests[2].headers.get("CSeq"), Some("8"));
        assert_eq!(requests[2].headers.get("Session"), Some("12345678"));
        assert_eq!(requests[2].body, None);
    }

    const EXAMPLE_REQUEST_PLAY_CRLN: &[u8] = b"PLAY rtsp://example.com/stream/0 RTSP/1.0\r\n\
CSeq: 1\r\n\
Session: 1234abcd\r\n\
Content-Length: 16\r\n\
\r\n\
0123456789abcdef";

    #[test]
    fn parse_play_request() {
        let request = RequestParser::new()
            .parse_and_into_request(EXAMPLE_REQUEST_PLAY_CRLN)
            .unwrap();
        test_example_request_play(&request);
    }

    #[test]
    fn parse_play_request_partial_piece1_ln() {
        parse_play_request_partial_piece1(&request_play_ln());
    }

    #[test]
    fn parse_play_request_partial_piece2_ln() {
        parse_play_request_partial_piece(&request_play_ln(), 2);
    }

    #[test]
    fn parse_play_request_partial_piece3_ln() {
        parse_play_request_partial_piece(&request_play_ln(), 3);
    }

    #[test]
    fn parse_play_request_partial_piece_varying_ln() {
        parse_play_request_partial_piece_varying(&request_play_ln());
    }

    #[test]
    fn parse_play_request_partial_piece1_cr() {
        parse_play_request_partial_piece1(&request_play_cr());
    }

    #[test]
    fn parse_play_request_partial_piece2_cr() {
        parse_play_request_partial_piece(&request_play_cr(), 2);
    }

    #[test]
    fn parse_play_request_partial_piece3_cr() {
        parse_play_request_partial_piece(&request_play_cr(), 3);
    }

    #[test]
    fn parse_play_request_partial_piece_varying_cr() {
        parse_play_request_partial_piece_varying(&request_play_cr());
    }

    #[test]
    fn parse_play_request_partial_piece1_crln() {
        parse_play_request_partial_piece1(&request_play_crln());
    }

    #[test]
    fn parse_play_request_partial_piece2_crln() {
        parse_play_request_partial_piece(&request_play_crln(), 2);
    }

    #[test]
    fn parse_play_request_partial_piece3_crln() {
        parse_play_request_partial_piece(&request_play_crln(), 3);
    }

    #[test]
    fn parse_play_request_partial_piece_varying_crln() {
        parse_play_request_partial_piece_varying(&request_play_crln());
    }

    fn request_play_ln() -> Bytes {
        EXAMPLE_REQUEST_PLAY_CRLN
            .iter()
            .copied()
            .filter(|b| *b != b'\x0d')
            .collect::<Bytes>()
    }

    fn request_play_cr() -> Bytes {
        EXAMPLE_REQUEST_PLAY_CRLN
            .iter()
            .copied()
            .filter(|b| *b != b'\x0a')
            .collect::<Bytes>()
    }

    fn request_play_crln() -> Bytes {
        Bytes::from_static(EXAMPLE_REQUEST_PLAY_CRLN)
    }

    fn parse_play_request_partial_piece1(request_bytes: &[u8]) {
        let mut buffer = BytesMut::new();
        let mut parser = RequestParser::new();

        let upto_last = request_bytes.len() - 1;
        for i in 0..upto_last {
            let i_range = i..i + 1;
            buffer.extend_from_slice(&request_bytes[i_range]);
            assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Hungry);
        }

        let last_range = request_bytes.len() - 1..;
        buffer.extend_from_slice(&request_bytes[last_range]);
        assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Done);

        let request = parser.into_request().unwrap();
        test_example_request_play(&request);
    }

    fn parse_play_request_partial_piece(request_bytes: &[u8], piece_size: usize) {
        let mut buffer = BytesMut::new();
        let mut parser = RequestParser::new();

        let pieces_upto_last = (request_bytes.len() / piece_size) - 1;
        for i in 0..pieces_upto_last {
            let piece_range = (i * piece_size)..(i * piece_size) + piece_size;
            buffer.extend_from_slice(&request_bytes[piece_range]);
            assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Hungry);
        }

        let last_piece = pieces_upto_last;
        let leftover_piece_range = last_piece * piece_size..;
        buffer.extend_from_slice(&request_bytes[leftover_piece_range]);
        assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Done);

        let request = parser.into_request().unwrap();
        test_example_request_play(&request);
    }

    fn parse_play_request_partial_piece_varying(request_bytes: &[u8]) {
        let mut buffer = BytesMut::new();
        let mut parser = RequestParser::new();

        let mut start = 0;
        let mut size = 1;
        loop {
            let piece_range = start..(start + size).min(request_bytes.len());
            buffer.extend_from_slice(&request_bytes[piece_range]);
            if let Status::Done = parser.parse(&mut buffer).unwrap() {
                break;
            }
            start += size;
            size = (size * 2) % 9;
        }

        let request = parser.into_request().unwrap();
        test_example_request_play(&request);
    }

    fn test_example_request_play(request: &Request) {
        assert_eq!(request.method, Method::Play);
        assert_eq!(request.uri, "rtsp://example.com/stream/0");
        assert_eq!(request.version, Version::V1);
        assert_eq!(request.headers.get("CSeq"), Some("1"));
        assert_eq!(request.headers.get("Session"), Some("1234abcd"));
        assert_eq!(request.headers.get("Content-Length"), Some("16"));
        assert_eq!(request.body, Some(b"0123456789abcdef".as_slice().into()));
    }
}
