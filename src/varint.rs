use std::io::{Read, Write};

pub fn read<R: Read>(readable: &mut R) -> Result<usize, std::io::Error> {
  let mut value = 0;
  let mut bits = 0;
  loop {
    let buf = &mut [0];
    readable.read(buf)?;
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
