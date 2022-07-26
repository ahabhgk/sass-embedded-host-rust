mod channel;
mod compiler;
mod connection;
mod dispatcher;
mod embedded;
mod pb;
mod varint;

pub use embedded::Embedded;
pub use embedded::Embedded as Sass;

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;

  fn exe_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
      .join("sass_embedded")
      .join("dart-sass-embedded")
  }

  #[test]
  fn version_smoke() {
    let mut sass = Sass::new(exe_path());
    let info = sass.info();
    assert_eq!(info, "sass-embedded\t#1.54.0");
  }
}
