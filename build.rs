use prost_build::Config;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=embedded_sass.proto");
  Config::new()
    .bytes(&["."])
    .out_dir("src/pb")
    .compile_protos(&["embedded_sass.proto"], &["."])
    .unwrap();
}
