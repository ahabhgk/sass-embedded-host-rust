//! A Rust library that will communicate with Embedded Dart Sass using the Embedded Sass protocol
//!
//! ```no_run
//! use sass_embedded::{Sass, StringOptions};
//!
//! let mut sass = Sass::new("path/to/sass_embedded").unwrap();
//! let res = sass.compile_string("a {b: c}", StringOptions::default()).unwrap();
//! println!("{:?}", res);
//! ```
//!
//! # features
//!
//! - **`legacy`**: support for [sass's legacy APIs](https://sass-lang.com/documentation/js-api/modules#renderSync)
//!

#![forbid(unsafe_code)]
#![deny(missing_docs)]

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

/// A Logger that silently ignores all warnings and debug messages.
///
/// More information:
///  - [Sass documentation][https://sass-lang.com/documentation/js-api/modules/Logger#silent]
#[derive(Debug, Default, Clone)]
pub struct Silent;

impl Logger for Silent {
  fn debug(&self, _: &str, _: &LoggerDebugOptions) {}

  fn warn(&self, _: &str, _: &LoggerWarnOptions) {}
}
