use sass_embedded_host_rust::{Options, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

fn main() {
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
  dbg!(res1, res2, res3, res4);
}
