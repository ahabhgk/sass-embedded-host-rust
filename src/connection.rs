use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

use crate::{
  dispatcher::Dispatcher,
  pb::{
    inbound_message::{self, VersionRequest},
    outbound_message::{self, CompileResponse, VersionResponse},
    InboundMessage,
  },
};

#[derive(Debug)]
pub struct Connected {
  id: u32,
  tx: Sender<outbound_message::Message>,
  rx: Receiver<outbound_message::Message>,
}

#[derive(Debug)]
pub struct Unconnected;

#[derive(Debug)]
pub struct Connection<S> {
  state: S,
  dispatcher: Arc<Dispatcher>,
}

impl Connection<Unconnected> {
  pub fn new(dispatcher: Arc<Dispatcher>) -> Connection<Unconnected> {
    Self {
      state: Unconnected,
      dispatcher,
    }
  }

  pub fn connect(self, id: u32) -> Arc<Connection<Connected>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    Arc::new(Connection::<Connected> {
      state: Connected { id, tx, rx },
      dispatcher: self.dispatcher,
    })
  }
}

impl Connection<Connected> {
  fn id(&self) -> u32 {
    self.state.id
  }

  pub fn disconnect(&self) {
    self.dispatcher.unsubscribe(&self.id());
  }

  fn send_message(
    &self,
    inbound_message: InboundMessage,
  ) -> Result<(), std::io::Error> {
    self.dispatcher.send_message(inbound_message)
  }

  pub fn error(&self, message: &str) {}

  pub fn compile_response(&self, response: CompileResponse) {}

  pub fn version_request(&self) -> Result<VersionResponse, ()> {
    self
      .send_message(InboundMessage::new(
        inbound_message::Message::VersionRequest(VersionRequest {
          id: self.id(),
        }),
      ))
      .unwrap();
    if let outbound_message::Message::VersionResponse(response) =
      self.state.rx.recv().unwrap()
    {
      return Ok(response);
    }
    unreachable!()
  }

  pub fn version_response(&self, response: outbound_message::Message) {
    self.state.tx.send(response).unwrap();
  }
}
