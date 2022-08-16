use sass_embedded::{Sass, StringOptionsBuilder};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
    .join("dart-sass-embedded")
}

fn main() {
  let mut sass = Sass::new(exe_path()).unwrap();
  let res = sass
    .compile_string(
      "a {b: c}",
      StringOptionsBuilder::default().source_map(true).build(),
    )
    .unwrap();
  dbg!(res);
}
