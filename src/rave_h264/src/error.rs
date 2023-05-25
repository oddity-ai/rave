#[derive(Debug)]
pub enum Error {
    H264AnnexBStartCodeMissing,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::H264AnnexBStartCodeMissing => {
                write!(f, "expected annex b start code but it is not there")
            }
        }
    }
}

impl std::error::Error for Error {}
