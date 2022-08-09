use std::process::Command;

use prost_build::Config;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  println!("cargo:rerun-if-changed=ext/sass/sass-embedded.proto");
  Config::new()
    .out_dir("src/protocol")
    .compile_protos(&["ext/sass/sass-embedded.proto"], &["."])
    .unwrap();

  println!("cargo:rerun-if-changed=.rustfmt.toml");
  Command::new("rustfmt")
    .arg("./src/protocol/sass.embedded_protocol.rs")
    .spawn()
    .unwrap();
}
