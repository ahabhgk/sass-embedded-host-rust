use std::time;

use sass_embedded::{Options, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
    .join("dart-sass-embedded")
}

fn main() {
  let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("examples/abc.scss");
  let now = time::Instant::now();
  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass.compile(&path, Options::default()).unwrap();
  dbg!(res.loaded_urls, now.elapsed());
}
