use std::time;

use sass_embedded_host_rust::{Options, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

fn main() {
  let path = exe_path()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("examples/big_scss/big.scss");
  let path = &path.to_string_lossy().into_owned();
  let now = time::Instant::now();
  let mut sass = Sass::new(exe_path());
  let _ = sass.compile(path, Options::default()).unwrap();
  let _ = sass.compile(path, Options::default()).unwrap();
  let _ = sass.compile(path, Options::default()).unwrap();
  let _ = sass.compile(path, Options::default()).unwrap();
  dbg!(now.elapsed());
}
