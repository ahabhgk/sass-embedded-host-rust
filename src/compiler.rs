use std::{ffi::OsStr, process::Stdio, sync::Mutex};

use futures::{channel::oneshot, future, pin_mut, StreamExt, TryStreamExt};
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
  stdout: Option<ChildStdout>,
  stdin: Option<ChildStdin>,
  // importers: ImporterRegistry,

  // pending_inbound_requests: RequestTracker,
  // pending_outbound_requests: RequestTracker,
}

impl Embedded {
  pub fn new(program: impl AsRef<OsStr>) -> Self {
    let mut child = Command::new(program)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();

    Self {
      stdout: child.stdout.take(),
      stdin: child.stdin.take(),
      // importers,
      // pending_inbound_requests: RequestTracker::new(),
      // pending_outbound_requests: RequestTracker::new(),
    }
  }

  // async fn write(&mut self, buf: &[u8]) -> Result<usize> {
  //   self.stdin.write(buf).await.map_err(|e| e.into())
  // }

  pub async fn send_compile_request(
    &mut self,
    request: CompileRequest,
    importers: ImporterRegistry,
  ) -> Result<CompileResponse> {
    let stdin = self.stdin.take().unwrap();
    let mut dispatcher = Dispatcher::new(stdin, importers);
    dispatcher.send_compile_request(request).await?;

    let stdout = self.stdout.take().unwrap();
    let reader = ReaderStream::new(BufReader::new(stdout))
      .map_err(|e| Error::from(e))
      .and_then(|buf| {
        let outbound = OutboundMessage::decode_length_delimited(buf).unwrap();
        future::ok(outbound.message.unwrap())
      })
      .try_filter_map(|m| async {
        dispatcher.handle_outbound_message(m).await
      });
    pin_mut!(reader);
    let r = reader.next().await.unwrap()?;
    Ok(r)
  }

  // async fn send_inbound_message(
  //   &mut self,
  //   id: u32,
  //   mut message: inbound_message::Message,
  // ) -> Result<()> {
  //   match &mut message {
  //     inbound_message::Message::CompileRequest(request) => {
  //       request.id = id;
  //       self.pending_inbound_requests.add(id);
  //     }
  //     inbound_message::Message::CanonicalizeResponse(response) => {
  //       response.id = id;
  //       self.pending_outbound_requests.resolve(id);
  //     }
  //     inbound_message::Message::ImportResponse(response) => {
  //       response.id = id;
  //       self.pending_outbound_requests.resolve(id);
  //     }
  //     inbound_message::Message::FileImportResponse(response) => {
  //       response.id = id;
  //       self.pending_outbound_requests.resolve(id);
  //     }
  //     inbound_message::Message::FunctionCallResponse(response) => {
  //       response.id = id;
  //       self.pending_outbound_requests.resolve(id);
  //     }
  //     _ => panic!("Unknown message type {message:?}"),
  //   };
  //   let inbound = InboundMessage::new(message);
  //   let buf = inbound.encode_length_delimited_to_vec();
  //   self.write(&buf).await?;
  //   Ok(())
  // }

  // async fn handle_outbound_message(
  //   &mut self,
  //   message: outbound_message::Message,
  // ) -> Result<Option<CompileResponse>> {
  //   match message {
  //     outbound_message::Message::CompileResponse(response) => {
  //       self.pending_inbound_requests.resolve(response.id);
  //       Ok(Some(response))
  //     }
  //     outbound_message::Message::CanonicalizeRequest(request) => {
  //       self.pending_outbound_requests.add(request.id);
  //       let response = self.importers.canonicalize(&request).await?;
  //       self
  //         .send_inbound_message(
  //           request.id,
  //           inbound_message::Message::CanonicalizeResponse(response),
  //         )
  //         .await?;
  //       Ok(None)
  //     }
  //     outbound_message::Message::ImportRequest(request) => {
  //       self.pending_outbound_requests.add(request.id);
  //       let response = self.importers.import(&request).await?;
  //       self
  //         .send_inbound_message(
  //           request.id,
  //           inbound_message::Message::ImportResponse(response),
  //         )
  //         .await?;
  //       Ok(None)
  //     }
  //     outbound_message::Message::FileImportRequest(request) => {
  //       self.pending_outbound_requests.add(request.id);
  //       let response = self.importers.file_import(&request).await?;
  //       self
  //         .send_inbound_message(
  //           request.id,
  //           inbound_message::Message::FileImportResponse(response),
  //         )
  //         .await?;
  //       Ok(None)
  //     }
  //     outbound_message::Message::FunctionCallRequest(request) => {
  //       self.pending_outbound_requests.add(request.id);
  //       // TODO
  //       Ok(None)
  //     }
  //     outbound_message::Message::Error(_) => todo!(),
  //     outbound_message::Message::LogEvent(_) => todo!(),
  //     outbound_message::Message::VersionResponse(_) => todo!(),
  //   }
  // }
}

pub struct Dispatcher {
  importers: ImporterRegistry,
  stdin: Mutex<ChildStdin>,
  pending_inbound_requests: RequestTracker,
  pending_outbound_requests: RequestTracker,
}

impl Dispatcher {
  pub fn new(stdin: ChildStdin, importers: ImporterRegistry) -> Self {
    Self {
      importers,
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
      _ => panic!("Unknown message type {message:?}"),
    };
    let inbound = InboundMessage::new(message);
    let buf = inbound.encode_length_delimited_to_vec();
    self.stdin.lock().unwrap().write(&buf).await?;
    Ok(())
  }

  async fn handle_outbound_message(
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
        // TODO
        Ok(None)
      }
      outbound_message::Message::Error(_) => todo!(),
      outbound_message::Message::LogEvent(_) => todo!(),
      outbound_message::Message::VersionResponse(_) => todo!(),
    }
  }
}
