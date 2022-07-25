use std::sync::Mutex;

use prost::Message;
use tokio::{io::AsyncWriteExt, process::ChildStdin};

use crate::{
  importer_registry::ImporterRegistry,
  logger_registry::LoggerRegistry,
  pb::{
    inbound_message::{self, CompileRequest},
    outbound_message::{self, CompileResponse},
    InboundMessage,
  },
  request_tracker::RequestTracker,
  Error, Result,
};

pub struct Dispatcher<'i, 'l> {
  importers: &'i ImporterRegistry,
  logger: &'l LoggerRegistry,
  stdin: Mutex<ChildStdin>,
  pending_inbound_requests: RequestTracker,
  pending_outbound_requests: RequestTracker,
}

impl<'i, 'l> Dispatcher<'i, 'l> {
  pub fn new(
    stdin: ChildStdin,
    importers: &'i ImporterRegistry,
    logger: &'l LoggerRegistry,
  ) -> Self {
    Self {
      importers,
      logger,
      stdin: Mutex::new(stdin),
      pending_inbound_requests: RequestTracker::new(),
      pending_outbound_requests: RequestTracker::new(),
    }
  }

  pub async fn send_compile_request(
    &mut self,
    request: CompileRequest,
  ) -> Result<()> {
    let id = self.pending_inbound_requests.next_id();
    self
      .send_inbound_message(
        id,
        inbound_message::Message::CompileRequest(request),
      )
      .await
  }

  async fn send_inbound_message(
    &self,
    id: u32,
    mut message: inbound_message::Message,
  ) -> Result<()> {
    match &mut message {
      inbound_message::Message::CompileRequest(request) => {
        request.id = id;
        self.pending_inbound_requests.add(id);
      }
      inbound_message::Message::CanonicalizeResponse(response) => {
        response.id = id;
        self.pending_outbound_requests.resolve(id);
      }
      inbound_message::Message::ImportResponse(response) => {
        response.id = id;
        self.pending_outbound_requests.resolve(id);
      }
      inbound_message::Message::FileImportResponse(response) => {
        response.id = id;
        self.pending_outbound_requests.resolve(id);
      }
      inbound_message::Message::FunctionCallResponse(response) => {
        response.id = id;
        self.pending_outbound_requests.resolve(id);
      }
      inbound_message::Message::VersionRequest(_) => unreachable!(),
    };
    let inbound = InboundMessage::new(message);
    let buf = inbound.encode_length_delimited_to_vec();
    self.stdin.lock().unwrap().write_all(&buf).await?;
    Ok(())
  }

  pub async fn handle_outbound_message(
    &self,
    message: outbound_message::Message,
  ) -> Result<Option<CompileResponse>> {
    match message {
      outbound_message::Message::CompileResponse(response) => {
        self.pending_inbound_requests.resolve(response.id);
        Ok(Some(response))
      }
      outbound_message::Message::CanonicalizeRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        let response = self.importers.canonicalize(&request).await?;
        self
          .send_inbound_message(
            request.id,
            inbound_message::Message::CanonicalizeResponse(response),
          )
          .await?;
        Ok(None)
      }
      outbound_message::Message::ImportRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        let response = self.importers.import(&request).await?;
        self
          .send_inbound_message(
            request.id,
            inbound_message::Message::ImportResponse(response),
          )
          .await?;
        Ok(None)
      }
      outbound_message::Message::FileImportRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        let response = self.importers.file_import(&request).await?;
        self
          .send_inbound_message(
            request.id,
            inbound_message::Message::FileImportResponse(response),
          )
          .await?;
        Ok(None)
      }
      outbound_message::Message::FunctionCallRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        unimplemented!("Not supported yet");
      }
      outbound_message::Message::Error(e) => Err(Error::Host(e.message)),
      outbound_message::Message::LogEvent(e) => {
        self.logger.log(e);
        Ok(None)
      }
      outbound_message::Message::VersionResponse(_) => unreachable!(),
    }
  }
}
