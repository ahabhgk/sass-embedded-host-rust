use std::fmt::Debug;

use crate::{
  protocol::{OutputStyle, SourceSpan, Syntax},
  Result, Url,
};

/// https://sass-lang.com/documentation/js-api/interfaces/Options
#[derive(Debug)]
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
  /// https://sass-lang.com/documentation/js-api/interfaces/Options#logger
  pub logger: Option<SassLogger>,
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
      alert_color: None,
      load_paths: None,
      importers: None,
      logger: None,
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
  options: Options,
}

impl OptionsBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn build(self) -> Options {
    self.options
  }

  pub fn alert_ascii(mut self, arg: bool) -> Self {
    self.options.alert_ascii = arg;
    self
  }

  pub fn alert_color(mut self, arg: bool) -> Self {
    self.options.alert_color = Some(arg);
    self
  }

  pub fn load_paths(mut self, arg: impl IntoIterator<Item = String>) -> Self {
    self.options.load_paths = Some(arg.into_iter().collect());
    self
  }

  pub fn load_path(mut self, arg: impl Into<String>) -> Self {
    let mut load_paths = if let Some(load_paths) = self.options.load_paths {
      load_paths
    } else {
      Vec::new()
    };
    load_paths.push(arg.into());
    self.options.load_paths = Some(load_paths);
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

  pub fn logger(mut self, arg: SassLogger) -> Self {
    self.options.logger = Some(arg);
    self
  }

  pub fn sass_importer(mut self, arg: SassImporter) -> Self {
    let mut importers = if let Some(importers) = self.options.importers {
      importers
    } else {
      Vec::new()
    };
    importers.push(arg);
    self.options.importers = Some(importers);
    self
  }

  pub fn sass_importers(mut self, arg: Vec<SassImporter>) -> Self {
    self.options.importers = Some(arg);
    self
  }

  pub fn importer(mut self, arg: impl Into<Box<dyn Importer>>) -> Self {
    let mut importers = if let Some(importers) = self.options.importers {
      importers
    } else {
      Vec::new()
    };
    importers.push(SassImporter::Importer(arg.into()));
    self.options.importers = Some(importers);
    self
  }

  pub fn file_importer(
    mut self,
    arg: impl Into<Box<dyn FileImporter>>,
  ) -> Self {
    let mut importers = if let Some(importers) = self.options.importers {
      importers
    } else {
      Vec::new()
    };
    importers.push(SassImporter::FileImporter(arg.into()));
    self.options.importers = Some(importers);
    self
  }
}

#[derive(Debug, Default)]
pub struct StringOptions {
  pub common: Options,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#importer
  pub importer: Option<SassImporter>,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#syntax
  pub syntax: Syntax,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#url
  pub url: Option<Url>,
}

#[derive(Debug, Default)]
pub struct StringOptionsBuilder {
  options: Options,
  /// https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#importer
  importer: Option<SassImporter>,
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
      common: self.options,
      importer: self.importer,
      syntax: self.syntax,
      url: self.url,
    }
  }

  pub fn input_sass_importer(mut self, arg: SassImporter) -> Self {
    self.importer = Some(arg);
    self
  }

  pub fn input_importer(mut self, arg: impl Into<Box<dyn Importer>>) -> Self {
    self.importer = Some(SassImporter::Importer(arg.into()));
    self
  }

  pub fn input_file_importer(
    mut self,
    arg: impl Into<Box<dyn FileImporter>>,
  ) -> Self {
    self.importer = Some(SassImporter::FileImporter(arg.into()));
    self
  }

  pub fn syntax(mut self, arg: Syntax) -> Self {
    self.syntax = arg;
    self
  }

  pub fn url(mut self, arg: impl Into<Url>) -> Self {
    self.url = Some(arg.into());
    self
  }

  pub fn alert_ascii(mut self, arg: bool) -> Self {
    self.options.alert_ascii = arg;
    self
  }

  pub fn alert_color(mut self, arg: bool) -> Self {
    self.options.alert_color = Some(arg);
    self
  }

  pub fn load_paths(mut self, arg: impl IntoIterator<Item = String>) -> Self {
    self.options.load_paths = Some(arg.into_iter().collect());
    self
  }

  pub fn load_path(mut self, arg: impl Into<String>) -> Self {
    let mut load_paths = if let Some(load_paths) = self.options.load_paths {
      load_paths
    } else {
      Vec::new()
    };
    load_paths.push(arg.into());
    self.options.load_paths = Some(load_paths);
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

  pub fn logger(mut self, arg: SassLogger) -> Self {
    self.options.logger = Some(arg);
    self
  }

  pub fn sass_importer(mut self, arg: SassImporter) -> Self {
    let mut importers = if let Some(importers) = self.options.importers {
      importers
    } else {
      Vec::new()
    };
    importers.push(arg);
    self.options.importers = Some(importers);
    self
  }

  pub fn sass_importers(mut self, arg: Vec<SassImporter>) -> Self {
    self.options.importers = Some(arg);
    self
  }

  pub fn importer(mut self, arg: impl Into<Box<dyn Importer>>) -> Self {
    let mut importers = if let Some(importers) = self.options.importers {
      importers
    } else {
      Vec::new()
    };
    importers.push(SassImporter::Importer(arg.into()));
    self.options.importers = Some(importers);
    self
  }

  pub fn file_importer(
    mut self,
    arg: impl Into<Box<dyn FileImporter>>,
  ) -> Self {
    let mut importers = if let Some(importers) = self.options.importers {
      importers
    } else {
      Vec::new()
    };
    importers.push(SassImporter::FileImporter(arg.into()));
    self.options.importers = Some(importers);
    self
  }
}

pub type SassLogger = Box<dyn Logger>;

pub trait Logger: Debug + Send + Sync {
  fn warn(&self, message: &str, options: &LoggerWarnOptions);

  fn debug(&self, message: &str, options: &LoggerDebugOptions);
}

pub struct LoggerWarnOptions {
  pub deprecation: bool,
  pub span: Option<SourceSpan>,
  pub stack: Option<String>,
}

pub struct LoggerDebugOptions {
  pub span: Option<SourceSpan>,
}

#[derive(Debug)]
pub enum SassImporter {
  Importer(Box<dyn Importer>),
  FileImporter(Box<dyn FileImporter>),
}

/// https://sass-lang.com/documentation/js-api/interfaces/Importer
pub trait Importer: Debug + Send + Sync {
  /// https://sass-lang.com/documentation/js-api/interfaces/Importer#canonicalize
  fn canonicalize<'u, 'o>(
    &self,
    url: &'u str,
    options: &'o ImporterOptions,
  ) -> Result<Option<Url>>;

  /// https://sass-lang.com/documentation/js-api/interfaces/Importer#load
  fn load<'u>(&self, canonical_url: &'u Url) -> Result<Option<ImporterResult>>;
}

pub struct ImporterOptions {
  pub from_import: bool,
}

/// https://sass-lang.com/documentation/js-api/interfaces/FileImporter
pub trait FileImporter: Debug + Send + Sync {
  /// https://sass-lang.com/documentation/js-api/interfaces/FileImporter#findFileUrl
  fn find_file_url<'u, 'o>(
    &self,
    url: &'u str,
    options: &'o ImporterOptions,
  ) -> Result<Option<Url>>;
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
