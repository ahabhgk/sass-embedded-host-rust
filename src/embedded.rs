use std::{ffi::OsStr, path::Path};

use crate::channel::Channel;

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
    let response = self.channel.connect().version_request().unwrap();
    format!("sass-embedded\t#{}", response.implementation_version)
  }
}
