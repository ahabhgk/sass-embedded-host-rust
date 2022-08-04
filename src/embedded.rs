use std::{env, ffi::OsStr};

use atty::Stream;

use crate::{
  channel::Channel,
  host::ImporterRegistry,
  host::{Host, LoggerRegistry},
  legacy::{LegacyOptions, LegacyResult, LEGACY_IMPORTER_PROTOCOL},
  protocol::inbound_message::{
    compile_request::{self, Input, StringInput},
    CompileRequest,
  },
  CompileResult, Options, Result, StringOptions,
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
        options.importers,
        options.load_paths,
      )
      .collect();
    if let Some(l) = options.logger {
      logger_registry.register(l);
    }

    let request = CompileRequest {
      style: options.style as i32,
      source_map: options.source_map,
      alert_color: options
        .alert_color
        .unwrap_or_else(|| atty::is(Stream::Stdout)),
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
    CompileResult::try_from(response)
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
        options.common.importers,
        options.common.load_paths,
      )
      .collect();
    if let Some(l) = options.common.logger {
      logger_registry.register(l);
    }
    let importer = if matches!(&options.url, Some(u) if u.to_string() == LEGACY_IMPORTER_PROTOCOL)
    {
      Some(compile_request::Importer {
        importer: Some(compile_request::importer::Importer::Path(
          env::current_dir().unwrap().to_string_lossy().to_string(),
        )),
      })
    } else {
      options
        .input_importer
        .map(|i| importer_registry.register(i))
    };

    let request = CompileRequest {
      style: options.common.style as i32,
      source_map: options.common.source_map,
      alert_color: options
        .common
        .alert_color
        .unwrap_or_else(|| atty::is(Stream::Stdout)),
      alert_ascii: options.common.alert_ascii,
      verbose: options.common.verbose,
      quiet_deps: options.common.quiet_deps,
      source_map_include_sources: options.common.source_map_include_sources,
      charset: options.common.charset,
      importers,
      input: Some(Input::String(StringInput {
        source: source.into(),
        url: options
          .url
          .map(|url| url.to_string())
          .filter(|url| url != LEGACY_IMPORTER_PROTOCOL)
          .unwrap_or_default(),
        syntax: options.syntax as i32,
        importer,
      })),
      // id: set in compile_request
      // global_functions: not implemented
      ..Default::default()
    };

    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host);
    let response = conn.compile_request(request)?;
    CompileResult::try_from(response)
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
