use std::{
  collections::VecDeque,
  fmt::Debug,
  fs,
  path::{Path, PathBuf},
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::Mutex;
use regex::Regex;

use crate::{
  Exception, Importer, ImporterOptions, ImporterResult, Result, Syntax, Url,
};

use super::{LegacyImporterResult, LegacyImporterThis, LegacyPluginThis};

pub const END_OF_LOAD_PROTOCOL: &str = "sass-embedded-legacy-load-done:";
pub const LEGACY_IMPORTER_PROTOCOL: &str = "legacy-importer:";

pub trait LegacyImporter: Debug + Sync + Send {
  fn call(
    &self,
    this: &LegacyImporterThis,
    url: &str,
    prev: &str,
  ) -> Result<Option<LegacyImporterResult>>;
}

pub type SassLegacyImporter = Box<dyn LegacyImporter>;

#[derive(Debug)]
pub struct LegacyImporterWrapper {
  prev_stack: Mutex<Vec<PreviousUrl>>,
  last_contents: Mutex<Option<String>>,
  expecting_relative_load: Mutex<bool>,
  callbacks: Vec<SassLegacyImporter>,
  this: LegacyPluginThis,
  load_paths: Vec<String>,
}

impl LegacyImporterWrapper {
  pub fn new(
    this: LegacyPluginThis,
    callbacks: Vec<SassLegacyImporter>,
    load_paths: Vec<String>,
    initial_prev: String,
  ) -> Arc<Self> {
    let path = initial_prev != "stdin";
    Arc::new(Self {
      prev_stack: Mutex::new(vec![PreviousUrl {
        url: if path {
          initial_prev
        } else {
          "stdin".to_string()
        },
        path,
      }]),
      last_contents: Mutex::new(None),
      expecting_relative_load: Mutex::new(true),
      callbacks,
      this,
      load_paths,
    })
  }

  fn invoke_callbacks(
    &self,
    url: &str,
    prev: &str,
    options: &ImporterOptions,
  ) -> Result<Option<LegacyImporterResult>> {
    assert!(!self.callbacks.is_empty());

    let this = LegacyImporterThis {
      options: self.this.options.clone(),
      from_import: options.from_import,
    };
    for callback in &self.callbacks {
      match callback.call(&this, url, prev) {
        Ok(Some(result)) => return Ok(Some(result)),
        Ok(None) => continue,
        Err(e) => return Err(e),
      }
    }
    Ok(None)
  }
}

impl Importer for Arc<LegacyImporterWrapper> {
  fn canonicalize(
    &self,
    url: &str,
    options: &ImporterOptions,
  ) -> Result<Option<url::Url>> {
    if url.starts_with(END_OF_LOAD_PROTOCOL) {
      return Ok(Some(Url::parse(url).unwrap()));
    }

    let mut prev_stack = self.prev_stack.lock();

    let mut expecting_relative_load = self.expecting_relative_load.lock();
    if *expecting_relative_load {
      if url.starts_with("file:") {
        let path = url_to_file_path_cross_platform(&Url::parse(url).unwrap());
        let resolved = resolve_path(path, options.from_import)?;
        if let Some(p) = resolved {
          prev_stack.push(PreviousUrl {
            url: p.to_string_lossy().to_string(),
            path: true,
          });
          return Ok(Some(Url::from_file_path(p).unwrap()));
        }
      }
      *expecting_relative_load = false;
    } else {
      *expecting_relative_load = true;
    }

    let prev = prev_stack.last().unwrap();
    let result = match self.invoke_callbacks(url, &prev.url, options) {
      Err(e) => Err(e),
      Ok(None) => Ok(None),
      Ok(Some(result)) => match result {
        LegacyImporterResult::Contents(contents) => {
          *self.last_contents.lock() = Some(contents);
          Ok(Some(
            if Regex::new("^[A-Za-z+.-]+:").unwrap().is_match(url) {
              Url::parse(url).unwrap()
            } else {
              Url::parse(&format!("{}{}", LEGACY_IMPORTER_PROTOCOL, url))
                .unwrap()
            },
          ))
        }
        LegacyImporterResult::File(file) => {
          if file.is_absolute() {
            let resolved = resolve_path(file, options.from_import)?;
            return Ok(resolved.map(|p| Url::from_file_path(p).unwrap()));
          }
          let mut prefixes = VecDeque::from(self.load_paths.clone());
          prefixes.push_back(".".to_owned());
          if prev.path {
            prefixes.push_front(
              Path::new(&prev.url)
                .parent()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            );
          }
          for prefix in prefixes {
            let resolved = resolve_path(
              Path::new(&prefix).join(file.clone()),
              options.from_import,
            )?;
            if let Some(p) = resolved {
              return Ok(Some(Url::from_file_path(p).unwrap()));
            }
          }
          Ok(None)
        }
      },
    }?;
    if let Some(result) = &result {
      let path = result.scheme() == "file";
      prev_stack.push(PreviousUrl {
        url: if path {
          url_to_file_path_cross_platform(result)
            .to_string_lossy()
            .to_string()
        } else {
          url.to_string()
        },
        path,
      });
    } else {
      for load_path in &self.load_paths {
        let resolved =
          resolve_path(Path::new(&load_path).join(url), options.from_import)?;
        if let Some(p) = resolved {
          return Ok(Some(Url::from_file_path(p).unwrap()));
        }
      }
    }
    Ok(result)
  }

  fn load(&self, canonical_url: &Url) -> Result<Option<ImporterResult>> {
    let protocol = format!("{}:", canonical_url.scheme());
    if protocol == END_OF_LOAD_PROTOCOL {
      self.prev_stack.lock().pop();
      return Ok(Some(ImporterResult {
        contents: String::new(),
        source_map_url: Some(END_OF_LOAD_PROTOCOL.to_string()),
        syntax: Syntax::Scss,
      }));
    }
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_micros();
    if protocol == "file:" {
      let syntax = if canonical_url.path().ends_with(".sass") {
        Syntax::Indented
      } else if canonical_url.path().ends_with(".css") {
        Syntax::Css
      } else {
        Syntax::Scss
      };
      let mut last_contents = self.last_contents.lock();
      let contents = last_contents.clone().unwrap_or_else(|| {
        fs::read_to_string(url_to_file_path_cross_platform(canonical_url))
          .unwrap()
      });
      *last_contents = None;
      let contents = match syntax {
        Syntax::Scss => {
          format!("{contents}\n;@import \"{END_OF_LOAD_PROTOCOL}{timestamp}\"")
        }
        Syntax::Indented => {
          format!("{contents}\n@import \"{END_OF_LOAD_PROTOCOL}{timestamp}\"")
        }
        Syntax::Css => {
          self.prev_stack.lock().pop();
          contents
        }
      };
      return Ok(Some(ImporterResult {
        contents,
        syntax,
        source_map_url: Some(canonical_url.to_string()),
      }));
    }
    let mut last_contents = self.last_contents.lock();
    assert!(last_contents.is_some());
    let contents = format!(
      "{}\n;@import \"{END_OF_LOAD_PROTOCOL}{timestamp}\"",
      last_contents.clone().unwrap()
    );
    *last_contents = None;
    Ok(Some(ImporterResult {
      contents,
      syntax: Syntax::Scss,
      source_map_url: Some(canonical_url.to_string()),
    }))
  }
}

#[derive(Debug)]
struct PreviousUrl {
  url: String,
  path: bool,
}

pub fn url_to_file_path_cross_platform(file_url: &Url) -> PathBuf {
  let p = file_url
    .to_file_path()
    .unwrap()
    .to_string_lossy()
    .to_string();
  if Regex::new("^/[A-Za-z]:/").unwrap().is_match(&p) {
    PathBuf::from(&p[1..])
  } else {
    PathBuf::from(p)
  }
}

fn resolve_path(path: PathBuf, from_import: bool) -> Result<Option<PathBuf>> {
  let extension = path.extension();
  if let Some(extension) = extension {
    if extension == "sass" || extension == "scss" || extension == "css" {
      if from_import {
        if let Ok(Some(p)) = exactly_one(try_path(Path::new(&format!(
          "{}.import.{}",
          path.file_stem().unwrap().to_string_lossy(),
          extension.to_string_lossy()
        )))) {
          return Ok(Some(p));
        }
      }
      return exactly_one(try_path(&path));
    }
  }
  if from_import {
    if let Ok(Some(p)) = exactly_one(try_path_with_extensions(Path::new(
      &format!("{}.import", path.file_stem().unwrap().to_string_lossy()),
    ))) {
      return Ok(Some(p));
    }
  }
  if let Ok(Some(p)) = exactly_one(try_path_with_extensions(&path)) {
    return Ok(Some(p));
  }
  try_path_as_directory(&path.join("index"), from_import)
}

fn exactly_one(paths: Vec<PathBuf>) -> Result<Option<PathBuf>> {
  if paths.is_empty() {
    Ok(None)
  } else if paths.len() == 1 {
    Ok(Some(paths[0].clone()))
  } else {
    Err(Exception::new(format!(
      "It's not clear which file to import. Found:\n{}",
      paths
        .iter()
        .map(|p| format!("  {}", p.to_string_lossy()))
        .collect::<Vec<String>>()
        .join("\n")
    )))
  }
}

fn dir_exists(path: &Path) -> bool {
  path.exists() && path.is_dir()
}

fn file_exists(path: &Path) -> bool {
  path.exists() && path.is_file()
}

fn try_path_as_directory(
  path: &Path,
  from_import: bool,
) -> Result<Option<PathBuf>> {
  if !dir_exists(path) {
    return Ok(None);
  }
  if from_import {
    if let Ok(Some(p)) =
      exactly_one(try_path_with_extensions(&path.join("index.import")))
    {
      return Ok(Some(p));
    }
  }
  exactly_one(try_path_with_extensions(&path.join("index")))
}

fn try_path_with_extensions(path: &Path) -> Vec<PathBuf> {
  let result = [
    try_path(Path::new(&format!("{}.sass", path.to_string_lossy()))),
    try_path(Path::new(&format!("{}.scss", path.to_string_lossy()))),
  ]
  .concat();
  if result.is_empty() {
    try_path(Path::new(&format!("{}.css", path.to_string_lossy())))
  } else {
    result
  }
}

fn try_path(path: &Path) -> Vec<PathBuf> {
  let partial = path
    .parent()
    .unwrap()
    .join(format!("_{}", path.file_name().unwrap().to_string_lossy()));
  let mut result = Vec::new();
  if file_exists(&partial) {
    result.push(partial);
  }
  if file_exists(path) {
    result.push(path.to_path_buf());
  }
  result
}