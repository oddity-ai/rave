use crate::message::{Bytes, Headers, Message, Method, Uri, Version};
use crate::range::Range;
use crate::transport::Transport;
use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: Headers,
    pub body: Option<Bytes>,
}

impl Message for Request {
    type Metadata = RequestMetadata;

    fn new(metadata: RequestMetadata, headers: Headers, body: Option<Bytes>) -> Self {
        Self {
            method: metadata.method,
            uri: metadata.uri,
            version: metadata.version,
            headers,
            body,
        }
    }
}

impl Request {
    pub fn options(uri: &Uri, cseq: usize) -> Self {
        Request::new(
            RequestMetadata::new_v1(Method::Options, uri.clone()),
            Headers::with_cseq(cseq),
            None,
        )
    }

    pub fn describe(uri: &Uri, cseq: usize) -> Self {
        Request::new(
            RequestMetadata::new_v1(Method::Describe, uri.clone()),
            Headers::with_cseq(cseq),
            None,
        )
    }

    pub fn setup(uri: &Uri, cseq: usize, transport: Transport, session: Option<&str>) -> Self {
        let mut headers = match session {
            Some(session_id) => Headers::with_cseq_and_session(cseq, session_id),
            None => Headers::with_cseq(cseq),
        };
        headers.insert("Transport".to_string(), transport.to_string());
        Request::new(
            RequestMetadata::new_v1(Method::Setup, uri.clone()),
            headers,
            None,
        )
    }

    pub fn play(uri: &Uri, cseq: usize, session: &str, range: Range) -> Self {
        let mut headers = Headers::with_cseq_and_session(cseq, session);
        headers.insert("Range".to_string(), range.to_string());
        Request::new(
            RequestMetadata::new_v1(Method::Play, uri.clone()),
            headers,
            None,
        )
    }

    pub fn pause(uri: &Uri, cseq: usize, session: &str) -> Self {
        Request::new(
            RequestMetadata::new_v1(Method::Pause, uri.clone()),
            Headers::with_cseq_and_session(cseq, session),
            None,
        )
    }

    pub fn teardown(uri: &Uri, cseq: usize, session: &str) -> Self {
        Request::new(
            RequestMetadata::new_v1(Method::Teardown, uri.clone()),
            Headers::with_cseq_and_session(cseq, session),
            None,
        )
    }

    // FIXME: implement request constructors for remaining RTSP methods.

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn path(&self) -> &str {
        self.uri.path().trim_end_matches('/')
    }

    pub fn require(&self) -> Option<&str> {
        self.headers.get("Require")
    }

    pub fn accept(&self) -> Vec<&str> {
        self.headers
            .get("Accept")
            .map(|val| val.split(',').map(|part| part.trim()).collect::<Vec<_>>())
            .unwrap_or_default()
    }

    pub fn session(&self) -> Option<&str> {
        self.headers.get("Session")
    }

    pub fn transport(&self) -> Result<Vec<Transport>, Error> {
        if let Some(value) = self.headers.get("Transport") {
            value
                .split(',')
                .map(|part| part.parse())
                .collect::<Result<Vec<_>, _>>()
        } else {
            Ok(Vec::new())
        }
    }

    pub fn range(&self) -> Option<Result<Range, Error>> {
        self.headers.get("Range").map(|value| value.parse())
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Version: {}, Method: {}, Uri: {}",
            self.version, self.method, self.uri
        )?;

        if !self.headers.is_empty() {
            writeln!(f, "\nHeaders:")?;
            for (var, val) in self.headers.as_map() {
                writeln!(f, " - {}: {}", &var, &val)?;
            }
        }

        if let Some(body) = &self.body {
            writeln!(f, "[{} bytes]", body.len())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestMetadata {
    method: Method,
    uri: Uri,
    version: Version,
}

impl RequestMetadata {
    pub(super) fn new(method: Method, uri: Uri, version: Version) -> Self {
        Self {
            method,
            uri,
            version,
        }
    }

    pub(super) fn new_v1(method: Method, uri: Uri) -> Self {
        Self {
            method,
            uri,
            version: Version::V1,
        }
    }
}
