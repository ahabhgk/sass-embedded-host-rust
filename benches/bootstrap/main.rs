use std::process::Command;

use criterion::{criterion_group, criterion_main, Criterion};
use sass_embedded::{OptionsBuilder, OutputStyle, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
    .join("dart-sass-embedded")
}

fn host_rust() {
  let mut sass = Sass::new(exe_path()).unwrap();
  let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("benches/bootstrap/bootstrap/scss");
  let _ = sass.compile(
    path.join("bootstrap.scss"),
    OptionsBuilder::default()
      .style(OutputStyle::Expanded)
      .source_map(true)
      .build(),
  );
  let _ = sass.compile(
    path.join("bootstrap-grid.scss"),
    OptionsBuilder::default()
      .style(OutputStyle::Expanded)
      .source_map(true)
      .build(),
  );
  let _ = sass.compile(
    path.join("bootstrap-reboot.scss"),
    OptionsBuilder::default()
      .style(OutputStyle::Expanded)
      .source_map(true)
      .build(),
  );
  let _ = sass.compile(
    path.join("bootstrap-utilities.scss"),
    OptionsBuilder::default()
      .style(OutputStyle::Expanded)
      .source_map(true)
      .build(),
  );
}

fn host_node() {
  let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("benches/bootstrap/sass-embedded.mjs");
  let status = Command::new("node")
    .arg(path)
    .spawn()
    .unwrap()
    .wait()
    .unwrap();
  assert!(status.success());
}

fn dart_sass() {
  let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("benches/bootstrap/dart-sass.mjs");
  let status = Command::new("node")
    .arg(path)
    .spawn()
    .unwrap()
    .wait()
    .unwrap();
  assert!(status.success());
}

fn benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("bootstrap");
  group.sample_size(10);
  group.bench_function("Host Rust", |b| b.iter(|| host_rust()));
  group.bench_function("Host Node", |b| b.iter(|| host_node()));
  group.bench_function("Dart Sass", |b| b.iter(|| dart_sass()));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
