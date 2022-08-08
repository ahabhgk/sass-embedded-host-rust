use std::time;

#[cfg(feature = "legacy")]
use sass_embedded_host_rust::legacy::LegacyOptionsBuilder;
use sass_embedded_host_rust::{Options, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

fn main() {
  let path = exe_path()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("examples/bootstrap5/bootstrap/scss");
  let bootstrap = path.join("bootstrap.scss");
  let bootstrap_grid = path.join("bootstrap-grid.scss");
  let bootstrap_reboot = path.join("bootstrap-reboot.scss");
  let bootstrap_utilities = path.join("bootstrap-utilities.scss");
  let mut sass = Sass::new(exe_path());
  let _ = sass
    .compile(bootstrap.to_string_lossy(), Options::default())
    .unwrap();
  let _ = sass
    .compile(bootstrap_grid.to_string_lossy(), Options::default())
    .unwrap();
  let _ = sass
    .compile(bootstrap_reboot.to_string_lossy(), Options::default())
    .unwrap();
  let _ = sass
    .compile(bootstrap_utilities.to_string_lossy(), Options::default())
    .unwrap();

  let now = time::Instant::now();
  let _ = sass
    .compile(bootstrap.to_string_lossy(), Options::default())
    .unwrap();
  let _ = sass
    .compile(bootstrap_grid.to_string_lossy(), Options::default())
    .unwrap();
  let _ = sass
    .compile(bootstrap_reboot.to_string_lossy(), Options::default())
    .unwrap();
  let _ = sass
    .compile(bootstrap_utilities.to_string_lossy(), Options::default())
    .unwrap();
  println!("modern: {:?}", now.elapsed());

  #[cfg(feature = "legacy")]
  {
    let now = time::Instant::now();
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(bootstrap.to_string_lossy())
          .build(),
      )
      .unwrap();
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(bootstrap_grid.to_string_lossy())
          .build(),
      )
      .unwrap();
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(bootstrap_reboot.to_string_lossy())
          .build(),
      )
      .unwrap();
    let _ = sass
      .render(
        LegacyOptionsBuilder::default()
          .file(bootstrap_utilities.to_string_lossy())
          .build(),
      )
      .unwrap();
    println!("legacy: {:?}", now.elapsed());
  }
}
