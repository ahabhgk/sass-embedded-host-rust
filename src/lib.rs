mod channel;
mod compiler;
mod connection;
mod dispatcher;
mod embedded;
mod error;
mod importer_registry;
mod logger_registry;
mod options;
mod protocol;
mod varint;

pub use embedded::{CompileResult, Embedded, Embedded as Sass};
pub use error::{Exception, Result};
pub use options::{
  FileImporter, Importer, Logger, LoggerDebugOptions, LoggerWarnOptions,
  Options, OptionsBuilder, SassLogger, StringOptions, StringOptionsBuilder,
};
pub use protocol::{OutputStyle, Syntax};
pub use url::{self, Url};

#[cfg(test)]
pub fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn version_smoke() {
    let mut sass = Sass::new(exe_path());
    let info = sass.info().unwrap();
    assert_eq!(info, "sass-embedded\t#1.54.0");
  }
}
