use std::fmt::Display;

use url::Url;

use crate::pb::{
  inbound_message::CompileRequest,
  outbound_message::{
    compile_response::{self, CompileFailure, CompileSuccess},
    CompileResponse,
  },
  OutputStyle, ProtocolError, SourceSpan, Syntax,
};

/// https://sass-lang.com/documentation/js-api/interfaces/Options
#[derive(Debug)]
pub struct Options {
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#alertAscii
  pub alert_ascii: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#alertColor
  pub alert_color: bool,
  // /// https://sass-lang.com/documentation/js-api/interfaces/Options#functions
  // pub functions
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#importers
  // pub importers: Option<Vec<SassImporter>>,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#loadPaths
  pub load_paths: Vec<String>,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#logger
  // pub logger: Option<SassLogger>,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#quietDeps
  pub quiet_deps: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#sourceMap
  pub source_map: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#sourceMapIncludeSources
  pub source_map_include_sources: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#style
  pub style: OutputStyle,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#verbose
  pub verbose: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#charset
  pub charset: bool,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      alert_ascii: false,
      alert_color: false,
      load_paths: Vec::new(),
      quiet_deps: false,
      source_map: false,
      source_map_include_sources: false,
      style: OutputStyle::default(),
      verbose: false,
      charset: true,
    }
  }
}

#[derive(Debug, Default)]
pub struct OptionsBuilder {
  common: Options,
}

impl OptionsBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn build(self) -> Options {
    self.common
  }

  pub fn alert_ascii(mut self, arg: bool) -> Self {
    self.common.alert_ascii = arg;
    self
  }

  pub fn alert_color(mut self, arg: bool) -> Self {
    self.common.alert_color = arg;
    self
  }

  pub fn load_paths(mut self, arg: Vec<String>) -> Self {
    self.common.load_paths = arg;
    self
  }

  pub fn load_path(mut self, arg: &str) -> Self {
    self.common.load_paths.push(arg.to_owned());
    self
  }

  pub fn quiet_deps(mut self, arg: bool) -> Self {
    self.common.quiet_deps = arg;
    self
  }

  pub fn source_map(mut self, arg: bool) -> Self {
    self.common.source_map = arg;
    self
  }

  pub fn source_map_include_sources(mut self, arg: bool) -> Self {
    self.common.source_map_include_sources = arg;
    self
  }

  pub fn style(mut self, arg: OutputStyle) -> Self {
    self.common.style = arg;
    self
  }

  pub fn verbose(mut self, arg: bool) -> Self {
    self.common.verbose = arg;
    self
  }

  pub fn charset(mut self, arg: bool) -> Self {
    self.common.charset = arg;
    self
  }
}

#[derive(Debug, Default)]
pub struct StringOptions {
  pub options: Options,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#importer
  // pub importer: SassImporter,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#syntax
  pub syntax: Syntax,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#url
  pub url: Option<Url>,
}

#[derive(Debug, Default)]
pub struct StringOptionsBuilder {
  options: Options,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#importer
  // pub importer: SassImporter,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#syntax
  syntax: Syntax,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#url
  url: Option<Url>,
}

impl StringOptionsBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn build(self) -> StringOptions {
    StringOptions {
      options: self.options,
      syntax: self.syntax,
      url: self.url,
    }
  }

  pub fn syntax(mut self, arg: Syntax) -> Self {
    self.syntax = arg;
    self
  }

  pub fn url(mut self, arg: Url) -> Self {
    self.url = Some(arg);
    self
  }

  pub fn alert_ascii(mut self, arg: bool) -> Self {
    self.options.alert_ascii = arg;
    self
  }

  pub fn alert_color(mut self, arg: bool) -> Self {
    self.options.alert_color = arg;
    self
  }

  pub fn load_paths(mut self, arg: Vec<String>) -> Self {
    self.options.load_paths = arg;
    self
  }

  pub fn load_path(mut self, arg: &str) -> Self {
    self.options.load_paths.push(arg.to_owned());
    self
  }

  pub fn quiet_deps(mut self, arg: bool) -> Self {
    self.options.quiet_deps = arg;
    self
  }

  pub fn source_map(mut self, arg: bool) -> Self {
    self.options.source_map = arg;
    self
  }

  pub fn source_map_include_sources(mut self, arg: bool) -> Self {
    self.options.source_map_include_sources = arg;
    self
  }

  pub fn style(mut self, arg: OutputStyle) -> Self {
    self.options.style = arg;
    self
  }

  pub fn verbose(mut self, arg: bool) -> Self {
    self.options.verbose = arg;
    self
  }

  pub fn charset(mut self, arg: bool) -> Self {
    self.options.charset = arg;
    self
  }
}

impl From<Options> for CompileRequest {
  fn from(options: Options) -> Self {
    Self {
      style: options.style as i32,
      source_map: options.source_map,
      // importers: (),
      // global_functions: (),
      alert_color: options.alert_color,
      alert_ascii: options.alert_ascii,
      verbose: options.verbose,
      quiet_deps: options.quiet_deps,
      source_map_include_sources: options.source_map_include_sources,
      ..Default::default()
    }
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

pub type Result<T> = std::result::Result<T, Exception>;

#[derive(Debug)]
pub struct Exception {
  message: String,
  sass_message: Option<String>,
  sass_stack: Option<String>,
  span: Option<SourceSpan>,
}

impl Exception {
  pub fn message(&self) -> &str {
    &self.message
  }

  pub fn sass_message(&self) -> Option<&str> {
    self.sass_message.as_deref()
  }

  pub fn sass_stack(&self) -> Option<&str> {
    self.sass_stack.as_deref()
  }

  pub fn span(&self) -> Option<&SourceSpan> {
    self.span.as_ref()
  }
}

impl std::error::Error for Exception {}

impl Display for Exception {
  /// https://sass-lang.com/documentation/js-api/classes/Exception#toString
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl From<CompileFailure> for Exception {
  fn from(failure: CompileFailure) -> Self {
    Self {
      message: failure.formatted,
      sass_message: Some(failure.message),
      sass_stack: Some(failure.stack_trace),
      span: failure.span,
    }
  }
}

impl From<ProtocolError> for Exception {
  fn from(e: ProtocolError) -> Self {
    Self {
      message: e.message,
      sass_message: None,
      sass_stack: None,
      span: None,
    }
  }
}
