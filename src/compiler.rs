use std::{ffi::OsStr, process::Stdio};

use futures::{future, pin_mut, StreamExt, TryStreamExt};
use prost::Message;
use tokio::{
  io::BufReader,
  process::{ChildStdin, ChildStdout, Command},
};
use tokio_util::io::ReaderStream;

use crate::{
  dispatcher::Dispatcher,
  importer_registry::ImporterRegistry,
  pb::{
    inbound_message::CompileRequest, outbound_message::CompileResponse,
    OutboundMessage,
  },
  Error, Result, logger_registry::LoggerRegistry,
};

pub struct Embedded {
  stdout: Option<ChildStdout>,
  stdin: Option<ChildStdin>,
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
    }
  }

  pub async fn send_compile_request(
    &mut self,
    request: CompileRequest,
    importers: ImporterRegistry,
    logger: LoggerRegistry,
  ) -> Result<CompileResponse> {
    let stdin = self.stdin.take().unwrap();
    let mut dispatcher = Dispatcher::new(stdin, importers, logger);
    dispatcher.send_compile_request(request).await?;

    let stdout = self.stdout.take().unwrap();
    let reader = ReaderStream::new(BufReader::new(stdout))
      .map_err(|io_err| Error::from(io_err))
      .and_then(|buf| {
        let outbound = OutboundMessage::decode_length_delimited(buf).unwrap();
        future::ok(outbound.message.unwrap())
      })
      .try_filter_map(|m| async {
        dispatcher.handle_outbound_message(m).await
      });
    pin_mut!(reader);
    reader.next().await.unwrap()
  }
}
