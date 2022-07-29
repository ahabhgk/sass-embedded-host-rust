mod importer_registry;
mod logger_registry;

pub use importer_registry::ImporterRegistry;
pub use logger_registry::LoggerRegistry;

use crate::protocol::{
  inbound_message::{CanonicalizeResponse, FileImportResponse, ImportResponse},
  outbound_message::{
    CanonicalizeRequest, FileImportRequest, ImportRequest, LogEvent,
  },
};

#[derive(Debug, Default)]
pub struct Host {
  importer: ImporterRegistry,
  logger: LoggerRegistry,
}

impl Host {
  pub fn new(importer: ImporterRegistry, logger: LoggerRegistry) -> Self {
    Self { importer, logger }
  }

  pub fn canonicalize(
    &self,
    request: &CanonicalizeRequest,
  ) -> CanonicalizeResponse {
    self.importer.canonicalize(request)
  }

  pub fn import(&self, request: &ImportRequest) -> ImportResponse {
    self.importer.import(request)
  }

  pub fn file_import(&self, request: &FileImportRequest) -> FileImportResponse {
    self.importer.file_import(request)
  }

  pub fn log(&self, event: LogEvent) {
    self.logger.log(event);
  }
}
