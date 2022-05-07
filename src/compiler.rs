use std::{ffi::OsStr, process::Stdio};

use prost::bytes::Bytes;
use tokio::{
  io::{AsyncWriteExt, BufReader},
  process::{ChildStdin, ChildStdout, Command},
};
use tokio_util::io::ReaderStream;

pub struct Embedded {
  reader: ReaderStream<BufReader<ChildStdout>>,
  stdin: ChildStdin,
}

impl Embedded {
  pub fn new(program: impl AsRef<OsStr>) -> Self {
    let mut child = Command::new(program)
      .kill_on_drop(true)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stdin = child.stdin.take().unwrap();
    let reader = ReaderStream::new(BufReader::new(stdout));

    Self { reader, stdin }
  }

  pub async fn write(&mut self, buf: &Bytes) -> Result<usize, std::io::Error> {
    self.stdin.write(buf).await
  }

  pub fn reader(&self) -> &ReaderStream<BufReader<ChildStdout>> {
    &self.reader
  }

  pub async fn compile_string() {
    
  }
}
