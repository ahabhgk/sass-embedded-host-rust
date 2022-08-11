use std::{
  env, fs,
  io::{Read, Write},
  path::{Path, PathBuf},
};

use gag::BufferRedirect;
use sass_embedded_host_rust::{Sass, Url};
use tempfile::TempDir;

#[test]
fn version_smoke() {
  let mut sass = Sass::new(exe_path());
  let info = sass.info().unwrap();
  // !!! the crate's version should be the same as the embedded's version !!!
  assert_eq!(info, "sass-embedded\t#1.54.4");
}

#[cfg(target_family = "windows")]
pub fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
    .join("dart-sass-embedded.bat")
}

#[cfg(target_family = "unix")]
pub fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
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

  #[cfg(feature = "legacy")]
  #[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub struct Captured {
  pub out: String,
  pub err: String,
}

#[allow(dead_code)]
pub fn capture_stdio(f: impl Fn()) -> Captured {
  let mut stdout = BufferRedirect::stdout().unwrap();
  let mut stderr = BufferRedirect::stderr().unwrap();
  f();
  let mut out = String::new();
  let mut err = String::new();
  stdout.read_to_string(&mut out).unwrap();
  stderr.read_to_string(&mut err).unwrap();
  Captured { out, err }
}

pub trait ToUrl {
  fn to_url(&self) -> Url;
}

impl ToUrl for Path {
  fn to_url(&self) -> Url {
    Url::from_file_path(self).unwrap()
  }
}
