use crate::message::{
    status_to_code, status_to_reason, Bytes, Headers, Message, Status, StatusCategory, StatusCode,
    Version,
};
use crate::request::Request;
use crate::rtp_info::RtpInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub version: Version,
    pub status: StatusCode,
    pub reason: String,
    pub headers: Headers,
    pub body: Option<Bytes>,
}

impl Message for Response {
    type Metadata = ResponseMetadata;

    fn new(metadata: ResponseMetadata, headers: Headers, body: Option<Bytes>) -> Self {
        Self {
            version: metadata.version,
            status: metadata.status,
            reason: metadata.reason,
            headers,
            body,
        }
    }
}

impl Response {
    pub fn ok() -> ResponseBuilder {
        ResponseBuilder::ok()
    }

    pub fn error(status: Status) -> ResponseBuilder {
        ResponseBuilder::error(status)
    }

    pub fn status(&self) -> StatusCategory {
        match self.status {
            s if s >= 600 => StatusCategory::Unknown,
            s if s >= 500 => StatusCategory::ServerError,
            s if s >= 400 => StatusCategory::ClientError,
            s if s >= 300 => StatusCategory::Redirection,
            s if s >= 200 => StatusCategory::Success,
            s if s >= 100 => StatusCategory::Informational,
            _ => StatusCategory::Unknown,
        }
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Version: {}, Status Code: {}, Reason Phrase: {}",
            self.version, self.status, &self.reason
        )?;

        if !self.headers.is_empty() {
            writeln!(f, "\nHeaders:")?;
            for (var, val) in &self.headers {
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
pub struct ResponseMetadata {
    version: Version,
    status: StatusCode,
    reason: String,
}

impl ResponseMetadata {
    pub(super) fn new(version: Version, status: StatusCode, reason: String) -> Self {
        Self {
            version,
            status,
            reason,
        }
    }
}

pub struct ResponseBuilder {
    response: Response,
}

impl ResponseBuilder {
    pub fn from_status(status: Status) -> ResponseBuilder {
        ResponseBuilder {
            response: Response {
                version: Default::default(),
                status: status_to_code(status),
                reason: status_to_reason(status).to_string(),
                headers: Default::default(),
                body: Default::default(),
            },
        }
    }

    pub fn ok() -> ResponseBuilder {
        Self::from_status(Status::Ok)
    }

    pub fn error(status: Status) -> ResponseBuilder {
        Self::from_status(status)
    }

    pub fn with_cseq_of(mut self, request: &Request) -> ResponseBuilder {
        if let Some(cseq) = request.headers.get("CSeq") {
            self.response
                .headers
                .insert("CSeq".to_string(), cseq.to_string());
        }
        self
    }

    pub fn with_header(mut self, var: impl ToString, val: impl ToString) -> ResponseBuilder {
        self.response
            .headers
            .insert(var.to_string(), val.to_string());
        self
    }

    pub fn with_rtp_info(mut self, rtp_info: impl IntoIterator<Item = RtpInfo>) -> ResponseBuilder {
        self.response.headers.insert(
            "RTP-Info".to_string(),
            rtp_info
                .into_iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        self
    }

    pub fn with_body(mut self, body: Bytes, content_type: &str) -> ResponseBuilder {
        self = self
            .with_header("Content-Length", body.len())
            .with_header("Content-Type", content_type);
        self.response.body = Some(body);
        self
    }

    pub fn with_sdp(self, contents: String) -> ResponseBuilder {
        self.with_body(Bytes::from(contents), "application/sdp")
    }

    pub fn build(self) -> Response {
        self.response
    }
}
