use std::process::Command;

use prost_build::Config;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=embedded_sass.proto");
  Config::new()
    .out_dir("src/pb")
    .compile_protos(&["embedded_sass.proto"], &["."])
    .unwrap();
  Command::new("rustfmt")
    .arg("./src/pb/sass_embedded_protocol.rs")
    .spawn()
    .unwrap();
}
