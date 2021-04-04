use thiserror::Error;

#[derive(Error, Debug)]
pub enum TdmsReadError {
    #[error("{0}")]
    TdmsError(String),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("UTF8 decode error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, TdmsReadError>;

