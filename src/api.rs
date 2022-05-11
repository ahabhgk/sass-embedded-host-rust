use std::fmt::{Debug, Display};

use async_trait::async_trait;
use url::Url;

pub use crate::{
  error::Result,
  pb::{
    outbound_message::compile_response::CompileFailure, OutputStyle,
    SourceSpan, Syntax,
  },
};

/// https://sass-lang.com/documentation/js-api/interfaces/Options
#[derive(Debug, Default)]
pub struct Options {
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#alertAscii
  pub alert_ascii: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#alertColor
  pub alert_color: Option<bool>,
  // /// https://sass-lang.com/documentation/js-api/interfaces/Options#functions
  // pub functions
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#importers
  pub importers: Option<Vec<SassImporter>>,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#loadPaths
  pub load_paths: Option<Vec<String>>,
  // /// https://sass-lang.com/documentation/js-api/interfaces/Options#logger
  // pub logger
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#quietDeps
  pub quiet_deps: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#sourceMap
  pub source_map: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#sourceMapIncludeSources
  pub source_map_include_sources: bool,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#style
  pub style: OutputStyle,
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#verbose
  pub verbose: Option<bool>,
}

/// https://sass-lang.com/documentation/js-api/modules#StringOptions
#[derive(Debug)]
pub enum StringOptions {
  WithImporter(StringOptionsWithImporter),
  WithoutImporter(StringOptionsWithoutImporter),
}

impl StringOptions {
  pub fn get_options_mut(&mut self) -> &mut Options {
    match self {
      StringOptions::WithImporter(o) => &mut o.base.base,
      StringOptions::WithoutImporter(o) => &mut o.base,
    }
  }
}

/// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter
#[derive(Debug, Default)]
pub struct StringOptionsWithoutImporter {
  /// extends [Options]
  pub base: Options,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#syntax
  pub syntax: Syntax,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#url
  pub url: Option<Url>,
}

/// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter
#[derive(Debug)]
pub struct StringOptionsWithImporter {
  /// extends [StringOptionsWithoutImporter]
  pub base: StringOptionsWithoutImporter,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#importer
  pub importer: SassImporter,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#url
  pub url: Url,
}

#[derive(Debug)]
pub enum SassImporter {
  Importer(Box<dyn Importer>),
  FileImporter(Box<dyn FileImporter>),
}

/// https://sass-lang.com/documentation/js-api/interfaces/Importer
#[async_trait]
pub trait Importer: Debug {
  /// https://sass-lang.com/documentation/js-api/interfaces/Importer#canonicalize
  async fn canonicalize(
    &self,
    url: &str,
    options: &ImporterOptions,
  ) -> Result<Option<Url>>;

  /// https://sass-lang.com/documentation/js-api/interfaces/Importer#load
  async fn load(&self, canonicalUrl: &Url) -> Result<Option<ImporterResult>>;
}

pub struct ImporterOptions {
  pub from_import: bool,
}

/// https://sass-lang.com/documentation/js-api/interfaces/FileImporter
#[async_trait]
pub trait FileImporter: Debug {
  /// https://sass-lang.com/documentation/js-api/interfaces/FileImporter#findFileUrl
  async fn find_file_url(
    &self,
    url: &str,
    options: &ImporterOptions,
  ) -> Result<Option<Url>>;
}

/// https://sass-lang.com/documentation/js-api/classes/Exception
#[derive(Debug)]
pub struct Exception {
  /// https://sass-lang.com/documentation/js-api/classes/Exception#message
  pub message: String,
  /// https://sass-lang.com/documentation/js-api/classes/Exception#name
  pub name: String,
  /// https://sass-lang.com/documentation/js-api/classes/Exception#sassMessage
  sass_message: String,
  /// https://sass-lang.com/documentation/js-api/classes/Exception#sassStack
  sass_stack: String,
  /// https://sass-lang.com/documentation/js-api/classes/Exception#span
  span: Option<SourceSpan>,
  // /// https://sass-lang.com/documentation/js-api/classes/Exception#stack
  // pub stack: Option<String>,
  // TODO: prepareStackTrace, stackTraceLimit, captureStackTrace
}

impl Exception {
  pub fn new(failure: CompileFailure) -> Self {
    Self {
      message: failure.formatted,
      name: "Error".to_string(),
      sass_message: failure.message,
      sass_stack: failure.stack_trace,
      span: failure.span,
    }
  }

  pub fn sass_message(&self) -> &str {
    &self.sass_message
  }

  pub fn sass_stack(&self) -> &str {
    &self.sass_stack
  }

  pub fn span(&self) -> &Option<SourceSpan> {
    &self.span
  }
}

impl std::error::Error for Exception {}

impl Display for Exception {
  /// https://sass-lang.com/documentation/js-api/classes/Exception#toString
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

/// https://sass-lang.com/documentation/js-api/interfaces/ImporterResult
pub struct ImporterResult {
  /// https://sass-lang.com/documentation/js-api/interfaces/ImporterResult#contents
  pub contents: String,
  /// https://sass-lang.com/documentation/js-api/interfaces/ImporterResult#sourceMapUrl
  pub source_map_url: Option<String>,
  /// https://sass-lang.com/documentation/js-api/interfaces/ImporterResult#syntax
  pub syntax: Syntax,
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
