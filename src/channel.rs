use std::{
  ffi::{OsStr, OsString},
  sync::Arc,
};

use crate::{
  compiler::Compiler,
  connection::{ConnectedGuard, Connection},
  dispatcher::Dispatcher,
};

#[derive(Debug)]
pub struct Channel {
  path: OsString,
  dispatcher: Arc<Dispatcher>,
}

impl Channel {
  pub fn new(path: impl AsRef<OsStr>) -> Self {
    let path = path.as_ref().to_os_string();
    let compiler = Compiler::new(&path);
    let dispatcher = Dispatcher::new(compiler);
    Self { path, dispatcher }
  }

  pub fn connect(&mut self) -> ConnectedGuard {
    let conn = Connection::new(Arc::clone(&self.dispatcher));
    match self.dispatcher.subscribe(conn) {
      Err(conn) => {
        let compiler = Compiler::new(&self.path);
        self.dispatcher = Dispatcher::new(compiler);
        self.dispatcher.subscribe(conn).unwrap()
      }
      Ok(conn) => conn,
    }
  }
}
