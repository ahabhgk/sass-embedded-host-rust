use std::{fs, io::Write, path::Path};

use sass_embedded_host_rust::Url;
use tempfile::TempDir;

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
}

pub trait ToUrl {
  fn to_url(&self) -> Url;
}

impl ToUrl for Path {
  fn to_url(&self) -> Url {
    Url::from_file_path(self).unwrap()
  }
}
