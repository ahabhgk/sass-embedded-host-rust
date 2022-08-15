use std::{
  ffi::OsStr,
  io::{Read, Write},
  ops::DerefMut,
  process::{ChildStdin, ChildStdout, Command, Stdio},
};

use parking_lot::Mutex;
use prost::Message;

use crate::{
  protocol::{InboundMessage, OutboundMessage},
  varint, Exception, Result,
};

#[derive(Debug)]
pub struct Compiler {
  stdin: Mutex<ChildStdin>,
  stdout: Mutex<ChildStdout>,
}

impl Compiler {
  pub fn new(path: impl AsRef<OsStr>) -> Result<Self> {
    let cmd = Command::new(path)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|e| Exception::new(e.to_string()).set_source(e))?;
    let stdin = Mutex::new(cmd.stdin.unwrap());
    let stdout = Mutex::new(cmd.stdout.unwrap());

    Ok(Self { stdin, stdout })
  }

  pub fn write(&self, message: InboundMessage) {
    let buf = message.encode_to_vec();
    let mut stdin = self.stdin.lock();
    varint::write(stdin.deref_mut(), buf.len());
    stdin.write_all(&buf[..]).unwrap();
  }

  pub fn read(&self) -> OutboundMessage {
    let mut stdout = self.stdout.lock();
    let len = varint::read(stdout.deref_mut());
    let mut buf = vec![0; len];
    stdout.read_exact(&mut buf).unwrap();
    OutboundMessage::decode(&buf[..]).unwrap()
  }
}
