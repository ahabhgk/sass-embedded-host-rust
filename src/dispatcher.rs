use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::{
  compiler::Compiler,
  connection::{Connected, ConnectedGuard, Connection, Unconnected},
  pb::{outbound_message, InboundMessage, OutboundMessage},
};

#[derive(Debug)]
pub struct Dispatcher {
  compiler: Compiler,
  observers: DashMap<u32, Arc<Connection<Connected>>>,
  id: Mutex<u32>,
}

impl Dispatcher {
  const PROTOCOL_ERROR_ID: u32 = 0xffffffff; // u32::MAX

  pub fn new(compiler: Compiler) -> Arc<Dispatcher> {
    let this = Arc::new(Self {
      compiler,
      observers: DashMap::new(),
      id: Mutex::new(0),
    });
    Self::spawn(Arc::clone(&this));
    this
  }

  fn spawn(dispatcher: Arc<Dispatcher>) {
    std::thread::spawn(move || loop {
      dispatcher.receive_message(dispatcher.compiler.read());
    });
  }

  pub fn subscribe(
    &self,
    observer: Connection<Unconnected>,
  ) -> Result<ConnectedGuard, Connection<Unconnected>> {
    let mut id = self.id.lock();
    if *id == Self::PROTOCOL_ERROR_ID {
      return Err(observer);
    }
    let observer = observer.connect(*id);
    self.observers.insert(*id, Arc::clone(&observer));
    *id += 1;
    Ok(observer)
  }

  pub fn unsubscribe(&self, id: &u32) {
    self.observers.remove(&id);
  }

  pub fn send_message(&self, inbound_message: InboundMessage) {
    self.compiler.write(inbound_message);
  }

  fn receive_message(&self, outbound_message: OutboundMessage) {
    let oneof = outbound_message.message.unwrap();
    match oneof {
      outbound_message::Message::Error(e) => {
        *self.id.lock() = Self::PROTOCOL_ERROR_ID;
        if e.id == Self::PROTOCOL_ERROR_ID {
          for ob in self.observers.iter() {
            ob.error(e.clone());
          }
        } else {
          if let Some(ob) = self.observers.get(&e.id) {
            ob.error(e);
          }
        }
      }
      outbound_message::Message::CompileResponse(response) => {
        if let Some(ob) = self.observers.get(&response.id) {
          ob.compile_response(response);
        }
      }
      outbound_message::Message::VersionResponse(response) => {
        if let Some(ob) = self.observers.get(&response.id) {
          ob.version_response(response);
        }
      }
      outbound_message::Message::LogEvent(_) => todo!(),
      outbound_message::Message::CanonicalizeRequest(_) => todo!(),
      outbound_message::Message::ImportRequest(_) => todo!(),
      outbound_message::Message::FileImportRequest(_) => todo!(),
      outbound_message::Message::FunctionCallRequest(_) => todo!(),
    }
  }
}
