use std::ffi::OsStr;

use crate::{
  channel::Channel,
  host::ImporterRegistry,
  host::{Host, LoggerRegistry},
  protocol::{
    inbound_message::{
      compile_request::{Input, StringInput},
      CompileRequest,
    },
    outbound_message::{
      compile_response::{self, CompileSuccess},
      CompileResponse,
    },
  },
  Exception, Options, Result, StringOptions,
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
    options: Options,
  ) -> Result<CompileResult> {
    let mut logger_registry = LoggerRegistry::default();
    let mut importer_registry = ImporterRegistry::default();
    let importers = importer_registry
      .register_all(
        options.importers.unwrap_or_default(),
        options.load_paths.unwrap_or_default(),
      )
      .collect();
    options.logger.map(|l| logger_registry.register(l));

    let request = CompileRequest {
      style: options.style as i32,
      source_map: options.source_map,
      alert_color: options.alert_color,
      alert_ascii: options.alert_ascii,
      verbose: options.verbose,
      quiet_deps: options.quiet_deps,
      source_map_include_sources: options.source_map_include_sources,
      charset: options.charset,
      importers,
      input: Some(Input::Path(path.into())),
      // id: set in compile_request
      // global_functions: not implemented
      ..Default::default()
    };

    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host);
    let response = conn.compile_request(request)?;
    Ok(CompileResult::try_from(response)?)
  }

  pub fn compile_string(
    &mut self,
    source: impl Into<String>,
    options: StringOptions,
  ) -> Result<CompileResult> {
    let mut logger_registry = LoggerRegistry::default();
    let mut importer_registry = ImporterRegistry::default();
    let importers = importer_registry
      .register_all(
        options.common.importers.unwrap_or_default(),
        options.common.load_paths.unwrap_or_default(),
      )
      .collect();
    options.common.logger.map(|l| logger_registry.register(l));

    let request = CompileRequest {
      style: options.common.style as i32,
      source_map: options.common.source_map,
      alert_color: options.common.alert_color,
      alert_ascii: options.common.alert_ascii,
      verbose: options.common.verbose,
      quiet_deps: options.common.quiet_deps,
      source_map_include_sources: options.common.source_map_include_sources,
      charset: options.common.charset,
      importers,
      input: Some(Input::String(StringInput {
        source: source.into(),
        url: options.url.map(|url| url.to_string()).unwrap_or_default(),
        syntax: options.syntax as i32,
        importer: options.importer.map(|i| importer_registry.register(i)),
      })),
      // id: set in compile_request
      // global_functions: not implemented
      ..Default::default()
    };

    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host);
    let response = conn.compile_request(request)?;
    Ok(CompileResult::try_from(response)?)
  }

  pub fn info(&mut self) -> Result<String> {
    let logger_registry = LoggerRegistry::default();
    let importer_registry = ImporterRegistry::default();
    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host);
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
