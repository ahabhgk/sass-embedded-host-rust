use std::ffi::OsStr;

use crate::{
  channel::Channel,
  pb::inbound_message::{compile_request::Input, CompileRequest},
  CompileResult, Options, Result,
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
  ) -> Result<CompileResult> {
    let conn = self.channel.connect();
    let mut request = CompileRequest::from(options);
    request.input = Some(Input::Path(path.into()));
    let response = conn.compile_request(request)?;
    Ok(CompileResult::try_from(response)?)
  }

  pub fn info(&mut self) -> Result<String> {
    let conn = self.channel.connect();
    let response = conn.version_request()?;
    Ok(format!(
      "sass-embedded\t#{}",
      response.implementation_version
    ))
  }
}
