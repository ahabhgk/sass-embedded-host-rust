use channel::Channel;

mod channel;
mod compiler;
mod connection;
mod dispatcher;
mod pb;
mod varint;

#[test]
fn version_smoke() {
  let mut ch = Channel::new("/Users/bytedance/Codes/sass-embedded-host-rust/dart-sass-embedded/dart-sass-embedded");
  let conn = ch.connect();
  let response = conn.version_request().unwrap();
  dbg!(response);
}
