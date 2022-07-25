use std::{
  ffi::OsStr,
  io::{Read, Write},
  process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
};

use crate::varint;

#[derive(Debug)]
pub struct Compiler {
  stdin: ChildStdin,
  stdout: ChildStdout,
  stderr: ChildStderr,
}

impl Compiler {
  pub fn new(path: impl AsRef<OsStr>) -> Self {
    let cmd = Command::new(path)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    let stdin = cmd.stdin.unwrap();
    let stdout = cmd.stdout.unwrap();
    let stderr = cmd.stderr.unwrap();

    Self {
      stdin,
      stdout,
      stderr,
    }
  }

  pub fn write(&mut self, payload: &[u8]) -> Result<(), std::io::Error> {
    varint::write(&mut self.stdin, payload.len())?;
    self.stdin.write_all(payload)
  }

  pub fn read(&mut self) -> Result<Vec<u8>, std::io::Error> {
    let len = varint::read(&mut self.stdout)?;
    let mut buf = vec![0; len];
    self.stdout.read(&mut buf)?;
    Ok(buf)
  }
}
