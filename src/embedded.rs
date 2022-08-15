use std::{ffi::OsStr, path::Path};

use atty::Stream;

use crate::{
  channel::Channel,
  host::ImporterRegistry,
  host::{Host, LoggerRegistry},
  protocol::inbound_message::{
    compile_request::{Input, StringInput},
    CompileRequest,
  },
  CompileResult, Options, Result, StringOptions,
};
#[cfg(feature = "legacy")]
use crate::{
  legacy::LEGACY_IMPORTER_PROTOCOL, protocol::inbound_message::compile_request,
};

/// The sass-embedded compiler for rust host.
#[derive(Debug)]
pub struct Embedded {
  channel: Channel,
}

impl Embedded {
  /// Creates a sass-embedded compiler and connects with the dart-sass-embedded.
  ///
  /// ```no_run
  /// let mut sass = Sass::new("path/to/sass_embedded").unwrap();
  /// ```
  pub fn new(exe_path: impl AsRef<OsStr>) -> Result<Self> {
    Ok(Self {
      channel: Channel::new(exe_path)?,
    })
  }

  /// Compiles the Sass file at path to CSS. If it succeeds it returns a [CompileResult],
  /// and if it fails it throws an [Exception].
  /// 
  /// ```no_run
  /// let mut sass = Sass::new("path/to/sass_embedded").unwrap();
  /// let res = sass.compile("../styles/a.scss", Options::default()).unwrap();
  /// ```
  ///
  /// More information:
  ///  - [Sass documentation](https://sass-lang.com/documentation/js-api/modules#compile)
  pub fn compile(
    &mut self,
    path: impl AsRef<Path>,
    options: Options,
  ) -> Result<CompileResult> {
    let mut logger_registry = LoggerRegistry::default();
    let mut importer_registry = ImporterRegistry::default();
    let importers = importer_registry
      .register_all(options.importers, options.load_paths)
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
      input: Some(Input::Path(path.as_ref().to_str().unwrap().to_string())),
      // id: set in compile_request
      // global_functions: not implemented
      ..Default::default()
    };

    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host)?;
    let response = conn.compile_request(request)?;
    CompileResult::try_from(response)
  }

  /// Compiles a stylesheet whose contents is source to CSS. If it succeeds it returns
  /// a [CompileResult], and if it fails it throws an [Exception].
  /// 
  /// ```no_run
  /// let mut sass = Sass::new("path/to/sass_embedded").unwrap();
  /// let res = sass.compile_string("a {b: c}", StringOptions::default()).unwrap();
  /// ```
  ///
  /// More information:
  ///  - [Sass documentation](https://sass-lang.com/documentation/js-api/modules#compileString)
  pub fn compile_string(
    &mut self,
    source: impl Into<String>,
    options: StringOptions,
  ) -> Result<CompileResult> {
    let mut logger_registry = LoggerRegistry::default();
    let mut importer_registry = ImporterRegistry::default();
    let importers = importer_registry
      .register_all(options.common.importers, options.common.load_paths)
      .collect();
    if let Some(l) = options.common.logger {
      logger_registry.register(l);
    }

    #[cfg(feature = "legacy")]
    let importer = if let Some(input_importer) = options.input_importer {
      Some(importer_registry.register(input_importer))
    } else if matches!(&options.url, Some(u) if u.to_string() == LEGACY_IMPORTER_PROTOCOL)
    {
      Some(compile_request::Importer {
        importer: Some(compile_request::importer::Importer::Path(
          std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        )),
      })
    } else {
      None
    };

    #[cfg(feature = "legacy")]
    let url = options
      .url
      .map(|url| url.to_string())
      .filter(|url| url != LEGACY_IMPORTER_PROTOCOL)
      .unwrap_or_default();

    #[cfg(not(feature = "legacy"))]
    let importer = options
      .input_importer
      .map(|i| importer_registry.register(i));

    #[cfg(not(feature = "legacy"))]
    let url = options.url.map(|url| url.to_string()).unwrap_or_default();

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
        url,
        syntax: options.syntax as i32,
        importer,
      })),
      // id: set in compile_request
      // global_functions: not implemented
      ..Default::default()
    };

    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host)?;
    let response = conn.compile_request(request)?;
    CompileResult::try_from(response)
  }

  /// Gets the version of the sass-embedded compiler.
  pub fn info(&mut self) -> Result<String> {
    let logger_registry = LoggerRegistry::default();
    let importer_registry = ImporterRegistry::default();
    let host = Host::new(importer_registry, logger_registry);
    let conn = self.channel.connect(host)?;
    let response = conn.version_request()?;
    Ok(format!(
      "sass-embedded\t#{}",
      response.implementation_version
    ))
  }
}
