#[derive(Debug, Default)]
pub struct PacketTransformer {
  packet: Packet,
}

impl PacketTransformer {
  pub fn encode(buf: Vec<u8>) -> Vec<u8> {
    let mut len = buf.len();
    if len == 0 {
      return vec![0];
    }

    let mut header = Vec::with_capacity(8);
    while len > 0 {
      header.push((if len > 0x7f { 0x80 } else { 0 }) | (len & 0x7f) as u8);
      len >>= 7;
    }
    [header, buf].concat()
  }

  pub fn decode(&mut self, buf: Vec<u8>) -> Vec<Vec<u8>> {
    let mut payloads = Vec::new();
    let mut decoded_bytes = 0;
    while decoded_bytes < buf.len() {
      decoded_bytes += self.packet.write(&buf[decoded_bytes..]);
      if self.packet.is_complete() && self.packet.payload.is_some() {
        payloads.push(self.packet.payload.take().unwrap());
        self.packet = Packet::default();
      }
    }
    payloads
  }
}

#[derive(Debug, Default)]
struct Packet {
  payload_len_bits: usize,
  payload_len: usize,
  payload_offset: usize,
  payload: Option<Vec<u8>>,
}

impl Packet {
  pub fn is_complete(&self) -> bool {
    self.payload.is_some() && self.payload_offset >= self.payload_len
  }

  pub fn write(&mut self, source: &[u8]) -> usize {
    if self.is_complete() {
      panic!("Cannot write to a completed Packet.");
    }

    let mut i = 0;
    if self.payload.is_none() {
      loop {
        let byte = source[i];
        self.payload_len += (byte as usize & 0x7f) << self.payload_len_bits;
        self.payload_len_bits += 7;
        i += 1;

        if byte <= 0x7f {
          self.payload = Some(vec![0; self.payload_len]);
          break;
        } else if i == source.len() {
          return i;
        } else {
        }
      }
    }

    let bytes_to_write = (self.payload.as_ref().unwrap().len()
      - self.payload_offset)
      .min(source.len() - i);
    self.payload.as_mut().unwrap().splice(
      self.payload_offset..self.payload_offset + bytes_to_write,
      source[i..i + bytes_to_write].to_vec(),
    );
    self.payload_offset += bytes_to_write;
    i + bytes_to_write
  }
}

#[cfg(test)]
mod tests {
  use std::vec;

  use super::*;

  #[test]
  fn encode_an_empty_message() {
    let encoded = PacketTransformer::encode(Vec::new());
    assert_eq!(encoded, vec![0]);
  }

  #[test]
  fn encode_a_message_of_length_1() {
    let encoded = PacketTransformer::encode(vec![123]);
    assert_eq!(encoded, vec![1, 123]);
  }

  #[test]
  fn encode_a_message_of_length_greater_than_256() {
    let encoded = PacketTransformer::encode([1; 300].to_vec());
    assert_eq!(encoded, [vec![172, 2], vec![1; 300]].concat());
  }

  #[test]
  fn encode_multiple_message() {
    // TODO: test write in
    let encoded1 = PacketTransformer::encode(vec![10]);
    let encoded2 = PacketTransformer::encode(vec![20, 30]);
    let encoded3 = PacketTransformer::encode(vec![40, 50, 60]);
    assert_eq!(encoded1, vec![1, 10]);
    assert_eq!(encoded2, vec![2, 20, 30]);
    assert_eq!(encoded3, vec![3, 40, 50, 60]);
  }

  #[test]
  fn decode_empty_message_a_single_chunk() {
    let mut pt = PacketTransformer::default();
    let packets = pt.decode(vec![0]);
    assert_eq!(packets, vec![vec![]]);
  }

  #[test]
  fn decode_empty_message_a_chunk_contains_more_data() {
    let mut pt = PacketTransformer::default();
    let packets = pt.decode(vec![0, 1, 100]);
    assert_eq!(packets, vec![vec![], vec![100]]);
  }

  #[test]
  fn decode_longer_message_a_single_chunk() {
    let mut pt = PacketTransformer::default();
    let packets = pt.decode(vec![4, 1, 2, 3, 4]);
    assert_eq!(packets, vec![vec![1, 2, 3, 4]]);
  }

  #[test]
  fn decode_longer_message_multiple_chunks() {
    let mut pt = PacketTransformer::default();
    let packets = [
      pt.decode(vec![172]),
      pt.decode(vec![2, 1]),
      pt.decode(vec![1; 299]),
    ]
    .concat();
    assert_eq!(packets, vec![vec![1; 300]]);
  }

  #[test]
  fn decode_longer_message_one_chunk_per_byte() {
    let mut pt = PacketTransformer::default();
    let buf = [vec![172, 2], vec![1; 300]].concat();
    let packets = buf
      .into_iter()
      .flat_map(|byte| pt.decode(vec![byte]))
      .collect::<Vec<_>>();
    assert_eq!(packets, vec![vec![1; 300]]);
  }

  #[test]
  fn decode_longer_message_a_chunk_that_contains_more_data() {
    let mut pt = PacketTransformer::default();
    let packets = pt.decode(vec![4, 1, 2, 3, 4, 1, 0]);
    assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![0]]);
  }

  #[test]
  fn decode_longer_message_a_full_chunk_of_length_greater_than_256() {
    let mut pt = PacketTransformer::default();
    let packets = pt.decode([vec![172, 2], vec![1; 300]].concat());
    assert_eq!(packets, vec![vec![1; 300]]);
  }

  #[test]
  fn decode_mulitple_message_a_single_chunk() {
    let mut pt = PacketTransformer::default();
    let packets = pt.decode(vec![4, 1, 2, 3, 4, 2, 101, 102]);
    assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![101, 102]]);
  }

  #[test]
  fn decode_mulitple_message_multiple_chunks() {
    let mut pt = PacketTransformer::default();
    let packets = [
      pt.decode(vec![4]),
      pt.decode(vec![1, 2, 3, 4, 172]),
      pt.decode([vec![2], vec![1; 300]].concat()),
    ]
    .concat();
    assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![1; 300]]);
  }

  #[test]
  fn decode_mulitple_message_one_chunk_per_byte() {
    let mut pt = PacketTransformer::default();
    let buf = [vec![4, 1, 2, 3, 4, 172, 2], vec![1; 300]].concat();
    let packets = buf
      .into_iter()
      .flat_map(|byte| pt.decode(vec![byte]))
      .collect::<Vec<_>>();
    assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![1; 300]]);
  }
}
