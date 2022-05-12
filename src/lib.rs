mod api;
mod compile;
mod compiler;
mod compiler_path;
mod connection;
mod dispatcher;
mod error;
mod importer_registry;
mod logger_registry;
mod message_transformer;
mod packet_transformer;
mod pb;
mod request_tracker;

use api::Logger;
pub use error::{Error, Result};

#[derive(Debug, Default)]
pub struct SilentLogger;

impl Logger for SilentLogger {
  fn warn(&self, _message: &str, _options: &api::LoggerWarnOptions) {}

  fn debug(&self, _message: &str, _options: &api::LoggerDebugOptions) {}
}
