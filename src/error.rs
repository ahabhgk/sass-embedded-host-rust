use std::fmt;

use crate::{
  protocol::{
    outbound_message::compile_response::CompileFailure, ProtocolError,
  },
  SourceSpan,
};

/// An alias for [std::result::Result<T, Exception>].
pub type Result<T> = std::result::Result<T, Box<Exception>>;

/// An exception for this crate, thrown because a Sass compilation failed or `io::Error`.
///
/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/classes/Exception)
#[derive(Debug)]
pub struct Exception {
  message: String,
  sass_message: Option<String>,
  sass_stack: Option<String>,
  span: Option<SourceSpan>,
  source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Exception {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/classes/Exception#message)
  pub fn message(&self) -> &str {
    &self.message
  }

  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/classes/Exception#sassMessage)
  pub fn sass_message(&self) -> Option<&str> {
    self.sass_message.as_deref()
  }

  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/classes/Exception#sassStack)
  pub fn sass_stack(&self) -> Option<&str> {
    self.sass_stack.as_deref()
  }

  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/classes/Exception#span)
  pub fn span(&self) -> Option<&SourceSpan> {
    self.span.as_ref()
  }
}

impl std::error::Error for Exception {}

impl fmt::Display for Exception {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/classes/Exception#toString)
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl From<CompileFailure> for Exception {
  fn from(failure: CompileFailure) -> Self {
    Self {
      message: failure.formatted,
      sass_message: Some(failure.message),
      sass_stack: Some(failure.stack_trace),
      span: failure.span.map(|span| span.into()),
      source: None,
    }
  }
}

impl From<ProtocolError> for Exception {
  fn from(e: ProtocolError) -> Self {
    Self::new(e.message)
  }
}

impl Exception {
  /// Creates a new Exception with the given message.
  pub fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      sass_message: None,
      sass_stack: None,
      span: None,
      source: None,
    }
  }

  /// Sets the source error of the exception.
  pub fn set_source(
    mut self,
    source: impl std::error::Error + Send + Sync + 'static,
  ) -> Self {
    self.source = Some(Box::new(source));
    self
  }
}
