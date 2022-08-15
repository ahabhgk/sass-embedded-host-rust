use std::{
  ffi::{OsStr, OsString},
  sync::Arc,
};

use crate::{
  compiler::Compiler,
  connection::{ConnectedGuard, Connection},
  dispatcher::Dispatcher,
  host::Host,
  Result,
};

#[derive(Debug)]
pub struct Channel {
  path: OsString,
  dispatcher: Arc<Dispatcher>,
}

impl Channel {
  pub fn new(path: impl AsRef<OsStr>) -> Result<Self> {
    let path = path.as_ref().to_os_string();
    let compiler = Compiler::new(&path)?;
    let dispatcher = Dispatcher::new(compiler);
    Ok(Self { path, dispatcher })
  }

  pub fn connect(&mut self, host: Host) -> Result<ConnectedGuard> {
    let conn = Connection::new(Arc::clone(&self.dispatcher));
    match self.dispatcher.subscribe(conn, host) {
      Err((conn, host)) => {
        let compiler = Compiler::new(&self.path)?;
        self.dispatcher = Dispatcher::new(compiler);
        Ok(self.dispatcher.subscribe(conn, host).unwrap())
      }
      Ok(conn) => Ok(conn),
    }
  }
}
