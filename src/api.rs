use std::{
  fmt::Debug,
  path::{Path, PathBuf},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
  protocol::{
    self,
    outbound_message::{
      compile_response::{self, CompileSuccess},
      CompileResponse,
    },
  },
  Exception, Result, Url,
};

/// Options that can be passed to [Sass::compile].
///
/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Options {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#alertAscii)
  pub alert_ascii: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#alertColor)
  pub alert_color: Option<bool>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#importers)
  #[cfg_attr(feature = "serde", serde(skip))]
  pub importers: Vec<SassImporter>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#loadPaths)
  pub load_paths: Vec<PathBuf>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#logger)
  #[cfg_attr(feature = "serde", serde(skip))]
  pub logger: Option<BoxedLogger>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#quietDeps)
  pub quiet_deps: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#sourceMap)
  pub source_map: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#sourceMapIncludeSources)
  pub source_map_include_sources: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#style)
  pub style: OutputStyle,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#verbose)
  pub verbose: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Options#charset)
  pub charset: bool,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      alert_ascii: false,
      alert_color: None,
      load_paths: Vec::new(),
      importers: Vec::new(),
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

/// A builder for [Options].
#[derive(Debug, Default)]
pub struct OptionsBuilder {
  options: Options,
}

impl OptionsBuilder {
  /// Creates a new [OptionsBuilder].
  pub fn new() -> Self {
    Self::default()
  }

  /// Build the [Options].
  pub fn build(self) -> Options {
    self.options
  }

  /// Sets the [Options]'s [alert_ascii] field.
  pub fn alert_ascii(mut self, arg: impl Into<bool>) -> Self {
    self.options.alert_ascii = arg.into();
    self
  }

  /// Sets the [Options]'s [alert_color] field.
  pub fn alert_color(mut self, arg: impl Into<bool>) -> Self {
    self.options.alert_color = Some(arg.into());
    self
  }

  /// Sets the [Options]'s [load_paths] field.
  pub fn load_paths<P: AsRef<Path>>(mut self, arg: impl AsRef<[P]>) -> Self {
    self.options.load_paths =
      arg.as_ref().iter().map(|p| p.as_ref().to_owned()).collect();
    self
  }

  /// Adds a load_path to the [Options]'s [load_paths] field.
  pub fn load_path(mut self, arg: impl AsRef<Path>) -> Self {
    self.options.load_paths.push(arg.as_ref().to_owned());
    self
  }

  /// Sets the [Options]'s [quiet_deps] field.
  pub fn quiet_deps(mut self, arg: impl Into<bool>) -> Self {
    self.options.quiet_deps = arg.into();
    self
  }

  /// Sets the [Options]'s [source_map] field.
  pub fn source_map(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map = arg.into();
    self
  }

  /// Sets the [Options]'s [source_map_include_sources] field.
  pub fn source_map_include_sources(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map_include_sources = arg.into();
    self
  }

  /// Sets the [Options]'s [style] field.
  pub fn style(mut self, arg: impl Into<OutputStyle>) -> Self {
    self.options.style = arg.into();
    self
  }

  /// Sets the [Options]'s [verbose] field.
  pub fn verbose(mut self, arg: impl Into<bool>) -> Self {
    self.options.verbose = arg.into();
    self
  }

  /// Sets the [Options]'s [charset] field.
  pub fn charset(mut self, arg: impl Into<bool>) -> Self {
    self.options.charset = arg.into();
    self
  }

  /// Sets the [Options]'s [logger] field.
  pub fn logger<L: 'static + Logger>(mut self, arg: L) -> Self {
    self.options.logger = Some(Box::new(arg));
    self
  }

  /// Adds a [SassImporter] to the [Options]'s [importers] field.
  pub fn sass_importer(mut self, arg: impl Into<SassImporter>) -> Self {
    self.options.importers.push(arg.into());
    self
  }

  /// Sets the [Options]'s [importers] field.
  pub fn sass_importers(
    mut self,
    arg: impl IntoIterator<Item = impl Into<SassImporter>>,
  ) -> Self {
    self.options.importers = arg.into_iter().map(|i| i.into()).collect();
    self
  }

  /// Adds a [Importer] to the [Options]'s [importers] field.
  pub fn importer<I: 'static + Importer>(mut self, arg: I) -> Self {
    self
      .options
      .importers
      .push(SassImporter::Importer(Box::new(arg) as Box<dyn Importer>));
    self
  }

  /// Adds a [FileImporter] to the [Options]'s [importers] field.
  pub fn file_importer<I: 'static + FileImporter>(mut self, arg: I) -> Self {
    self.options.importers.push(SassImporter::FileImporter(
      Box::new(arg) as Box<dyn FileImporter>
    ));
    self
  }
}

/// Options that can be passed to [Sass::compile_string].
///
/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/modules#StringOptions)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default)]
pub struct StringOptions {
  /// Field for [Options]
  pub common: Options,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#importer)
  #[cfg_attr(feature = "serde", serde(skip))]
  pub input_importer: Option<SassImporter>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#syntax)
  ///  - [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#syntax)
  pub syntax: Syntax,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithoutImporter#url)
  ///  - [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/StringOptionsWithImporter#url)
  pub url: Option<Url>,
}

/// A builder for [StringOptions].
#[derive(Debug, Default)]
pub struct StringOptionsBuilder {
  options: Options,
  input_importer: Option<SassImporter>,
  syntax: Syntax,
  url: Option<Url>,
}

impl StringOptionsBuilder {
  /// Creates a new [StringOptionsBuilder].
  pub fn new() -> Self {
    Self::default()
  }

  /// Build the [StringOptions].
  pub fn build(self) -> StringOptions {
    StringOptions {
      common: self.options,
      input_importer: self.input_importer,
      syntax: self.syntax,
      url: self.url,
    }
  }

  /// Sets the [StringOptions]'s [input_importer] field with a [SassImporter].
  pub fn input_sass_importer(mut self, arg: impl Into<SassImporter>) -> Self {
    self.input_importer = Some(arg.into());
    self
  }

  /// Sets the [StringOptions]'s [input_importer] field with a [Importer].
  pub fn input_importer<I: 'static + Importer>(mut self, arg: I) -> Self {
    self.input_importer = Some(SassImporter::Importer(Box::new(arg)));
    self
  }

  /// Sets the [StringOptions]'s [input_importer] field with a [FileImporter].
  pub fn input_file_importer<I: 'static + FileImporter>(
    mut self,
    arg: I,
  ) -> Self {
    self.input_importer = Some(SassImporter::FileImporter(Box::new(arg)));
    self
  }

  /// Sets the [StringOptions]'s [syntax] field.
  pub fn syntax(mut self, arg: impl Into<Syntax>) -> Self {
    self.syntax = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [url] field.
  pub fn url(mut self, arg: impl Into<Url>) -> Self {
    self.url = Some(arg.into());
    self
  }

  /// Sets the [StringOptions]'s [alert_ascii] field.
  pub fn alert_ascii(mut self, arg: impl Into<bool>) -> Self {
    self.options.alert_ascii = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [alert_color] field.
  pub fn alert_color(mut self, arg: impl Into<bool>) -> Self {
    self.options.alert_color = Some(arg.into());
    self
  }

  /// Sets the [StringOptions]'s [load_paths] field.
  pub fn load_paths<P: AsRef<Path>>(mut self, arg: impl AsRef<[P]>) -> Self {
    self.options.load_paths =
      arg.as_ref().iter().map(|p| p.as_ref().to_owned()).collect();
    self
  }

  /// Adds a [Path] to the [StringOptions]'s [load_paths] field.
  pub fn load_path(mut self, arg: impl AsRef<Path>) -> Self {
    self.options.load_paths.push(arg.as_ref().to_owned());
    self
  }

  /// Sets the [StringOptions]'s [quiet_deps] field.
  pub fn quiet_deps(mut self, arg: impl Into<bool>) -> Self {
    self.options.quiet_deps = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [source_map] field.
  pub fn source_map(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [source_map_include_sources] field.
  pub fn source_map_include_sources(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map_include_sources = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [style] field.
  pub fn style(mut self, arg: impl Into<OutputStyle>) -> Self {
    self.options.style = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [verbose] field.
  pub fn verbose(mut self, arg: impl Into<bool>) -> Self {
    self.options.verbose = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [charset] field.
  pub fn charset(mut self, arg: impl Into<bool>) -> Self {
    self.options.charset = arg.into();
    self
  }

  /// Sets the [StringOptions]'s [logger] field.
  pub fn logger<L: 'static + Logger>(mut self, arg: L) -> Self {
    self.options.logger = Some(Box::new(arg));
    self
  }

  /// Adds a [SassImporter] to the [StringOptions]'s [importers] field.
  pub fn sass_importer(mut self, arg: impl Into<SassImporter>) -> Self {
    self.options.importers.push(arg.into());
    self
  }

  /// Sets the [StringOptions]'s [importers] field with [SassImporter]s.
  pub fn sass_importers(
    mut self,
    arg: impl IntoIterator<Item = impl Into<SassImporter>>,
  ) -> Self {
    self.options.importers = arg.into_iter().map(|i| i.into()).collect();
    self
  }

  /// Adds a [Importer] to the [StringOptions]'s [importers] field.
  pub fn importer<I: 'static + Importer>(mut self, arg: I) -> Self {
    self
      .options
      .importers
      .push(SassImporter::Importer(Box::new(arg)));
    self
  }

  /// Adds a [FileImporter] to the [StringOptions]'s [importers] field.
  pub fn file_importer<I: 'static + FileImporter>(mut self, arg: I) -> Self {
    self
      .options
      .importers
      .push(SassImporter::FileImporter(Box::new(arg)));
    self
  }
}

/// A type alias for [Box<dyn Logger>].
pub type BoxedLogger = Box<dyn Logger>;

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Logger)
pub trait Logger: Debug + Send + Sync {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Logger#warn)
  fn warn(&self, _message: &str, options: &LoggerWarnOptions) {
    eprintln!("{}", options.formatted);
  }

  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Logger#debug)
  fn debug(&self, _message: &str, options: &LoggerDebugOptions) {
    eprintln!("{}", options.formatted);
  }
}

/// Options for [Logger::warn].
///
/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Logger#warn)
pub struct LoggerWarnOptions {
  /// Whether this is a deprecation warning.
  pub deprecation: bool,
  /// The location in the Sass source code that generated this warning.
  pub span: Option<SourceSpan>,
  /// The Sass stack trace at the point the warning was issued.
  pub stack: Option<String>,
  pub(crate) formatted: String,
}

/// Options for [Logger::debug].
///
/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Logger#debug)
pub struct LoggerDebugOptions {
  /// The location in the Sass source code that generated this debug message.
  pub span: Option<SourceSpan>,
  pub(crate) formatted: String,
}

/// Enum wrapper for [BoxedImporter] and [BoxedFileImporter].
#[derive(Debug)]
pub enum SassImporter {
  /// A [BoxedImporter].
  Importer(BoxedImporter),
  /// A [BoxedFileImporter].
  FileImporter(BoxedFileImporter),
}

/// A type alias for [Box<dyn Importer>].
pub type BoxedImporter = Box<dyn Importer>;

/// A type alias for [Box<dyn FileImporter>].
pub type BoxedFileImporter = Box<dyn FileImporter>;

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Importer)
pub trait Importer: Debug + Send + Sync {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Importer#canonicalize)
  fn canonicalize(
    &self,
    url: &str,
    options: &ImporterOptions,
  ) -> Result<Option<Url>>;

  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Importer#load)
  fn load(&self, canonical_url: &Url) -> Result<Option<ImporterResult>>;
}

/// Options for [Importer::canonicalize] or [Importer::load].
///
/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/Importer#canonicalize)
pub struct ImporterOptions {
  /// Whether this is being invoked because of a Sass @import rule, as opposed to a @use
  /// or @forward rule.
  pub from_import: bool,
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/FileImporter)
pub trait FileImporter: Debug + Send + Sync {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/FileImporter#findFileUrl)
  fn find_file_url(
    &self,
    url: &str,
    options: &ImporterOptions,
  ) -> Result<Option<Url>>;
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/ImporterResult)
pub struct ImporterResult {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/ImporterResult#contents)
  pub contents: String,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/ImporterResult#sourceMapUrl)
  pub source_map_url: Option<Url>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/ImporterResult#syntax)
  pub syntax: Syntax,
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/CompileResult)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CompileResult {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/CompileResult#css)
  pub css: String,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/CompileResult#loadedUrls)
  pub loaded_urls: Vec<Url>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/CompileResult#sourceMap)
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
      loaded_urls: s
        .loaded_urls
        .iter()
        .map(|url| Url::parse(url).unwrap())
        .collect(),
      source_map: if s.source_map.is_empty() {
        None
      } else {
        Some(s.source_map)
      },
    }
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/modules#OutputStyle)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum OutputStyle {
  /// Writes each selector and declaration on its own line.
  Expanded,
  /// Removes as many extra characters as possible, and writes the entire stylesheet on a single line.
  Compressed,
}

impl Default for OutputStyle {
  fn default() -> Self {
    Self::Expanded
  }
}

impl From<OutputStyle> for protocol::OutputStyle {
  fn from(o: OutputStyle) -> Self {
    match o {
      OutputStyle::Expanded => Self::Expanded,
      OutputStyle::Compressed => Self::Compressed,
    }
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/modules#Syntax)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum Syntax {
  /// the [scss syntax](https://sass-lang.com/documentation/syntax#scss)
  Scss,
  /// the [indented syntax](https://sass-lang.com/documentation/syntax#the-indented-syntax)
  Indented,
  /// the plain css syntax, which is parsed like SCSS but forbids the use of any special Sass features.
  Css,
}

impl Default for Syntax {
  fn default() -> Self {
    Self::Scss
  }
}

impl From<Syntax> for protocol::Syntax {
  fn from(s: Syntax) -> Self {
    match s {
      Syntax::Scss => Self::Scss,
      Syntax::Indented => Self::Indented,
      Syntax::Css => Self::Css,
    }
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/SourceSpan)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct SourceSpan {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/SourceSpan#context)
  pub context: Option<String>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/SourceSpan#end)
  pub end: SourceLocation,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/SourceSpan#start)
  pub start: SourceLocation,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/SourceSpan#url)
  pub url: Option<Url>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/SourceSpan#text)
  pub text: String,
}

impl From<protocol::SourceSpan> for SourceSpan {
  fn from(span: protocol::SourceSpan) -> Self {
    let start = span.start.unwrap();
    Self {
      context: if span.context.is_empty() {
        None
      } else {
        Some(span.context)
      },
      end: span.end.unwrap_or_else(|| start.clone()).into(),
      start: start.into(),
      url: if span.url.is_empty() {
        None
      } else {
        Some(Url::parse(&span.url).unwrap())
      },
      text: span.text,
    }
  }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct SourceLocation {
  pub offset: usize,
  pub line: usize,
  pub column: usize,
}

impl From<protocol::source_span::SourceLocation> for SourceLocation {
  fn from(location: protocol::source_span::SourceLocation) -> Self {
    Self {
      offset: location.offset as usize,
      line: location.line as usize,
      column: location.column as usize,
    }
  }
}
