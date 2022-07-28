use std::{
  fmt::Debug,
  ops::{Deref, DerefMut},
  sync::Arc,
};

use crossbeam_channel::{Receiver, Sender};

use crate::{
  dispatcher::Dispatcher,
  pb::{
    inbound_message::{self, CompileRequest, VersionRequest},
    outbound_message::{CompileResponse, VersionResponse},
    InboundMessage, ProtocolError,
  },
};

enum ProtocolResponse {
  Compile(CompileResponse),
  Version(VersionResponse),
}

type Response = Result<ProtocolResponse, ProtocolError>;

#[derive(Debug)]
pub struct Connected {
  id: u32,
  tx: Sender<Response>,
  rx: Receiver<Response>,
}

#[derive(Debug)]
pub struct Unconnected;

pub struct Connection<S: Debug> {
  state: S,
  dispatcher: Arc<Dispatcher>,
}

impl<S: Debug> Debug for Connection<S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.state.fmt(f)
  }
}

pub struct ConnectedGuard(pub(crate) Arc<Connection<Connected>>);

impl Drop for ConnectedGuard {
  fn drop(&mut self) {
    self.0.disconnect();
  }
}

impl Deref for ConnectedGuard {
  type Target = Arc<Connection<Connected>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ConnectedGuard {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Connection<Unconnected> {
  pub fn new(dispatcher: Arc<Dispatcher>) -> Connection<Unconnected> {
    Self {
      state: Unconnected,
      dispatcher,
    }
  }

  pub fn connect(self, id: u32) -> ConnectedGuard {
    let (tx, rx) = crossbeam_channel::bounded(1);
    ConnectedGuard(Arc::new(Connection {
      state: Connected { id, tx, rx },
      dispatcher: self.dispatcher,
    }))
  }
}

impl Connection<Connected> {
  fn id(&self) -> u32 {
    self.state.id
  }

  fn disconnect(&self) {
    self.dispatcher.unsubscribe(&self.id());
  }

  fn send_message(&self, inbound_message: InboundMessage) {
    self.dispatcher.send_message(inbound_message);
  }

  pub fn error(&self, message: ProtocolError) {
    self.response(Err(message));
  }

  pub fn compile_request(
    &self,
    mut request: CompileRequest,
  ) -> Result<CompileResponse, ProtocolError> {
    request.id = self.id();
    self.send_message(InboundMessage::new(
      inbound_message::Message::CompileRequest(request),
    ));
    self
      .state
      .rx
      .recv()
      .unwrap()
      .map(|response| match response {
        ProtocolResponse::Compile(response) => response,
        _ => unreachable!(),
      })
  }

  pub fn compile_response(&self, response: CompileResponse) {
    self.response(Ok(ProtocolResponse::Compile(response)));
  }

  pub fn version_request(&self) -> Result<VersionResponse, ProtocolError> {
    self.send_message(InboundMessage::new(
      inbound_message::Message::VersionRequest(VersionRequest {
        id: self.id(),
      }),
    ));
    self
      .state
      .rx
      .recv()
      .unwrap()
      .map(|response| match response {
        ProtocolResponse::Version(response) => response,
        _ => unreachable!(),
      })
  }

  pub fn version_response(&self, response: VersionResponse) {
    self.response(Ok(ProtocolResponse::Version(response)));
  }

  fn response(&self, response: Response) {
    self.state.tx.send(response).unwrap();
  }
}
