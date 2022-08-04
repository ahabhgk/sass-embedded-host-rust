use std::{
  env, fs,
  path::PathBuf,
  time::{Duration, SystemTime},
};

use url::Url;

use crate::{
  legacy::url_to_file_path_cross_platform, CompileResult, Options,
  StringOptions, Syntax,
};
pub use crate::{OutputStyle, SassLogger};

use super::{
  LegacyImporter, SassLegacyImporter, END_OF_LOAD_PROTOCOL,
  LEGACY_IMPORTER_PROTOCOL,
};

#[cfg(target_family = "windows")]
const PATH_DELIMITER: &str = ";";
#[cfg(target_family = "unix")]
const PATH_DELIMITER: &str = ":";

#[derive(Debug, Clone)]
pub enum IndentType {
  Tab,
  Space,
}

impl Default for IndentType {
  fn default() -> Self {
    Self::Space
  }
}

#[derive(Debug, Clone)]
pub enum LineFeed {
  CR,
  CRLF,
  LF,
  LFCR,
}

impl Default for LineFeed {
  fn default() -> Self {
    Self::LF
  }
}

#[derive(Debug, Clone)]
pub struct LegacyPluginThisOptionsResult {
  pub stats: LegacyPluginThisOptionsResultStats,
}

#[derive(Debug, Clone)]
pub struct LegacyPluginThisOptionsResultStats {
  pub start: SystemTime,
  pub entry: String,
}

#[derive(Debug, Clone)]
pub struct LegacyPluginThisOptions {
  pub file: Option<String>,
  pub data: Option<String>,
  pub include_paths: String,
  pub precision: u8,
  pub style: u8,
  pub indent_type: IndentType,
  pub indent_width: usize,
  pub linefeed: LineFeed,
  pub result: LegacyPluginThisOptionsResult,
}

#[derive(Debug, Clone)]
pub struct LegacyPluginThis {
  pub options: LegacyPluginThisOptions,
}

impl LegacyPluginThis {
  pub fn new(options: &LegacyOptions) -> Self {
    let mut include_paths =
      vec![env::current_dir().unwrap().to_string_lossy().to_string()];
    include_paths.extend(options.include_paths.clone());
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
              .clone()
              .unwrap_or_else(|| options.data.clone().unwrap()),
          },
        },
      },
    }
  }
}

pub struct LegacyImporterThis {
  pub options: LegacyPluginThisOptions,
  pub from_import: bool,
}

pub enum LegacyImporterResult {
  File(PathBuf),
  Contents(String),
}

#[derive(Debug, Default)]
pub struct LegacyOptionsBuilder {
  options: LegacyOptions,
}

impl LegacyOptionsBuilder {
  pub fn build(self) -> LegacyOptions {
    if self.options.data.is_none() && self.options.file.is_none() {
      panic!("Either options.data or options.file must be set.");
    }
    self.options
  }

  pub fn include_paths(
    mut self,
    arg: impl IntoIterator<Item = String>,
  ) -> Self {
    self.options.include_paths = arg.into_iter().collect();
    self
  }

  pub fn include_path(mut self, arg: impl Into<String>) -> Self {
    self.options.include_paths.push(arg.into());
    self
  }

  pub fn indent_type(mut self, arg: impl Into<IndentType>) -> Self {
    self.options.indent_type = arg.into();
    self
  }

  pub fn indent_width(mut self, arg: impl Into<usize>) -> Self {
    self.options.indent_width = arg.into();
    self
  }

  pub fn linefeed(mut self, arg: impl Into<LineFeed>) -> Self {
    self.options.linefeed = arg.into();
    self
  }

  pub fn output_style(mut self, arg: impl Into<OutputStyle>) -> Self {
    self.options.output_style = arg.into();
    self
  }

  pub fn source_map(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map = arg.into();
    self
  }

  pub fn source_map_contents(mut self, arg: impl Into<bool>) -> Self {
    self.options.source_map_contents = arg.into();
    self
  }

  pub fn sass_importers(
    mut self,
    arg: impl IntoIterator<Item = SassLegacyImporter>,
  ) -> Self {
    self.options.importers = Some(arg.into_iter().collect());
    self
  }

  pub fn sass_importer(mut self, arg: impl Into<SassLegacyImporter>) -> Self {
    self.options.importers =
      Some(if let Some(mut importers) = self.options.importers {
        importers.push(arg.into());
        importers
      } else {
        vec![arg.into()]
      });
    self
  }

  pub fn importers(
    self,
    arg: impl IntoIterator<Item = Box<dyn LegacyImporter>>,
  ) -> Self {
    self.sass_importers(arg)
  }

  pub fn importer(self, arg: impl Into<Box<dyn LegacyImporter>>) -> Self {
    self.sass_importer(arg)
  }

  pub fn charset(mut self, arg: impl Into<bool>) -> Self {
    self.options.charset = arg.into();
    self
  }

  pub fn quiet_deps(mut self, arg: impl Into<bool>) -> Self {
    self.options.quiet_deps = arg.into();
    self
  }

  pub fn verbose(mut self, arg: impl Into<bool>) -> Self {
    self.options.verbose = arg.into();
    self
  }

  pub fn logger(mut self, arg: impl Into<SassLogger>) -> Self {
    self.options.logger = Some(arg.into());
    self
  }

  pub fn file(mut self, arg: impl Into<String>) -> Self {
    self.options.file = Some(arg.into());
    self
  }

  pub fn data(mut self, arg: impl Into<String>) -> Self {
    self.options.data = Some(arg.into());
    self
  }

  pub fn indented_syntax(mut self, arg: impl Into<bool>) -> Self {
    self.options.indented_syntax = Some(arg.into());
    self
  }
}

#[derive(Debug)]
pub struct LegacyOptions {
  pub include_paths: Vec<String>,
  pub indent_type: IndentType,
  pub indent_width: usize,
  pub linefeed: LineFeed,
  pub output_style: OutputStyle,
  pub source_map: bool,
  pub source_map_contents: bool,
  pub importers: Option<Vec<SassLegacyImporter>>,
  pub charset: bool,
  pub quiet_deps: bool,
  pub verbose: bool,
  pub logger: Option<SassLogger>,
  pub file: Option<String>,
  pub data: Option<String>,
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
  pub fn adjust_options(mut self) -> Self {
    if let Some(file) = &self.file {
      if self.indented_syntax.is_some() || self.importers.is_some() {
        self.data = Some(fs::read_to_string(file).unwrap());
        self.indented_syntax = Some(self.indented_syntax.unwrap_or_default());
      }
    }
    self
  }
}

#[derive(Debug, Clone)]
pub struct LegacyResult {
  pub css: Vec<u8>,
  pub map: Option<Vec<u8>>,
  pub stats: LegacyResultStats,
}

#[derive(Debug, Clone)]
pub struct LegacyResultStats {
  pub entry: String,
  pub start: SystemTime,
  pub end: SystemTime,
  pub duration: Duration,
  pub included_files: Vec<String>,
}

impl LegacyResult {
  pub fn new(
    entry: Option<String>,
    start: SystemTime,
    result: CompileResult,
  ) -> Self {
    let end = SystemTime::now();
    Self {
      css: result.css.into_bytes(),
      map: result.source_map.map(|map| map.into_bytes()),
      stats: LegacyResultStats {
        entry: entry.unwrap_or_else(|| "data".to_string()),
        start,
        end,
        duration: end.duration_since(start).unwrap(),
        included_files: result
          .loaded_urls
          .into_iter()
          .map(|url| Url::parse(&url).unwrap())
          .filter(|url| format!("{}:", url.scheme()) != END_OF_LOAD_PROTOCOL)
          .map(|url| {
            if url.scheme() == "file" {
              url_to_file_path_cross_platform(&url)
                .to_string_lossy()
                .to_string()
            } else if format!("{}:", url.scheme()) == LEGACY_IMPORTER_PROTOCOL {
              url.path().to_string()
            } else {
              url.to_string()
            }
          })
          .collect(),
      },
    }
  }
}
