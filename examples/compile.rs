use std::time;

use sass_embedded_host_rust::{Options, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

fn main() {
  let now = time::Instant::now();
  let mut sass = Sass::new(exe_path());
  let res1 = sass
    .compile(
      "/Users/bytedance/Codes/sass-embedded-host-rust/examples/simple.scss",
      Options::default(),
    )
    .unwrap();
  let res2 = sass
    .compile(
      "/Users/bytedance/Codes/sass-embedded-host-rust/examples/simple.scss",
      Options::default(),
    )
    .unwrap();
  let res3 = sass
    .compile(
      "/Users/bytedance/Codes/sass-embedded-host-rust/examples/simple.scss",
      Options::default(),
    )
    .unwrap();
  let res4 = sass
    .compile(
      "/Users/bytedance/Codes/sass-embedded-host-rust/examples/simple.scss",
      Options::default(),
    )
    .unwrap();
  dbg!(now.elapsed());
  dbg!(
    res1.loaded_urls,
    res2.loaded_urls,
    res3.loaded_urls,
    res4.loaded_urls
  );
}
