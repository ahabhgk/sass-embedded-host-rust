use std::process::Command;

use prost_build::Config;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=embedded-protocol/embedded_sass.proto");
  Config::new()
    .out_dir("src/protocol")
    .compile_protos(&["embedded-protocol/embedded_sass.proto"], &["."])
    .unwrap();
  Command::new("rustfmt")
    .arg("./src/protocol/sass.embedded_protocol.rs")
    .spawn()
    .unwrap();
}
