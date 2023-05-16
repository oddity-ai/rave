pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    VersionUnknown { version: usize },
    CsrcCountInvalid { count: usize },
    ExtensionLengthInvalid { length: usize },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::VersionUnknown { version } => write!(f, "version number unknown: {version}"),
            Error::CsrcCountInvalid { count } => {
                write!(f, "csrc count invalid (overflow): {count}")
            }
            Error::ExtensionLengthInvalid { length } => {
                write!(f, "extension length invalid (overflow): {length}")
            }
        }
    }
}

impl std::error::Error for Error {}
