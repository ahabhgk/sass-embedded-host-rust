use std::{env, path::PathBuf, time::SystemTime};

use crate::{options::SassImporter, Importer, Options, OptionsBuilder};
pub use crate::{OutputStyle, SassLogger};

use super::{LegacyImporterWrapper, SassLegacyImporter};

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
pub enum SourceMap {
  Boolean(bool),
  String(String),
}

impl Default for SourceMap {
  fn default() -> Self {
    Self::Boolean(false)
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

impl LegacyOptionsBuilder {}

#[derive(Debug)]
pub struct LegacyOptions {
  pub include_paths: Vec<String>,
  pub indent_type: IndentType,
  pub indent_width: usize,
  pub linefeed: LineFeed,
  pub omit_source_map_url: bool,
  pub out_file: Option<String>,
  pub output_style: OutputStyle,
  pub source_map: SourceMap,
  pub source_map_contents: bool,
  pub source_map_embed: bool,
  pub source_map_root: String,
  pub importer: Vec<SassLegacyImporter>,
  pub charset: bool,
  pub quiet_deps: bool,
  pub verbose: bool,
  pub logger: Option<SassLogger>,
  pub file: Option<String>,
  pub data: Option<String>,
  pub indented_syntax: bool,
}

impl Default for LegacyOptions {
  fn default() -> Self {
    Self {
      indent_width: 2,
      charset: true,
      include_paths: Vec::new(),
      indent_type: IndentType::Space,
      linefeed: LineFeed::LF,
      omit_source_map_url: false,
      out_file: None,
      output_style: OutputStyle::Expanded,
      source_map: SourceMap::Boolean(false),
      source_map_contents: false,
      source_map_embed: false,
      source_map_root: String::new(),
      importer: Vec::new(),
      quiet_deps: false,
      verbose: false,
      logger: None,
      file: None,
      data: None,
      indented_syntax: false,
    }
  }
}

impl From<LegacyOptions> for Options {
  fn from(options: LegacyOptions) -> Self {
    let this = LegacyPluginThis::new(&options);
    let source_map = was_source_map_requested(&options);
    let load_paths = if options.importer.is_empty() {
      None
    } else {
      Some(options.include_paths.clone())
    };
    let importer = LegacyImporterWrapper::new(
      this,
      options.importer,
      options.include_paths,
      options.file.unwrap_or_else(|| "stdin".to_string()),
    );
    Self {
      importers: Some(vec![SassImporter::Importer(Box::new(importer))]),
      load_paths,
      logger: options.logger,
      quiet_deps: options.quiet_deps,
      source_map,
      source_map_include_sources: options.source_map_contents,
      style: options.output_style,
      verbose: options.verbose,
      charset: options.charset,
      ..Default::default()
    }
  }
}

impl LegacyOptions {
  pub fn is_string_options(&self) -> bool {
    self.data.is_some()
  }
}

fn was_source_map_requested(options: &LegacyOptions) -> bool {
  match &options.source_map {
    SourceMap::Boolean(s) => *s && options.out_file.is_some(),
    SourceMap::String(_) => true,
  }
}
