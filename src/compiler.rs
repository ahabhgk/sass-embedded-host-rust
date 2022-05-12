use std::{ffi::OsStr, process::Stdio};

use futures::{future, pin_mut, stream, StreamExt, TryStreamExt};
use prost::Message;
use tokio::{
  io::BufReader,
  process::{ChildStdin, ChildStdout, Command},
};
use tokio_util::io::ReaderStream;

use crate::{
  dispatcher::Dispatcher,
  importer_registry::ImporterRegistry,
  logger_registry::LoggerRegistry,
  packet_transformer::PacketTransformer,
  pb::{
    inbound_message::CompileRequest, outbound_message::CompileResponse,
    OutboundMessage,
  },
  Error, Result,
};

pub struct Embedded {
  stdout: ChildStdout,
  stdin: ChildStdin,
}

impl Embedded {
  pub fn new(program: impl AsRef<OsStr>) -> Self {
    let child = Command::new(program)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();

    Self {
      stdout: child.stdout.unwrap(),
      stdin: child.stdin.unwrap(),
    }
  }

  pub async fn compile(
    self,
    request: CompileRequest,
    importers: &ImporterRegistry,
    logger: &LoggerRegistry,
  ) -> Result<CompileResponse> {
    let stdin = self.stdin;
    let mut dispatcher = Dispatcher::new(stdin, &importers, &logger);
    dispatcher.send_compile_request(request).await?;

    let stdout = self.stdout;
    let mut pt = PacketTransformer::default();
    // TODO: refactor these shits
    let reader = ReaderStream::new(BufReader::new(stdout))
      .map_err(|io_err| Error::from(io_err))
      .flat_map(|res| match res {
        Ok(buf) => stream::iter(
          pt.decode(buf.to_vec())
            .into_iter()
            .map(|b| Ok(b))
            .collect::<Vec<Result<Vec<u8>>>>(),
        ),
        Err(e) => stream::iter(vec![Err(e)]),
      })
      .and_then(|buf| {
        let outbound = OutboundMessage::decode(buf.as_ref()).unwrap();
        future::ok(outbound.message.unwrap())
      })
      .try_filter_map(|m| async {
        dispatcher.handle_outbound_message(m).await
      });
    pin_mut!(reader);
    reader.next().await.unwrap()
  }
}
