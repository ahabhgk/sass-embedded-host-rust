use std::{env, path::PathBuf, process::Command};

use prost_build::Config;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=ext/sass/sass-embedded.proto");
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  Config::new()
    .out_dir(&out_dir)
    .compile_protos(&["ext/sass/sass-embedded.proto"], &["."])
    .unwrap();
  Command::new("rustfmt")
    .arg(out_dir.join("sass.embedded_protocol.rs"))
    .spawn()
    .unwrap();
}
