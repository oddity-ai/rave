use bytes::{BufMut, BytesMut};

use crate::error::{Error, Result};
use crate::message::{Method, StatusCode, Uri, Version};
use crate::request::Request;
use crate::response::Response;

pub trait Serialize {
    fn serialize(self, dst: &mut BytesMut) -> Result<()>;
}

impl Serialize for Request {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        self.method.serialize(dst)?;
        dst.put_u8(b' ');
        self.uri.serialize(dst)?;
        dst.put_u8(b' ');
        self.version.serialize(dst)?;
        dst.put_u8(b'\r');
        dst.put_u8(b'\n');

        for (var, val) in self.headers.into_map() {
            dst.put(format!("{var}: {val}\r\n").as_bytes());
        }

        dst.put(b"\r\n".as_slice());

        if let Some(body) = self.body {
            dst.put(body);
        }

        Ok(())
    }
}

impl Serialize for Response {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        self.version.serialize(dst)?;
        dst.put_u8(b' ');
        self.status.serialize(dst)?;
        dst.put_u8(b' ');
        dst.put(self.reason.as_bytes());
        dst.put_u8(b'\r');
        dst.put_u8(b'\n');

        for (var, val) in self.headers.into_map() {
            dst.put(format!("{var}: {val}\r\n").as_bytes());
        }

        dst.put(b"\r\n".as_slice());

        if let Some(body) = self.body {
            dst.put(body);
        }

        Ok(())
    }
}

impl Serialize for Version {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        let version = match self {
            Version::V1 => b"RTSP/1.0".as_slice(),
            Version::V2 => b"RTSP/2.0".as_slice(),
            Version::Unknown => return Err(Error::VersionUnknown),
        };

        dst.put(version);
        Ok(())
    }
}

impl Serialize for Method {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        let method = match self {
            Method::Describe => b"DESCRIBE".as_slice(),
            Method::Announce => b"ANNOUNCE".as_slice(),
            Method::Setup => b"SETUP".as_slice(),
            Method::Play => b"PLAY".as_slice(),
            Method::Pause => b"PAUSE".as_slice(),
            Method::Record => b"RECORD".as_slice(),
            Method::Options => b"OPTIONS".as_slice(),
            Method::Redirect => b"REDIRECT".as_slice(),
            Method::Teardown => b"TEARDOWN".as_slice(),
            Method::GetParameter => b"GET_PARAMETER".as_slice(),
            Method::SetParameter => b"SET_PARAMETER".as_slice(),
        };

        dst.put(method);
        Ok(())
    }
}

impl Serialize for Uri {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        dst.put(self.to_string().as_bytes());
        Ok(())
    }
}

impl Serialize for StatusCode {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        dst.put(self.to_string().as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // FIXME: A bunch of macros could make this way more readable.

    use bytes::{Bytes, BytesMut};

    use crate::message::{Headers, Message};
    use crate::request::RequestMetadata;
    use crate::response::ResponseMetadata;

    use super::*;

    #[test]
    fn serialize_options_request() {
        let request_bytes = Bytes::from(
            b"OPTIONS rtsp://example.com/media.mp4 RTSP/1.0\r\n\
CSeq: 1\r\n\
Proxy-Require: gzipped-messages\r\n\
Require: implicit-play\r\n\
\r\n\
"
            .as_slice(),
        );

        let request = Request::new(
            RequestMetadata::new(
                Method::Options,
                "rtsp://example.com/media.mp4".try_into().unwrap(),
                Version::V1,
            ),
            Headers::from_iter([
                ("CSeq".to_string(), "1".to_string()),
                ("Proxy-Require".to_string(), "gzipped-messages".to_string()),
                ("Require".to_string(), "implicit-play".to_string()),
            ]),
            None,
        );

        let mut request_serialized = BytesMut::new();
        request.serialize(&mut request_serialized).unwrap();
        assert_eq!(request_serialized, request_bytes);
    }

    #[test]
    fn serialize_options_request_headers_alphabetical() {
        let request_bytes = Bytes::from(
            b"OPTIONS rtsp://example.com/media.mp4 RTSP/1.0\r\n\
Aaa: value\r\n\
Bbb: value\r\n\
C: value\r\n\
Ca: value\r\n\
Cb: value\r\n\
Cc: value\r\n\
\r\n\
"
            .as_slice(),
        );

        let request = Request::new(
            RequestMetadata::new(
                Method::Options,
                "rtsp://example.com/media.mp4".try_into().unwrap(),
                Version::V1,
            ),
            Headers::from_iter([
                ("Cc".to_string(), "value".to_string()),
                ("C".to_string(), "value".to_string()),
                ("Cb".to_string(), "value".to_string()),
                ("Bbb".to_string(), "value".to_string()),
                ("Aaa".to_string(), "value".to_string()),
                ("Ca".to_string(), "value".to_string()),
            ]),
            None,
        );

        let mut request_serialized = BytesMut::new();
        request.serialize(&mut request_serialized).unwrap();
        assert_eq!(request_serialized, request_bytes);
    }

    #[test]
    fn serialize_options_request_any() {
        let request_bytes = Bytes::from(
            b"OPTIONS * RTSP/1.0\r\n\
CSeq: 1\r\n\
\r\n\
"
            .as_slice(),
        );

        let request = Request::new(
            RequestMetadata::new(Method::Options, "*".try_into().unwrap(), Version::V1),
            Headers::from_iter([("CSeq".to_string(), "1".to_string())]),
            None,
        );

        let mut request_serialized = BytesMut::new();
        request.serialize(&mut request_serialized).unwrap();
        assert_eq!(request_serialized, request_bytes);
    }

    #[test]
    fn serialize_options_response() {
        let response_bytes = Bytes::from(
            b"RTSP/1.0 200 OK\r\n\
CSeq: 1\r\n\
Public: DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE\r\n\
\r\n\
"
            .as_slice(),
        );

        let response = Response::new(
            ResponseMetadata::new(Version::V1, 200, "OK".to_string()),
            Headers::from_iter([
                ("CSeq".to_string(), "1".to_string()),
                (
                    "Public".to_string(),
                    "DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE".to_string(),
                ),
            ]),
            None,
        );

        let mut response_serialized = BytesMut::new();
        response.serialize(&mut response_serialized).unwrap();
        assert_eq!(response_serialized, response_bytes);
    }

    #[test]
    fn serialize_options_response_error() {
        let response_bytes = Bytes::from(
            b"RTSP/1.0 404 Stream Not Found\r\n\
CSeq: 1\r\n\
\r\n\
"
            .as_slice(),
        );

        let response = Response::new(
            ResponseMetadata::new(Version::V1, 404, "Stream Not Found".to_string()),
            Headers::from_iter([("CSeq".to_string(), "1".to_string())]),
            None,
        );

        let mut response_serialized = BytesMut::new();
        response.serialize(&mut response_serialized).unwrap();
        assert_eq!(response_serialized, response_bytes);
    }

    #[test]
    fn serialize_describe_request() {
        let request_bytes = Bytes::from(
            b"DESCRIBE rtsp://example.com/media.mp4 RTSP/1.0\r\n\
CSeq: 2\r\n\
\r\n\
"
            .as_slice(),
        );

        let request = Request::new(
            RequestMetadata::new(
                Method::Describe,
                "rtsp://example.com/media.mp4".try_into().unwrap(),
                Version::V1,
            ),
            Headers::from_iter([("CSeq".to_string(), "2".to_string())]),
            None,
        );

        let mut request_serialized = BytesMut::new();
        request.serialize(&mut request_serialized).unwrap();
        assert_eq!(request_serialized, request_bytes);
    }

    #[test]
    fn serialize_describe_request_v2() {
        let request_bytes = Bytes::from(
            b"DESCRIBE rtsp://example.com/media.mp4 RTSP/2.0\r\n\
CSeq: 2\r\n\
\r\n\
"
            .as_slice(),
        );

        let request = Request::new(
            RequestMetadata::new(
                Method::Describe,
                "rtsp://example.com/media.mp4".try_into().unwrap(),
                Version::V2,
            ),
            Headers::from_iter([("CSeq".to_string(), "2".to_string())]),
            None,
        );

        let mut request_serialized = BytesMut::new();
        request.serialize(&mut request_serialized).unwrap();
        assert_eq!(request_serialized, request_bytes);
    }

    #[test]
    fn serialize_describe_request_version_unknown_errors() {
        let request = Request::new(
            RequestMetadata::new(
                Method::Describe,
                "rtsp://example.com/media.mp4".try_into().unwrap(),
                Version::Unknown,
            ),
            Headers::from_iter([("CSeq".to_string(), "2".to_string())]),
            None,
        );

        let mut request_serialized = BytesMut::new();
        assert!(matches!(
            request.serialize(&mut request_serialized),
            Err(Error::VersionUnknown)
        ))
    }

    #[test]
    fn serialize_describe_response() {
        let response_bytes = Bytes::from(
            b"RTSP/1.0 200 OK\r\n\
CSeq: 2\r\n\
Content-Base: rtsp://example.com/media.mp4\r\n\
Content-Length: 443\r\n\
Content-Type: application/sdp\r\n\
\r\n\
m=video 0 RTP/AVP 96
a=control:streamid=0
a=range:npt=0-7.741000
a=length:npt=7.741000
a=rtpmap:96 MP4V-ES/5544
a=mimetype:string;\"video/MP4V-ES\"
a=AvgBitRate:integer;304018
a=StreamName:string;\"hinted video track\"
m=audio 0 RTP/AVP 97
a=control:streamid=1
a=range:npt=0-7.712000
a=length:npt=7.712000
a=rtpmap:97 mpeg4-generic/32000/2
a=mimetype:string;\"audio/mpeg4-generic\"
a=AvgBitRate:integer;65790
a=StreamName:string;\"hinted audio track\""
                .as_slice(),
        );

        let response = Response::new(
            ResponseMetadata::new(Version::V1, 200, "OK".to_string()),
            Headers::from_iter([
                ("CSeq".to_string(), "2".to_string()),
                (
                    "Content-Base".to_string(),
                    "rtsp://example.com/media.mp4".to_string(),
                ),
                ("Content-Length".to_string(), "443".to_string()),
                ("Content-Type".to_string(), "application/sdp".to_string()),
            ]),
            Some(Bytes::from(
                b"m=video 0 RTP/AVP 96
a=control:streamid=0
a=range:npt=0-7.741000
a=length:npt=7.741000
a=rtpmap:96 MP4V-ES/5544
a=mimetype:string;\"video/MP4V-ES\"
a=AvgBitRate:integer;304018
a=StreamName:string;\"hinted video track\"
m=audio 0 RTP/AVP 97
a=control:streamid=1
a=range:npt=0-7.712000
a=length:npt=7.712000
a=rtpmap:97 mpeg4-generic/32000/2
a=mimetype:string;\"audio/mpeg4-generic\"
a=AvgBitRate:integer;65790
a=StreamName:string;\"hinted audio track\""
                    .as_slice(),
            )),
        );

        let mut response_serialized = BytesMut::new();
        response.serialize(&mut response_serialized).unwrap();
        assert_eq!(response_serialized, response_bytes);
    }

    #[test]
    fn serialize_play_request() {
        let request_bytes = Bytes::from(
            b"PLAY rtsp://example.com/stream/0 RTSP/1.0\r\n\
CSeq: 1\r\n\
Content-Length: 16\r\n\
Session: 1234abcd\r\n\
\r\n\
0123456789abcdef"
                .as_slice(),
        );

        let request = Request::new(
            RequestMetadata::new(
                Method::Play,
                "rtsp://example.com/stream/0".try_into().unwrap(),
                Version::V1,
            ),
            Headers::from_iter([
                ("CSeq".to_string(), "1".to_string()),
                ("Content-Length".to_string(), "16".to_string()),
                ("Session".to_string(), "1234abcd".to_string()),
            ]),
            Some(Bytes::from(b"0123456789abcdef".as_slice())),
        );

        let mut request_serialized = BytesMut::new();
        request.serialize(&mut request_serialized).unwrap();
        assert_eq!(request_serialized, request_bytes);
    }
}
