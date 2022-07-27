use sass_embedded_host_rust::{compile, Options};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

#[tokio::main]
async fn main() {
  let mut o = Options::default();
  o.exe_path = Some(exe_path().to_string_lossy().to_string());
  let res = compile(
    "/Users/bytedance/Codes/sass-embedded-host-rust/examples/simple.scss"
      .to_string(),
    o,
  )
  .await
  .unwrap();
  dbg!(res.loaded_urls);
}
