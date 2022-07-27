use std::{ffi::OsStr, process::Stdio};

use futures::{future, pin_mut, stream, StreamExt, TryStreamExt};
use prost::Message;
use tokio::{
  io::BufReader,
  process::{Child, Command},
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
  child: Child,
}

impl Embedded {
  pub fn new(program: impl AsRef<OsStr>) -> Self {
    let child = Command::new(program)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();

    Self { child }
  }

  pub async fn compile(
    mut self,
    request: CompileRequest,
    importers: &ImporterRegistry,
    logger: &LoggerRegistry,
  ) -> Result<CompileResponse> {
    let stdin = self.child.stdin.take().unwrap();
    let mut dispatcher = Dispatcher::new(stdin, importers, logger);
    dispatcher.send_compile_request(request).await?;

    let stdout = self.child.stdout.take().unwrap();
    let mut pt = PacketTransformer::default();
    let reader = ReaderStream::new(BufReader::new(stdout))
      .map_err(Error::from)
      .flat_map(|res| match res {
        Ok(buf) => stream::iter(
          pt.decode(buf.to_vec())
            .into_iter()
            .map(Ok)
            .collect::<Vec<Result<Vec<u8>>>>(),
        ),
        Err(e) => stream::iter(vec![Err(e)]),
      })
      .and_then(|buf| {
        dbg!(buf.len(), buf.last());
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
