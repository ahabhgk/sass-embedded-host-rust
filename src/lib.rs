mod api;
mod compile;
mod compiler;
mod compiler_path;
mod dispatcher;
mod error;
mod importer_registry;
mod logger_registry;
mod packet_transformer;
mod pb;
mod request_tracker;

use api::Logger;
pub use api::Options;
pub use compile::{compile, compile_string};
pub use error::{Error, Result};

#[derive(Debug, Default, Clone)]
pub struct SilentLogger;

impl Logger for SilentLogger {
  fn warn(&self, _message: &str, _options: &api::LoggerWarnOptions) {}

  fn debug(&self, _message: &str, _options: &api::LoggerDebugOptions) {}
}
