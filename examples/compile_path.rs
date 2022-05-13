use sass_embedded_host_rust::{compile, Options};

#[tokio::main]
async fn main() {
  let res = compile(
    "/Users/bytedance/Codes/sass-embedded-host-rust/examples/simple.scss"
      .to_string(),
    Options::default(),
  )
  .await
  .unwrap();
  dbg!(res.loaded_urls);
}
