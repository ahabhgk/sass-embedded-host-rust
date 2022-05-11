use std::{ffi::OsStr, process::Stdio};

use futures::{future, StreamExt, TryStreamExt};
use prost::Message as _;
use tokio::{
  io::{AsyncWriteExt, BufReader},
  process::{ChildStdin, ChildStdout, Command},
};
use tokio_util::io::ReaderStream;

use crate::{
  api::{CompileResult, StringOptions},
  importer_registry::ImporterRegistry,
  pb::{
    inbound_message::{self, CompileRequest},
    outbound_message::{self, CompileResponse},
    InboundMessage, OutboundMessage,
  },
  request_tracker::RequestTracker,
  Error, Result,
};

pub struct Embedded {
  reader: ReaderStream<BufReader<ChildStdout>>,
  stdin: ChildStdin,

  pending_inbound_requests: RequestTracker,
  pending_outbound_requests: RequestTracker,
}

impl Embedded {
  pub fn new(
    program: impl AsRef<OsStr>,
    importers: &mut ImporterRegistry,
  ) -> Self {
    let mut child = Command::new(program)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stdin = child.stdin.take().unwrap();
    let reader = ReaderStream::new(BufReader::new(stdout));

    let this = Self {
      reader,
      stdin,
      pending_inbound_requests: RequestTracker::new(),
      pending_outbound_requests: RequestTracker::new(),
    };

    let reader = reader
      .map_err(|e| Error::from(e))
      .and_then(|buf| {
        let outbound = OutboundMessage::decode_length_delimited(buf).unwrap();
        future::ok(outbound.message.unwrap())
      })
      .try_filter_map(|message| async move {
        this.handle_outbound_message(message, importers).await
      });

    this
  }

  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.stdin.write(buf).await.map_err(|e| e.into())
  }

  pub async fn send_compile_request(
    &mut self,
    request: CompileRequest,
  ) -> Result<CompileResult> {
    let id = self.pending_inbound_requests.next_id();
    self
      .handle_inbound_message(
        id,
        inbound_message::Message::CompileRequest(request),
      )
      .await?;
    Ok(())
  }

  async fn handle_inbound_message(
    &mut self,
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
      _ => panic!("Unknown message type {message:?}"),
    };
    let inbound = InboundMessage::new(message);
    let buf = inbound.encode_length_delimited_to_vec();
    self.write(&buf).await?;
    Ok(())
  }

  async fn handle_outbound_message(
    &mut self,
    message: outbound_message::Message,
    importers: &mut ImporterRegistry,
  ) -> Result<Option<CompileResponse>> {
    match message {
      outbound_message::Message::CompileResponse(response) => {
        self.pending_inbound_requests.resolve(response.id);
        Ok(Some(response))
      }
      outbound_message::Message::CanonicalizeRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        let response = importers.canonicalize(&request).await?;
        self
          .handle_inbound_message(
            request.id,
            inbound_message::Message::CanonicalizeResponse(response),
          )
          .await?;
        Ok(None)
      }
      outbound_message::Message::ImportRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        let response = importers.import(&request).await?;
        self
          .handle_inbound_message(
            request.id,
            inbound_message::Message::ImportResponse(response),
          )
          .await?;
        Ok(None)
      }
      outbound_message::Message::FileImportRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        let response = importers.file_import(&request).await?;
        self
          .handle_inbound_message(
            request.id,
            inbound_message::Message::FileImportResponse(response),
          )
          .await?;
        Ok(None)
      }
      outbound_message::Message::FunctionCallRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        // TODO
        Ok(None)
      }
      outbound_message::Message::Error(_) => todo!(),
      outbound_message::Message::LogEvent(_) => todo!(),
      outbound_message::Message::VersionResponse(_) => todo!(),
    }
  }
}
