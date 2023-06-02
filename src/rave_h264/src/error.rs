#[derive(Debug)]
pub enum Error {
    OpenH264(openh264::Error),
    AnnexBStartCodeMissing,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::OpenH264(err) => {
                write!(f, "{err}")
            }
            Error::AnnexBStartCodeMissing => {
                write!(f, "expected annex b start code but it is not there")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenH264(err) => Some(err),
            _ => None,
        }
    }
}

impl From<openh264::Error> for Error {
    fn from(err: openh264::Error) -> Self {
        Error::OpenH264(err)
    }
}
