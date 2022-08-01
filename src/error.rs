use std::fmt::Display;

use crate::protocol::{
  outbound_message::compile_response::CompileFailure, ProtocolError, SourceSpan,
};

pub type Result<T> = std::result::Result<T, Exception>;

#[derive(Debug)]
pub struct Exception {
  message: String,
  sass_message: Option<String>,
  sass_stack: Option<String>,
  span: Option<SourceSpan>,
}

impl Exception {
  pub fn message(&self) -> &str {
    &self.message
  }

  pub fn sass_message(&self) -> Option<&str> {
    self.sass_message.as_deref()
  }

  pub fn sass_stack(&self) -> Option<&str> {
    self.sass_stack.as_deref()
  }

  pub fn span(&self) -> Option<&SourceSpan> {
    self.span.as_ref()
  }
}

impl std::error::Error for Exception {}

impl Display for Exception {
  /// https://sass-lang.com/documentation/js-api/classes/Exception#toString
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl From<CompileFailure> for Exception {
  fn from(failure: CompileFailure) -> Self {
    Self {
      message: failure.formatted,
      sass_message: Some(failure.message),
      sass_stack: Some(failure.stack_trace),
      span: failure.span,
    }
  }
}

impl From<ProtocolError> for Exception {
  fn from(e: ProtocolError) -> Self {
    Self::new(e.message)
  }
}

impl Exception {
  pub fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      sass_message: None,
      sass_stack: None,
      span: None,
    }
  }
}
