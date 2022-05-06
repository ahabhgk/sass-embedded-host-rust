use std::{error::Error as StdError, fmt::Display};

#[derive(Debug)]
pub enum Error {
  NotFound,
}

impl StdError for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::NotFound => write!(f, "dart-sass-embedded not found"),
    }
  }
}
