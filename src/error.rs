/// All possible errors that may be returned when reading a TDMS file
#[derive(Debug)]
pub enum TdmsReadError {
    /// Invalid data format
    TdmsError(String),
    /// An IO error reading the underlying file
    IoError(std::io::Error),
    /// An error decoding UTF-8 strings
    Utf8Error(std::string::FromUtf8Error),
}

impl std::error::Error for TdmsReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            TdmsReadError::TdmsError(_) => None,
            TdmsReadError::IoError(ref e) => Some(e),
            TdmsReadError::Utf8Error(ref e) => Some(e),
        }
    }
}

impl std::fmt::Display for TdmsReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TdmsReadError::TdmsError(ref s) => write!(f, "{}", s),
            TdmsReadError::IoError(_) => write!(f, "IO error"),
            TdmsReadError::Utf8Error(_) => write!(f, "UTF-8 decode error"),
        }
    }
}

impl From<std::io::Error> for TdmsReadError {
    fn from(err: std::io::Error) -> TdmsReadError {
        TdmsReadError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for TdmsReadError {
    fn from(err: std::string::FromUtf8Error) -> TdmsReadError {
        TdmsReadError::Utf8Error(err)
    }
}

pub type Result<T> = std::result::Result<T, TdmsReadError>;
