use std::{
  env, fs,
  path::{Path, PathBuf},
  time::{Duration, SystemTime},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use urlencoding::decode;

use crate::{
  legacy::url_to_file_path_cross_platform, CompileResult, Options,
  StringOptions, Syntax, Url,
};
pub use crate::{BoxLogger, Logger, OutputStyle};

use super::{
  BoxLegacyImporter, LegacyImporter, END_OF_LOAD_PROTOCOL,
  LEGACY_IMPORTER_PROTOCOL,
};

/// The platform-specific file delimiter, ';' for windows.
#[cfg(target_family = "windows")]
pub const PATH_DELIMITER: &str = ";";
/// The platform-specific file delimiter, ':' for unix.
#[cfg(target_family = "unix")]
pub const PATH_DELIMITER: &str = ":";

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyFileOptions#indentType)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndentType {
  /// Space IndentType.
  Space,
  /// Tab IndentType.
  Tab,
}

impl Default for IndentType {
  fn default() -> Self {
    Self::Space
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyFileOptions#linefeed)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineFeed {
  /// 'cr' uses U+000D CARRIAGE RETURN.
  CR,
  /// 'crlf' uses U+000D CARRIAGE RETURN followed by U+000A LINE FEED.
  CRLF,
  /// 'lf' uses U+000A LINE FEED.
  LF,
  /// 'lfcr' uses U+000A LINE FEED followed by U+000D CARRIAGE RETURN.
  LFCR,
}

impl Default for LineFeed {
  fn default() -> Self {
    Self::LF
  }
}

/// A partially-constructed [LegacyResult] object.
#[derive(Debug, Clone)]
pub struct LegacyPluginThisOptionsResult {
  /// Partial information about the compilation in progress.
  pub stats: LegacyPluginThisOptionsResultStats,
}

/// Partial information about the compilation in progress.
#[derive(Debug, Clone)]
pub struct LegacyPluginThisOptionsResultStats {
  /// The number of milliseconds between 1 January 1970 at 00:00:00 UTC and
  /// the time at which Sass compilation began.
  pub start: SystemTime,
  /// [LegacyOptions.file] if it was passed, otherwise the string `"data"`.
  pub entry: String,
}

/// A partial representation of the options passed to [Sass::render].
#[derive(Debug, Clone)]
pub struct LegacyPluginThisOptions {
  /// The value passed to [LegacyOptions.file].
  pub file: Option<PathBuf>,
  /// The value passed to [LegacyOptions.data].
  pub data: Option<String>,
  /// The value passed to [LegacyOptions.include_paths] separated by
  /// `";"` on Windows or `":"` on other operating systems. This always
  /// includes the current working directory as the first entry.
  pub include_paths: String,
  /// Always the number 10.
  pub precision: u8,
  /// Always the number 1.
  pub style: u8,
  /// The value passed to [LegacyOptions.indent_type], [IndentType::Space] otherwise.
  pub indent_type: IndentType,
  /// The value passed to [LegacyOptions.indent_width], or `2` otherwise.
  pub indent_width: usize,
  /// The value passed to [LegacyOptions.linefeed], or `"\n"` otherwise.
  pub linefeed: LineFeed,
  /// A partially-constructed [LegacyResult] object.
  pub result: LegacyPluginThisOptionsResult,
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyPluginThis)
#[derive(Debug, Clone)]
pub struct LegacyPluginThis {
  /// A partial representation of the options passed to [Sass::render].
  pub options: LegacyPluginThisOptions,
}

impl LegacyPluginThis {
  /// Creates a new [LegacyPluginThis].
  pub fn new(options: &LegacyOptions) -> Self {
    let mut include_paths =
      vec![env::current_dir().unwrap().to_string_lossy().to_string()];
    include_paths.extend(
      options
        .include_paths
        .iter()
        .map(|p| p.to_str().unwrap().to_string()),
    );
    Self {
      options: LegacyPluginThisOptions {
        file: options.file.clone(),
        data: options.data.clone(),
        include_paths: include_paths.join(PATH_DELIMITER),
        precision: 10,
        style: 1,
        indent_type: IndentType::Space,
        indent_width: 2,
        linefeed: LineFeed::LF,
        result: LegacyPluginThisOptionsResult {
          stats: LegacyPluginThisOptionsResultStats {
            start: SystemTime::now(),
            entry: options
              .file
              .as_ref()
              .map(|file| file.to_str().unwrap().to_string())
              .unwrap_or_else(|| "data".to_owned()),
          },
        },
      },
    }
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyImporterThis)
pub struct LegacyImporterThis {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyImporterThis#options)
  pub options: LegacyPluginThisOptions,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyImporterThis#fromImporter)
  pub from_import: bool,
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/modules#LegacyImporterResult)
pub enum LegacyImporterResult {
  /// An object with the key file whose value is a path on disk. This causes
  /// Sass to load that file as though it had been imported directly.
  File(PathBuf),
  /// An object with the key contents whose value is the contents of a stylesheet
  /// (in SCSS syntax). This causes Sass to load that stylesheet’s contents.
  Contents {
    /// The contents of the stylesheet.
    contents: String,
    /// The path to the stylesheet.
    file: Option<PathBuf>,
  },
}

impl LegacyImporterResult {
  /// Creates a new [LegacyImporterResult] from a [PathBuf].
  pub fn file(path: impl Into<PathBuf>) -> Self {
    Self::File(path.into())
  }

  /// Creates a new [LegacyImporterResult] from contents [String].
  pub fn contents(contents: impl Into<String>) -> Self {
    Self::Contents {
      contents: contents.into(),
      file: None,
    }
  }

  /// Creates a new [LegacyImporterResult] from contents [String] and a [PathBuf].
  pub fn both(contents: impl Into<String>, file: impl Into<PathBuf>) -> Self {
    Self::Contents {
      contents: contents.into(),
      file: Some(file.into()),
    }
  }
}

/// A builder for [LegacyOptionsBuilder].
#[derive(Debug, Default)]
pub struct LegacyOptionsBuilder {
  options: LegacyOptions,
}

impl LegacyOptionsBuilder {
  /// Creates a new [LegacyOptionsBuilder].
  pub fn new() -> Self {
    Self::default()
  }

  /// Builds a [LegacyOptions].
  pub fn build(self) -> LegacyOptions {
    if self.options.data.is_none() && self.options.file.is_none() {
      panic!("Either options.data or options.file must be set.");
    }
    self.options
  }

  /// Sets the [LegacyOptions]'s [include_paths] field.
  pub fn include_paths(mut self, arg: &[impl AsRef<Path>]) -> Self {
    self.options.include_paths =
      arg.iter().map(|p| p.as_ref().to_owned()).collect();
    self
  }

  /// Adds a path to the [LegacyOptions]'s [include_paths] field.
  pub fn include_path(mut self, arg: impl AsRef<Path>) -> Self {
    self.options.include_paths.push(arg.as_ref().to_owned());
    self
  }

  /// Sets the [LegacyOptions]'s [indent_type] field.
  pub fn indent_type(mut self, arg: impl Into<IndentType>) -> Self {
    self.options.indent_type = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [indent_width] field.
  pub fn indent_width(mut self, arg: impl Into<usize>) -> Self {
    self.options.indent_width = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [linefeed] field.
  pub fn linefeed(mut self, arg: impl Into<LineFeed>) -> Self {
    self.options.linefeed = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [output_style] field.
  pub fn output_style(mut self, arg: impl Into<OutputStyle>) -> Self {
    self.options.output_style = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [source_map] field.
  pub fn source_map(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [source_map_contents] field.
  pub fn source_map_contents(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map_contents = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [sass_importers] field with [SassLegacyImporter]s.
  pub fn sass_importers(
    mut self,
    arg: impl IntoIterator<Item = impl Into<BoxLegacyImporter>>,
  ) -> Self {
    self.options.importers = Some(arg.into_iter().map(|i| i.into()).collect());
    self
  }

  /// Adds a [SassLegacyImporter] to the [LegacyOptions]'s [sass_importers] field.
  pub fn sass_importer(mut self, arg: impl Into<BoxLegacyImporter>) -> Self {
    self.options.importers =
      Some(if let Some(mut importers) = self.options.importers {
        importers.push(arg.into());
        importers
      } else {
        vec![arg.into()]
      });
    self
  }

  /// Sets the [LegacyOptions]'s [sass_importers] field with [LegacyImporter]s.
  pub fn importers(
    self,
    arg: impl IntoIterator<Item = impl Into<Box<dyn LegacyImporter>>>,
  ) -> Self {
    self.sass_importers(arg)
  }

  /// Adds a [LegacyImporter] to the [LegacyOptions]'s [sass_importers] field.
  pub fn importer<I: 'static + LegacyImporter>(self, arg: I) -> Self {
    self.sass_importer(arg)
  }

  /// Sets the [LegacyOptions]'s [charset] field.
  pub fn charset(mut self, arg: impl Into<bool>) -> Self {
    self.options.charset = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [quiet_deps] field.
  pub fn quiet_deps(mut self, arg: impl Into<bool>) -> Self {
    self.options.quiet_deps = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [verbose] field.
  pub fn verbose(mut self, arg: impl Into<bool>) -> Self {
    self.options.verbose = arg.into();
    self
  }

  /// Sets the [LegacyOptions]'s [logger] field.
  pub fn logger<L: 'static + Logger>(mut self, arg: L) -> Self {
    self.options.logger = Some(Box::new(arg));
    self
  }

  /// Sets the [LegacyOptions]'s [file] field.
  pub fn file(mut self, arg: impl AsRef<Path>) -> Self {
    self.options.file = Some(arg.as_ref().to_owned());
    self
  }

  /// Sets the [LegacyOptions]'s [data] field.
  pub fn data(mut self, arg: impl Into<String>) -> Self {
    self.options.data = Some(arg.into());
    self
  }

  /// Sets the [LegacyOptions]'s [indented_syntax] field.
  pub fn indented_syntax(mut self, arg: impl Into<bool>) -> Self {
    self.options.indented_syntax = Some(arg.into());
    self
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyImporterThis)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct LegacyOptions {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#includePaths)
  pub include_paths: Vec<PathBuf>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#indentType)
  pub indent_type: IndentType,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#indentWidth)
  pub indent_width: usize,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#linefeed)
  pub linefeed: LineFeed,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#outputStyle)
  pub output_style: OutputStyle,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#sourceMap)
  pub source_map: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#sourceMapContents)
  pub source_map_contents: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#importer)
  #[cfg_attr(feature = "serde", serde(skip))]
  pub importers: Option<Vec<BoxLegacyImporter>>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#charset)
  pub charset: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#quietDeps)
  pub quiet_deps: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#verbose)
  pub verbose: bool,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacySharedOptions#logger)
  #[cfg_attr(feature = "serde", serde(skip))]
  pub logger: Option<BoxLogger>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyFileOptions#file)
  pub file: Option<PathBuf>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyStringOptions#data)
  pub data: Option<String>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyStringOptions#indentedSyntax)
  pub indented_syntax: Option<bool>,
}

impl Default for LegacyOptions {
  fn default() -> Self {
    Self {
      indent_width: 2,
      charset: true,
      include_paths: Vec::new(),
      indent_type: IndentType::Space,
      linefeed: LineFeed::LF,
      output_style: OutputStyle::Expanded,
      source_map: false,
      source_map_contents: false,
      importers: None,
      quiet_deps: false,
      verbose: false,
      logger: None,
      file: None,
      data: None,
      indented_syntax: None,
    }
  }
}

impl From<LegacyOptions> for Options {
  fn from(options: LegacyOptions) -> Self {
    Self {
      load_paths: options.include_paths,
      logger: options.logger,
      quiet_deps: options.quiet_deps,
      source_map: options.source_map,
      source_map_include_sources: options.source_map_contents,
      style: options.output_style,
      verbose: options.verbose,
      charset: options.charset,
      // importers: set manually after into modern options
      ..Default::default()
    }
  }
}

impl From<LegacyOptions> for StringOptions {
  fn from(options: LegacyOptions) -> Self {
    let url = options
      .file
      .clone()
      .map(|file| Url::from_file_path(file).unwrap())
      .unwrap_or_else(|| Url::parse(LEGACY_IMPORTER_PROTOCOL).unwrap());
    let syntax = options
      .indented_syntax
      .map(|s| if s { Syntax::Indented } else { Syntax::Scss })
      .unwrap_or_default();
    let options = Options::from(options);
    Self {
      common: options,
      input_importer: None,
      syntax,
      url: Some(url),
    }
  }
}

impl LegacyOptions {
  pub(crate) fn adjust_options(mut self) -> Self {
    if let Some(file) = &self.file {
      if self.data.is_none()
        && (self.indented_syntax.is_some() || self.importers.is_some())
      {
        self.data = Some(fs::read_to_string(file).unwrap());
        self.indented_syntax = Some(self.indented_syntax.unwrap_or_default());
      }
    }
    self
  }
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyResult)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct LegacyResult {
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyResult#css)
  pub css: Vec<u8>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyResult#map)
  pub map: Option<Vec<u8>>,
  /// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyResult#stats)
  pub stats: LegacyResultStats,
}

/// More information: [Sass documentation](https://sass-lang.com/documentation/js-api/interfaces/LegacyResult#stats)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct LegacyResultStats {
  /// The absolute path of [LegacyOptions.file], or "data" if [LegacyOptions.file]
  /// wasn't set.
  pub entry: String,
  /// The number of milliseconds between 1 January 1970 at 00:00:00 UTC and the time
  /// at which Sass compilation began.
  pub start: SystemTime,
  /// The number of milliseconds between 1 January 1970 at 00:00:00 UTC and the time
  /// at which Sass compilation ended.
  pub end: SystemTime,
  /// The number of milliseconds it took to compile the Sass file. This is always equal
  /// to start minus end.
  pub duration: Duration,
  /// An array of the absolute paths of all Sass files loaded during compilation. If
  /// a stylesheet was loaded from a LegacyImporter that returned the stylesheet’s
  /// contents, the raw string of the @use or @import that loaded that stylesheet
  /// included in this array.
  pub included_files: Vec<String>,
}

impl LegacyResult {
  /// Creates a new [LegacyResult].
  pub fn new(entry: String, start: SystemTime, result: CompileResult) -> Self {
    let end = SystemTime::now();
    Self {
      css: result.css.into_bytes(),
      map: result.source_map.map(|map| map.into_bytes()),
      stats: LegacyResultStats {
        entry,
        start,
        end,
        duration: end.duration_since(start).unwrap(),
        included_files: result
          .loaded_urls
          .into_iter()
          .filter(|url| format!("{}:", url.scheme()) != END_OF_LOAD_PROTOCOL)
          .map(|url| {
            if url.scheme() == "file" {
              url_to_file_path_cross_platform(&url)
                .to_string_lossy()
                .to_string()
            } else if format!("{}:", url.scheme()) == LEGACY_IMPORTER_PROTOCOL {
              decode(url.path()).unwrap().to_string()
            } else {
              url.to_string()
            }
          })
          .collect(),
      },
    }
  }
}
