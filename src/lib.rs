mod api;
mod channel;
mod compiler;
mod connection;
mod dispatcher;
mod embedded;
mod error;
mod host;
mod protocol;
mod varint;

#[cfg(feature = "legacy")]
pub mod legacy;

pub use api::{
  CompileResult, FileImporter, Importer, ImporterOptions, ImporterResult,
  Logger, LoggerDebugOptions, LoggerWarnOptions, Options, OptionsBuilder,
  SassImporter, SassLogger, StringOptions, StringOptionsBuilder,
};
pub use embedded::{Embedded, Embedded as Sass};
pub use error::{Exception, Result};
pub use protocol::{OutputStyle, SourceSpan, Syntax};
pub use url::{self, Url};

#[derive(Debug, Default, Clone)]
pub struct Silent;

impl Logger for Silent {
  fn debug(&self, _: &str, _: &LoggerDebugOptions) {}

  fn warn(&self, _: &str, _: &LoggerWarnOptions) {}
}
