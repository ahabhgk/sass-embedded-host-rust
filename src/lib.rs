pub mod api;
mod channel;
mod compiler;
mod connection;
mod dispatcher;
mod embedded;
mod pb;
mod varint;

pub use api::{
  CompileResult, Options, OptionsBuilder, StringOptions, StringOptionsBuilder,
};
pub use embedded::Embedded;
pub use embedded::Embedded as Sass;
pub use pb::{OutputStyle, Syntax};
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
