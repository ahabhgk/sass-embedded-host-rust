use std::ffi::OsStr;

use crate::{
  channel::Channel,
  importer_registry::ImporterRegistry,
  logger_registry::LoggerRegistry,
  protocol::{
    inbound_message::{compile_request::Input, CompileRequest},
    outbound_message::{
      compile_response::{self, CompileSuccess},
      CompileResponse,
    },
  },
  Exception, Options, Result,
};

#[derive(Debug)]
pub struct Embedded {
  channel: Channel,
}

impl Embedded {
  pub fn new(exe_path: impl AsRef<OsStr>) -> Self {
    Self {
      channel: Channel::new(exe_path),
    }
  }

  pub fn compile(
    &mut self,
    path: impl Into<String>,
    mut options: Options,
  ) -> Result<CompileResult> {
    let logger_registry = LoggerRegistry::new(options.logger.take());
    let importer_registry = ImporterRegistry::new(
      options.importers.take(),
      options.load_paths.take(),
    );

    let request = CompileRequest {
      style: options.style as i32,
      source_map: options.source_map,
      alert_color: options.alert_color,
      alert_ascii: options.alert_ascii,
      verbose: options.verbose,
      quiet_deps: options.quiet_deps,
      source_map_include_sources: options.source_map_include_sources,
      charset: options.charset,
      importers: importer_registry.importers(),
      input: Some(Input::Path(path.into())),
      ..Default::default()
    };

    let conn = self
      .channel
      .connect(Some(logger_registry), Some(importer_registry));
    let response = conn.compile_request(request)?;
    Ok(CompileResult::try_from(response)?)
  }

  pub fn info(&mut self) -> Result<String> {
    let conn = self.channel.connect(None, None);
    let response = conn.version_request()?;
    Ok(format!(
      "sass-embedded\t#{}",
      response.implementation_version
    ))
  }
}

/// https://sass-lang.com/documentation/js-api/interfaces/CompileResult
#[derive(Debug)]
pub struct CompileResult {
  /// https://sass-lang.com/documentation/js-api/interfaces/CompileResult#css
  pub css: String,
  /// https://sass-lang.com/documentation/js-api/interfaces/CompileResult#loadedUrls
  pub loaded_urls: Vec<String>,
  /// https://sass-lang.com/documentation/js-api/interfaces/CompileResult#sourceMap
  pub source_map: Option<String>,
}

impl TryFrom<CompileResponse> for CompileResult {
  type Error = Exception;

  fn try_from(response: CompileResponse) -> Result<Self> {
    let res = response.result.unwrap();
    match res {
      compile_response::Result::Success(success) => Ok(success.into()),
      compile_response::Result::Failure(failure) => {
        Err(Exception::from(failure))
      }
    }
  }
}

impl From<CompileSuccess> for CompileResult {
  fn from(s: CompileSuccess) -> Self {
    Self {
      css: s.css,
      loaded_urls: s.loaded_urls,
      source_map: if s.source_map.is_empty() {
        None
      } else {
        Some(s.source_map)
      },
    }
  }
}
