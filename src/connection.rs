// use std::{ffi::OsStr, process::Stdio};

// use futures::StreamExt;
// use prost::Message as _;
// use tokio::{
//   io::{AsyncWriteExt, BufReader},
//   process::{ChildStdin, ChildStdout, Command},
// };
// use tokio_util::io::ReaderStream;

// use crate::{
//   api::StringOptions,
//   importer_registry::ImporterRegistry,
//   pb::{
//     inbound_message::{self, CompileRequest},
//     InboundMessage,
//   },
//   request_tracker::RequestTracker,
//   Result,
// };

// pub struct Connection {
//   pending_inbound_requests: RequestTracker,
//   pending_outbound_requests: RequestTracker,
// }

// pub struct Sender {
//   stdin: ChildStdin,
// }

// impl Sender {
//   pub fn new(stdin: ChildStdin) -> Self {
//     Self { stdin }
//   }

//   pub async fn send(&mut self, buf: &[u8]) -> Result<usize> {
//     self.stdin.write(buf).await.map_err(|e| e.into())
//   }
// }

// pub struct Receiver {
//   stdout: ChildStdout,
// }

// impl Receiver {
//   pub fn new(stdout: ChildStdout) -> Self {
//     Self { stdout }
//   }

//   pub async fn into_stream(&mut self) -> Result<Vec<u8>> {
//     self.stdout.read_to_end().await.map_err(|e| e.into())
//   }
// }

// impl Connection {
//   pub fn new(program: impl AsRef<OsStr>) -> (Self, Sender, Receiver) {
//     let mut child = Command::new(program)
//       .kill_on_drop(true)
//       .stdin(Stdio::piped())
//       .stdout(Stdio::piped())
//       .stderr(Stdio::piped())
//       .spawn()
//       .unwrap();
//     let sender = Sender::new(child.stdin.take().unwrap());
//     let receiver = Receiver::new(child.stdout.take().unwrap());
//     let conn = Self {
//       pending_inbound_requests: RequestTracker::new(),
//       pending_outbound_requests: RequestTracker::new(),
//     };
//     (conn, sender, receiver)
//   }

//   pub async fn compile_string(
//     &mut self,
//     source: String,
//     mut options: StringOptions,
//   ) -> Result<()> {
//     let base = options.get_options_mut();
//     let mut importers =
//       ImporterRegistry::new(base.importers.take(), base.load_paths.take());
//     let request = CompileRequest::with_string(source, &mut importers, options);
//     let id = self.pending_inbound_requests.next_id();
//     self
//       .send_inbound_message(
//         id,
//         inbound_message::Message::CompileRequest(request),
//       )
//       .await?;
//     Ok(())
//   }

//   async fn send_inbound_message(
//     &mut self,
//     id: u32,
//     mut message: inbound_message::Message,
//   ) -> Result<()> {
//     match &mut message {
//       inbound_message::Message::CompileRequest(request) => {
//         request.id = id;
//         self.pending_inbound_requests.add(id);
//       }
//       inbound_message::Message::CanonicalizeResponse(response) => {
//         response.id = id;
//         self.pending_outbound_requests.resolve(id);
//       }
//       inbound_message::Message::ImportResponse(response) => {
//         response.id = id;
//         self.pending_outbound_requests.resolve(id);
//       }
//       inbound_message::Message::FileImportResponse(response) => {
//         response.id = id;
//         self.pending_outbound_requests.resolve(id);
//       }
//       inbound_message::Message::FunctionCallResponse(response) => {
//         response.id = id;
//         self.pending_outbound_requests.resolve(id);
//       }
//       _ => panic!("Unknown message type {message:?}"),
//     };
//     let inbound = InboundMessage::new(message);
//     let buf = inbound.encode_length_delimited_to_vec();
//     self.write(&buf).await?;
//     Ok(())
//   }

//   async fn receive_outbound_message(&self) {
//     self.reader().map(|buf| buf);
//   }
// }
