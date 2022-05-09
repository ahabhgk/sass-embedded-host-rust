use std::{error::Error as StdError, fmt::Display};

use crate::api::Exception;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  Compile(String),
  Mandatory(String),
  Host(String),
  Value(String),
  SassException(Exception),
}

impl StdError for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Compile(m) => write!(f, "Compiler caused error: {}", m),
      Error::Mandatory(m) => write!(f, "Missing mandatory field: {}", m),
      Error::Host(m) => write!(f, "Compiler reported error: {}", m),
      Error::Value(m) => write!(f, "{}", m),
      Error::SassException(e) => write!(f, "{}", e),
    }
  }
}
