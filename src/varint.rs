use std::io::{Read, Write};

pub fn read<R: Read>(readable: &mut R) -> Result<usize, std::io::Error> {
  let mut value = 0;
  let mut bits = 0;
  loop {
    let buf = &mut [0];
    readable.read_exact(buf)?;
    let byte = buf[0];
    value |= ((byte & 0x7f) as usize) << bits;
    bits += 7;
    if byte < 0x80 {
      break;
    }
  }
  Ok(value)
}

pub fn write<W: Write>(
  writeable: &mut W,
  mut value: usize,
) -> Result<(), std::io::Error> {
  let mut bytes = Vec::<u8>::new();
  while value >= 0x80 {
    bytes.push(0x80 | (value & 0x7f) as u8);
    value >>= 7;
  }
  bytes.push(value as u8);
  writeable.write_all(&bytes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_an_empty_message() {
    let buf = Vec::new();
    let len = read(&mut buf.as_slice()).unwrap();
    assert_eq!(len, 0);
  }

  #[test]
  fn read_a_message_of_length_1() {
    let buf = vec![123];
    let len = read(&mut buf.as_slice()).unwrap();
    assert_eq!(len, 1);
  }

  #[test]
  fn encode_a_message_of_length_greater_than_256() {
    let buf = vec![1; 300];
    let len = read(&mut buf.as_slice()).unwrap();
    dbg!(len);
    // assert_eq!(len, 1);
    // let encoded = PacketTransformer::encode([1; 300].to_vec());
    // assert_eq!(encoded, [vec![172, 2], vec![1; 300]].concat());
  }

  // #[test]
  // fn encode_multiple_message() {
  //   // TODO: test write in
  //   let encoded1 = PacketTransformer::encode(vec![10]);
  //   let encoded2 = PacketTransformer::encode(vec![20, 30]);
  //   let encoded3 = PacketTransformer::encode(vec![40, 50, 60]);
  //   assert_eq!(encoded1, vec![1, 10]);
  //   assert_eq!(encoded2, vec![2, 20, 30]);
  //   assert_eq!(encoded3, vec![3, 40, 50, 60]);
  // }

  // #[test]
  // fn decode_empty_message_a_single_chunk() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = pt.decode(vec![0]);
  //   assert_eq!(packets, vec![vec![]]);
  // }

  // #[test]
  // fn decode_empty_message_a_chunk_contains_more_data() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = pt.decode(vec![0, 1, 100]);
  //   assert_eq!(packets, vec![vec![], vec![100]]);
  // }

  // #[test]
  // fn decode_longer_message_a_single_chunk() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = pt.decode(vec![4, 1, 2, 3, 4]);
  //   assert_eq!(packets, vec![vec![1, 2, 3, 4]]);
  // }

  // #[test]
  // fn decode_longer_message_multiple_chunks() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = [
  //     pt.decode(vec![172]),
  //     pt.decode(vec![2, 1]),
  //     pt.decode(vec![1; 299]),
  //   ]
  //   .concat();
  //   assert_eq!(packets, vec![vec![1; 300]]);
  // }

  // #[test]
  // fn decode_longer_message_one_chunk_per_byte() {
  //   let mut pt = PacketTransformer::default();
  //   let buf = [vec![172, 2], vec![1; 300]].concat();
  //   let packets = buf
  //     .into_iter()
  //     .flat_map(|byte| pt.decode(vec![byte]))
  //     .collect::<Vec<_>>();
  //   assert_eq!(packets, vec![vec![1; 300]]);
  // }

  // #[test]
  // fn decode_longer_message_a_chunk_that_contains_more_data() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = pt.decode(vec![4, 1, 2, 3, 4, 1, 0]);
  //   assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![0]]);
  // }

  // #[test]
  // fn decode_longer_message_a_full_chunk_of_length_greater_than_256() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = pt.decode([vec![172, 2], vec![1; 300]].concat());
  //   assert_eq!(packets, vec![vec![1; 300]]);
  // }

  // #[test]
  // fn decode_mulitple_message_a_single_chunk() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = pt.decode(vec![4, 1, 2, 3, 4, 2, 101, 102]);
  //   assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![101, 102]]);
  // }

  // #[test]
  // fn decode_mulitple_message_multiple_chunks() {
  //   let mut pt = PacketTransformer::default();
  //   let packets = [
  //     pt.decode(vec![4]),
  //     pt.decode(vec![1, 2, 3, 4, 172]),
  //     pt.decode([vec![2], vec![1; 300]].concat()),
  //   ]
  //   .concat();
  //   assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![1; 300]]);
  // }

  // #[test]
  // fn decode_mulitple_message_one_chunk_per_byte() {
  //   let mut pt = PacketTransformer::default();
  //   let buf = [vec![4, 1, 2, 3, 4, 172, 2], vec![1; 300]].concat();
  //   let packets = buf
  //     .into_iter()
  //     .flat_map(|byte| pt.decode(vec![byte]))
  //     .collect::<Vec<_>>();
  //   assert_eq!(packets, vec![vec![1, 2, 3, 4], vec![1; 300]]);
  // }
}
