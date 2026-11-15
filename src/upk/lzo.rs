//! Original, bounded reader for raw LZO1X version-0 blocks.
//!
//! The byte format was studied from the Linux kernel's LZO format document;
//! no external decoder implementation is included. UE3 chunk framing belongs
//! to the caller. Versioned LZO-RLE streams are deliberately unsupported.

use std::io;

/// Resource ceiling for one raw block, independent of its enclosing package.
pub const MAX_BLOCK_BYTES: usize = 64 * 1024 * 1024;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Decode exactly one raw LZO1X v0 stream into the declared number of bytes.
///
/// Rejects truncated or trailing input, invalid dictionary distances, output
/// overruns, length overflow, and unsupported versioned streams. Match copies
/// permit overlap, as required for repeated-byte dictionary runs.
pub fn decompress(input: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
    if expected_size > MAX_BLOCK_BYTES {
        return Err(invalid("LZO block exceeds the 64 MiB output limit"));
    }
    if input.first() == Some(&17) && input.len() >= 5 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "versioned LZO streams are unsupported",
        ));
    }
    let mut decoder = Decoder {
        input,
        position: 0,
        output: Vec::new(),
        expected_size,
    };
    decoder
        .output
        .try_reserve_exact(expected_size)
        .map_err(|_| io::Error::new(io::ErrorKind::OutOfMemory, "cannot allocate LZO output"))?;

    // The initial literal instruction has its own encoding because no past
    // output exists yet. Four means "at least four", the only distinction
    // needed by the short-match instruction.
    let mut literal_state = 0;
    if input.first().is_some_and(|&byte| byte > 17) {
        let count = usize::from(decoder.byte()? - 17);
        decoder.literals(count)?;
        literal_state = count.min(4);
    }

    loop {
        let opcode = decoder.byte()?;
        let (length, distance, following_literals) = match opcode {
            0..=15 if literal_state == 0 => {
                let count = decoder.length(opcode, 15, 3)?;
                decoder.literals(count)?;
                literal_state = 4;
                continue;
            }
            0..=15 => {
                let high = usize::from(decoder.byte()?);
                let low = usize::from(opcode >> 2);
                let (length, base_distance) = if literal_state == 4 {
                    (3, 2049)
                } else {
                    (2, 1)
                };
                (
                    length,
                    base_distance + (high << 2) + low,
                    usize::from(opcode & 3),
                )
            }
            16..=31 => {
                let length = decoder.length(opcode, 7, 2)?;
                let operand = usize::from(decoder.word()?);
                let distance = 16384 + (usize::from(opcode & 8) << 11) + (operand >> 2);
                if distance == 16384 {
                    // Canonical v0 terminator: length three, zero distance
                    // operand, no trailing bytes and exactly the promised data.
                    if length != 3 || operand != 0 {
                        return Err(invalid("invalid LZO end marker"));
                    }
                    if decoder.position != input.len() {
                        return Err(invalid("trailing bytes after LZO end marker"));
                    }
                    if decoder.output.len() != expected_size {
                        return Err(invalid("LZO output differs from declared size"));
                    }
                    return Ok(decoder.output);
                }
                (length, distance, operand & 3)
            }
            32..=63 => {
                let length = decoder.length(opcode, 31, 2)?;
                let operand = usize::from(decoder.word()?);
                (length, 1 + (operand >> 2), operand & 3)
            }
            64..=255 => {
                let length = usize::from(opcode >> 5) + 1;
                let high = usize::from(decoder.byte()?);
                let distance = 1 + (high << 3) + usize::from((opcode >> 2) & 7);
                (length, distance, usize::from(opcode & 3))
            }
        };
        decoder.dictionary(length, distance)?;
        decoder.literals(following_literals)?;
        literal_state = following_literals;
    }
}

struct Decoder<'a> {
    input: &'a [u8],
    position: usize,
    output: Vec<u8>,
    expected_size: usize,
}

impl Decoder<'_> {
    fn byte(&mut self) -> io::Result<u8> {
        let byte = *self
            .input
            .get(self.position)
            .ok_or_else(|| invalid("truncated LZO stream"))?;
        self.position += 1;
        Ok(byte)
    }

    fn word(&mut self) -> io::Result<u16> {
        let low = self.byte()?;
        let high = self.byte()?;
        Ok(u16::from_le_bytes([low, high]))
    }

    fn length(&mut self, opcode: u8, mask: u8, bias: usize) -> io::Result<usize> {
        let mut length = usize::from(opcode & mask);
        if length == 0 {
            length = usize::from(mask);
            loop {
                let extension = self.byte()?;
                length = length
                    .checked_add(if extension == 0 {
                        255
                    } else {
                        usize::from(extension)
                    })
                    .ok_or_else(|| invalid("LZO length overflow"))?;
                if length > MAX_BLOCK_BYTES {
                    return Err(invalid("LZO length exceeds block limit"));
                }
                if extension != 0 {
                    break;
                }
            }
        }
        length
            .checked_add(bias)
            .ok_or_else(|| invalid("LZO length overflow"))
    }

    fn output_room(&self, count: usize) -> io::Result<()> {
        if count > self.expected_size.saturating_sub(self.output.len()) {
            return Err(invalid("LZO instruction exceeds declared output size"));
        }
        Ok(())
    }

    fn literals(&mut self, count: usize) -> io::Result<()> {
        self.output_room(count)?;
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| invalid("LZO input range overflow"))?;
        let data = self
            .input
            .get(self.position..end)
            .ok_or_else(|| invalid("truncated LZO literal run"))?;
        self.output.extend_from_slice(data);
        self.position = end;
        Ok(())
    }

    fn dictionary(&mut self, length: usize, distance: usize) -> io::Result<()> {
        self.output_room(length)?;
        if distance == 0 || distance > self.output.len() {
            return Err(invalid("LZO dictionary distance precedes available output"));
        }
        for _ in 0..length {
            let byte = self.output[self.output.len() - distance];
            self.output.push(byte);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const END: [u8; 3] = [17, 0, 0];

    #[test]
    fn empty_and_initial_literals() {
        assert_eq!(decompress(&END, 0).unwrap(), b"");
        assert_eq!(
            decompress(&[22, b'h', b'e', b'l', b'l', b'o', 17, 0, 0], 5).unwrap(),
            b"hello"
        );
    }

    #[test]
    fn dictionary_copy_overlaps_its_own_output() {
        assert_eq!(
            decompress(&[18, b'x', 39, 0, 0, 17, 0, 0], 10).unwrap(),
            b"xxxxxxxxxx"
        );
        assert_eq!(decompress(&[18, b'x', 0, 0, 17, 0, 0], 3).unwrap(), b"xxx");
    }

    #[test]
    fn immediate_literals_change_next_match_interpretation() {
        let encoded = [21, b'a', b'b', b'c', b'd', 65, 0, b'x', 4, 0, 17, 0, 0];
        assert_eq!(decompress(&encoded, 10).unwrap(), b"abcddddxdx");
    }

    #[test]
    fn long_literal_run_then_distant_three_byte_match() {
        let mut encoded = vec![0];
        encoded.extend([0; 7]);
        encoded.push(246);
        let literal: Vec<u8> = (0..2049).map(|n| (n % 251) as u8).collect();
        encoded.extend(&literal);
        encoded.extend([0, 0]);
        encoded.extend(END);
        let mut expected = literal;
        expected.extend([0, 1, 2]);
        assert_eq!(decompress(&encoded, 2052).unwrap(), expected);
    }

    #[test]
    fn distant_long_form_match() {
        let mut encoded = vec![0];
        encoded.extend([0; 64]);
        encoded.push(47);
        let literal: Vec<u8> = (0..16385).map(|n| (n % 251) as u8).collect();
        encoded.extend(&literal);
        encoded.extend([17, 4, 0]);
        encoded.extend(END);
        let mut expected = literal;
        expected.extend([0, 1, 2]);
        assert_eq!(decompress(&encoded, 16388).unwrap(), expected);
    }

    #[test]
    fn rejects_truncation_bad_distances_and_size_mismatch() {
        let encoded = [22, b'h', b'e', b'l', b'l', b'o', 17, 0, 0];
        for end in 0..encoded.len() {
            assert!(decompress(&encoded[..end], 5).is_err(), "prefix {end}");
        }
        assert!(decompress(&[18, b'x', 0, 1, 17, 0, 0], 3).is_err());
        assert!(decompress(&encoded, 4).is_err());
        assert!(decompress(&encoded, 6).is_err());
        assert!(decompress(&[0, 0, 0], 1000).is_err());
        assert!(decompress(&END, MAX_BLOCK_BYTES + 1).is_err());
    }

    #[test]
    fn rejects_trailing_data_noncanonical_end_and_new_versions() {
        assert!(decompress(&[17, 0, 0, 1], 0).is_err());
        assert!(decompress(&[18, 0, 0], 0).is_err());
        assert!(decompress(&[17, 1, 0], 0).is_err());
        assert_eq!(
            decompress(&[17, 1, 17, 0, 0], 0).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
