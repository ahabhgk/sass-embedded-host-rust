fn main() {
  prost_build::compile_protos(&["src/embedded_sass.proto"], &["src/"]).unwrap();
}
