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
  Result,
};

pub struct Embedded {
  reader: ReaderStream<BufReader<ChildStdout>>,
  stdin: ChildStdin,

  pending_inbound_requests: RequestTracker,
  pending_outbound_requests: RequestTracker,
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

    let this = Self {
      reader,
      stdin,
      pending_inbound_requests: RequestTracker::new(),
      pending_outbound_requests: RequestTracker::new(),
    };

    reader
      .and_then(|buf| {
        let outbound = OutboundMessage::decode_length_delimited(buf).unwrap();
        future::ok(outbound.message.unwrap())
      })
      .try_filter_map(|message| {
        future::ok(this.handle_outbound_message(message))
      });

    this
  }

  async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.stdin.write(buf).await.map_err(|e| e.into())
  }

  fn reader(&self) -> &ReaderStream<BufReader<ChildStdout>> {
    &self.reader
  }

  pub async fn compile_string(
    &mut self,
    source: String,
    mut options: StringOptions,
  ) -> Result<CompileResult> {
    let base = options.get_options_mut();
    let mut importers =
      ImporterRegistry::new(base.importers.take(), base.load_paths.take());
    let request = CompileRequest::with_string(source, &mut importers, options);
    let id = self.pending_inbound_requests.next_id();
    self
      .send_inbound_message(
        id,
        inbound_message::Message::CompileRequest(request),
      )
      .await?;
    Ok(())
  }

  async fn send_inbound_message(
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

  fn handle_outbound_message(
    &mut self,
    message: outbound_message::Message,
  ) -> Option<CompileResponse> {
    match message {
      outbound_message::Message::CompileResponse(response) => {
        self.pending_inbound_requests.resolve(response.id);
        Some(response)
      }
      outbound_message::Message::Error(_) => todo!(),
      outbound_message::Message::LogEvent(_) => todo!(),
      outbound_message::Message::CanonicalizeRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        // TODO
        None
      }
      outbound_message::Message::ImportRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        // TODO
        None
      }
      outbound_message::Message::FileImportRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        // TODO
        None
      }
      outbound_message::Message::FunctionCallRequest(request) => {
        self.pending_outbound_requests.add(request.id);
        // TODO
        None
      }
      outbound_message::Message::VersionResponse(_) => todo!(),
    }
  }
}
