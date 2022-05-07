// pub fn encode(message: protocol::InboundMessage) -> Vec<u8> {
//   // let
//   todo!()
// }

use crate::pb::sass_embedded_protocol::{
  inbound_message::{CompileRequest, Message},
  InboundMessage,
};
use prost::Message as _;

#[test]
fn tt() {
  let mut m = InboundMessage::default();
  m.message = Some(Message::CompileRequest(CompileRequest::default()));
  let buf = m.encode_length_delimited_to_vec();

  let i = InboundMessage::decode_length_delimited(buf.as_ref()).unwrap();
  dbg!(i, buf);
}
