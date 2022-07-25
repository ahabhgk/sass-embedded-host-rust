use dashmap::DashMap;
use parking_lot::Mutex;
use prost::Message;
use std::sync::Arc;

use crate::{
  compiler::Compiler,
  connection::{Connected, Connection, Unconnected},
  pb::{outbound_message, InboundMessage, OutboundMessage},
};

#[derive(Debug)]
pub struct Dispatcher {
  compiler: Mutex<Compiler>,
  observers: DashMap<u32, Arc<Connection<Connected>>>,
  id: Mutex<u32>,
}

impl Dispatcher {
  const PROTOCOL_ERROR_ID: u32 = 0xffffffff;

  pub fn new(compiler: Compiler) -> Arc<Dispatcher> {
    let this = Arc::new(Self {
      compiler: Mutex::new(compiler),
      observers: DashMap::new(),
      id: Mutex::new(0),
    });
    Self::spawn(Arc::clone(&this));
    this
  }

  fn spawn(dispatcher: Arc<Dispatcher>) {
    std::thread::spawn(move || loop {
      match dispatcher.compiler.lock().read() {
        Ok(buf) => {
          dispatcher.receive_message(OutboundMessage::decode(&buf[..]).unwrap())
        }
        Err(e) => {
          *dispatcher.id.lock() = Self::PROTOCOL_ERROR_ID;
          for ob in dispatcher.observers.iter() {
            ob.error(&e.to_string());
          }
          break;
        }
      }
    });
  }

  pub fn subscribe(
    &self,
    observer: Connection<Unconnected>,
  ) -> Result<Arc<Connection<Connected>>, Connection<Unconnected>> {
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

  pub fn send_message(
    &self,
    inbound_message: InboundMessage,
  ) -> Result<(), std::io::Error> {
    self.compiler.lock().write(&inbound_message.encode_to_vec())
  }

  fn receive_message(&self, outbound_message: OutboundMessage) {
    let oneof = outbound_message.message.unwrap();
    match oneof {
      outbound_message::Message::Error(e) => {
        *self.id.lock() = Self::PROTOCOL_ERROR_ID;
        if e.id == Self::PROTOCOL_ERROR_ID {
          for ob in self.observers.iter() {
            ob.error(&e.message);
          }
        } else {
          if let Some(ob) = self.observers.get(&e.id) {
            ob.error(&e.message);
          }
        }
      }
      outbound_message::Message::CompileResponse(r) => {
        if let Some(ob) = self.observers.get(&r.id) {
          ob.compile_response(r);
        }
      }
      outbound_message::Message::VersionResponse(r) => {
        if let Some(ob) = self.observers.get(&r.id) {
          ob.version_response(outbound_message::Message::VersionResponse(r));
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
