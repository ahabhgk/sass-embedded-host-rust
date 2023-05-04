// `pushd xtask/build_proto && cargo run -p build_proto && mv ./sass.embedded_protocol.rs ../../src/protocol.rs && popd && cargo fmt`
fn main() {
  prost_build::Config::new()
    .out_dir(".")
    .compile_protos(&["ext/sass/sass-embedded.proto"], &["../.."])
    .unwrap();
}
