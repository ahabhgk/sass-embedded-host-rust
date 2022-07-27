use std::{ffi::OsStr, path::Path};

use crate::{
  api::{CompileResult, Options},
  channel::Channel,
  connection::Connection,
  pb::{
    inbound_message::{compile_request::Input, CompileRequest},
    OutputStyle,
  },
};

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

  pub fn compile(
    &mut self,
    path: impl Into<String>,
    options: Options,
  ) -> Result<CompileResult, ()> {
    let conn = self.channel.connect();
    let mut request = CompileRequest::from(options);
    request.input = Some(Input::Path(path.into()));
    let response = conn.compile_request(request).unwrap();
    Ok(CompileResult::try_from(response)?)
  }

  pub fn info(&mut self) -> Result<String, String> {
    let conn = self.channel.connect();
    let response = conn.version_request()?;
    Ok(format!(
      "sass-embedded\t#{}",
      response.implementation_version
    ))
  }
}
