use std::{
  process::Stdio,
  sync::{Arc, Mutex},
  thread,
};

use futures::{
  future::{self, BoxFuture},
  stream, Stream, StreamExt,
};
use prost::Message as _;
use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
  process::Command,
};
// use tokio_stream::StreamExt as _;
use tokio_util::io::ReaderStream;

use crate::{
  api::{
    CompileResult, Exception, Result, StringOptions,
    StringOptionsWithoutImporter,
  },
  compiler::Embedded,
  compiler_path,
  importer_registry::ImporterRegistry,
  packet_transformer::PacketTransformer,
  pb::{
    inbound_message::{
      compile_request::{Input, StringInput},
      CompileRequest, Message,
    },
    outbound_message::{
      compile_response::{self, CompileSuccess},
      CompileResponse,
    },
    InboundMessage, OutboundMessage, OutputStyle, Syntax,
  },
};

pub async fn compile_string(
  source: String,
  mut options: StringOptions,
) -> Result<CompileResult> {
  let base = options.get_options_mut();
  let mut importers =
    ImporterRegistry::new(base.importers.take(), base.load_paths.take());
  let request = CompileRequest::with_string(source, &mut importers, options);
  let mut embedded = Embedded::new(compiler_path::compiler_path().unwrap());
  let response = embedded.send_compile_request(request, importers).await?;
  match response.result.unwrap() {
    compile_response::Result::Success(success) => {
      let css = success.css;
      let source_map = success.source_map;
      let loaded_urls = success.loaded_urls;
      Ok(CompileResult {
        css,
        source_map: Some(source_map),
        loaded_urls,
      })
    }
    compile_response::Result::Failure(failure) => {
      Err(Exception::new(failure).into())
    }
  }
}

#[tokio::test]
async fn test_compile_string() {
  let res = compile_string(
    ".foo {a: b}".to_string(),
    StringOptions::WithoutImporter(StringOptionsWithoutImporter::default()),
  )
  .await
  .unwrap();
  dbg!(res);
}

#[tokio::test]
async fn t_compile_string() {
  let source = ".a { color: red; }".to_string();
  let mut string = StringInput::default();
  string.source = source;
  string.set_syntax(Syntax::Scss);
  let mut request = CompileRequest::default();
  request.set_style(OutputStyle::Expanded);
  request.input = Some(Input::String(string));
  // request.id = 0;
  let mut inbound_message = InboundMessage::default();
  inbound_message.message = Some(Message::CompileRequest(request));
  let buf = inbound_message.encode_length_delimited_to_vec();
  // let buf = PacketTransformer::encode(buf);

  let path = compiler_path::compiler_path().unwrap();
  let mut child = Command::new(path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  dbg!(&buf);
  child
    .stdin
    .as_mut()
    .unwrap()
    .write_all(buf.as_ref())
    .await
    .unwrap();
  let stdout = child.stdout.take().unwrap();
  let reader = BufReader::new(stdout);
  let stream = ReaderStream::new(reader);
  stream
    // .flat_map(|buf| {
    //   let ps = pt.decode(buf.unwrap().to_vec());
    //   stream::iter(ps)
    // })
    .map(|buf| {
      let buf = buf.unwrap().to_vec();
      let m = OutboundMessage::decode_length_delimited(buf.as_ref()).unwrap();
      m
    })
    .for_each(|m| {
      dbg!(m);
      future::ready(())
    })
    .await;
}
