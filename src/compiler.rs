use std::{
  io::{Read, Write},
  process::{Command, Stdio},
};

use crate::compiler_path;

pub struct Connection {}

impl Connection {
  pub fn new() {
    let path = compiler_path::compiler_path().unwrap();
    let mut child = Command::new(path)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
  }
}
