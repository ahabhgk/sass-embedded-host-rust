use std::{ffi::OsStr, path::Path};

use crate::{channel::Channel, connection::Connection};

#[derive(Debug)]
pub struct Embedded {
  channel: Channel,
}

impl Embedded {
  pub fn new(exe_path: impl AsRef<OsStr>) -> Self {
    Self {
      channel: Channel::new(exe_path),
    }
  }

  // pub fn compile(path: &Path) ->

  pub fn info(&mut self) -> String {
    let conn = self.channel.connect();
    let response = conn.version_request().unwrap();
    format!("sass-embedded\t#{}", response.implementation_version)
  }
}
