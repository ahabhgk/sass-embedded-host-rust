use std::{
  env, fs,
  io::Write,
  path::{Path, PathBuf},
};

use sass_embedded_host_rust::Url;
use tempfile::TempDir;

pub fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("sass_embedded")
    .join("dart-sass-embedded")
}

#[derive(Debug)]
pub struct Sandbox {
  temp: TempDir,
}

impl Default for Sandbox {
  fn default() -> Self {
    Self {
      temp: TempDir::new().unwrap(),
    }
  }
}

impl Sandbox {
  pub fn path(&self) -> &Path {
    self.temp.path()
  }

  pub fn write(&self, path: impl AsRef<Path>, contents: &str) -> &Self {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = fs::File::create(path).unwrap();
    writeln!(file, "{}", contents).unwrap();
    self
  }

  // Since cargo test will run tests in parallel, and the host uses the
  // cwd when legacy is on, so we need to run legacy tests in sequentially
  // by adding `--test-threads=1`
  #[cfg(feature = "legacy")]
  pub fn chdir(&self) -> ChdirGuard {
    let cwd = env::current_dir().unwrap();
    env::set_current_dir(self.path()).unwrap();
    ChdirGuard(cwd)
  }
}

pub struct ChdirGuard(PathBuf);

impl Drop for ChdirGuard {
  fn drop(&mut self) {
    env::set_current_dir(&self.0).unwrap();
  }
}

pub trait ToUrl {
  fn to_url(&self) -> Url;
}

impl ToUrl for Path {
  fn to_url(&self) -> Url {
    Url::from_file_path(self).unwrap()
  }
}
