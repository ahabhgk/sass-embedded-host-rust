use std::{
  ffi::OsStr,
  io::{Read, Write},
  ops::DerefMut,
  process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
};

use parking_lot::Mutex;
use prost::Message;

use crate::{
  pb::{InboundMessage, OutboundMessage},
  varint,
};

#[derive(Debug)]
pub struct Compiler {
  stdin: Mutex<ChildStdin>,
  stdout: Mutex<ChildStdout>,
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
    let stdin = Mutex::new(cmd.stdin.unwrap());
    let stdout = Mutex::new(cmd.stdout.unwrap());
    let stderr = cmd.stderr.unwrap();

    Self {
      stdin,
      stdout,
      stderr,
    }
  }

  // pub fn write(&mut self, payload: &[u8]) -> Result<(), std::io::Error> {
  //   varint::write(&mut self.stdin, payload.len())?;
  //   self.stdin.write_all(payload)
  // }

  // pub fn read(&mut self) -> Result<Vec<u8>, std::io::Error> {
  //   let len = varint::read(&mut self.stdout)?;
  //   let mut buf = vec![0; len];
  //   self.stdout.read(&mut buf)?;
  //   Ok(buf)
  // }

  pub fn write(&self, message: InboundMessage) -> Result<(), std::io::Error> {
    let buf = message.encode_length_delimited_to_vec();
    self.stdin.lock().write(&buf)?;
    Ok(())
  }

  pub fn read(&self) -> Result<OutboundMessage, std::io::Error> {
    let mut stdout = self.stdout.lock();
    let len = varint::read(stdout.deref_mut())?;
    let mut buf = vec![0; len];
    stdout.read_exact(&mut buf)?;
    let msg = OutboundMessage::decode(&buf[..])?;
    Ok(msg)
  }
}
