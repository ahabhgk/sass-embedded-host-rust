use crate::api::Exception;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  Compile(String),
  Host(String),
  Value(String),
  SassException(Exception),
  IO(std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Compile(m) => write!(f, "Compiler caused error: {}", m),
      Error::Host(m) => write!(f, "Compiler reported error: {}", m),
      Error::Value(m) => write!(f, "{}", m),
      Error::SassException(e) => write!(f, "{}", e),
      Error::IO(e) => write!(f, "{}", e),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Error::IO(e)
  }
}

impl From<Exception> for Error {
  fn from(e: Exception) -> Self {
    Error::SassException(e)
  }
}
