//! Original bounded BC1/BC2/BC3 expansion to straight RGBA8.
//!
//! See docs/BC_FORMAT.md for format sources, deterministic rounding, and limits.

use std::io;

pub const MAX_RGBA_BYTES: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Bc1,
    Bc2,
    Bc3,
}

impl Format {
    pub const fn block_bytes(self) -> usize {
        match self {
            Self::Bc1 => 8,
            Self::Bc2 | Self::Bc3 => 16,
        }
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Expand one complete mip level. Bytes are row-major RGBA, top row first.
///
/// Width and height describe the logical image. Edge blocks are stored in full
/// and their out-of-image texels are discarded. Trailing compressed bytes are
/// rejected. No gamma conversion or alpha premultiplication is applied.
pub fn decode(format: Format, width: u32, height: u32, input: &[u8]) -> io::Result<Vec<u8>> {
    let width = usize::try_from(width).map_err(|_| invalid("width does not fit address space"))?;
    let height =
        usize::try_from(height).map_err(|_| invalid("height does not fit address space"))?;
    if width == 0 || height == 0 {
        return Err(invalid("BC image dimensions must be nonzero"));
    }
    let output_len = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .filter(|&n| n <= MAX_RGBA_BYTES)
        .ok_or_else(|| invalid("BC image exceeds the 256 MiB RGBA budget"))?;
    let columns = width.div_ceil(4);
    let rows = height.div_ceil(4);
    let block_bytes = format.block_bytes();
    let encoded_len = columns
        .checked_mul(rows)
        .and_then(|n| n.checked_mul(block_bytes))
        .ok_or_else(|| invalid("BC encoded size overflow"))?;
    if input.len() != encoded_len {
        return Err(invalid(
            "BC input length does not match the complete mip size",
        ));
    }

    let mut output = Vec::new();
    output
        .try_reserve_exact(output_len)
        .map_err(|_| invalid("BC output allocation failed"))?;
    output.resize(output_len, 0);
    for (block_index, block) in input.chunks_exact(block_bytes).enumerate() {
        let left = (block_index % columns) * 4;
        let top = (block_index / columns) * 4;
        let color_start = if format == Format::Bc1 { 0 } else { 8 };
        let color = &block[color_start..color_start + 8];
        let endpoint0 = u16::from_le_bytes([color[0], color[1]]);
        let endpoint1 = u16::from_le_bytes([color[2], color[3]]);
        let palette = color_palette(endpoint0, endpoint1, format == Format::Bc1);
        let indices = u32::from_le_bytes([color[4], color[5], color[6], color[7]]);
        let alpha = match format {
            Format::Bc1 => [255; 16],
            Format::Bc2 => explicit_alpha(block),
            Format::Bc3 => interpolated_alpha(block),
        };
        for (texel, &opacity) in alpha.iter().enumerate() {
            let x = left + texel % 4;
            let y = top + texel / 4;
            if x < width && y < height {
                let selector = ((indices >> (2 * texel)) & 3) as usize;
                let mut rgba = palette[selector];
                if format != Format::Bc1 {
                    rgba[3] = opacity;
                }
                let offset = (y * width + x) * 4;
                output[offset..offset + 4].copy_from_slice(&rgba);
            }
        }
    }
    Ok(output)
}

fn expand_565(value: u16) -> [u8; 4] {
    let red = ((value >> 11) & 31) as u8;
    let green = ((value >> 5) & 63) as u8;
    let blue = (value & 31) as u8;
    [
        (red << 3) | (red >> 2),
        (green << 2) | (green >> 4),
        (blue << 3) | (blue >> 2),
        255,
    ]
}

fn color_palette(a: u16, b: u16, may_be_transparent: bool) -> [[u8; 4]; 4] {
    let first = expand_565(a);
    let second = expand_565(b);
    let mut colors = [first, second, [0, 0, 0, 255], [0; 4]];
    for channel in 0..3 {
        let x = u16::from(first[channel]);
        let y = u16::from(second[channel]);
        if may_be_transparent && a <= b {
            colors[2][channel] = ((x + y) / 2) as u8;
        } else {
            // Integer rounding follows the documented Direct3D 9 examples.
            colors[2][channel] = ((2 * x + y + 1) / 3) as u8;
            colors[3][channel] = ((x + 2 * y + 1) / 3) as u8;
            colors[3][3] = 255;
        }
    }
    colors
}

fn explicit_alpha(block: &[u8]) -> [u8; 16] {
    let mut values = [0; 16];
    for (index, value) in values.iter_mut().enumerate() {
        *value = ((block[index / 2] >> ((index % 2) * 4)) & 15) * 17;
    }
    values
}

fn interpolated_alpha(block: &[u8]) -> [u8; 16] {
    let a = u16::from(block[0]);
    let b = u16::from(block[1]);
    let mut table = [block[0], block[1], 0, 0, 0, 0, 0, 255];
    let denominator: u16 = if a > b { 7 } else { 5 };
    for weight in 1..denominator {
        table[usize::from(weight + 1)] =
            (((denominator - weight) * a + weight * b + denominator / 2) / denominator) as u8;
    }
    let mut bits = 0_u64;
    for (shift, &byte) in block[2..8].iter().enumerate() {
        bits |= u64::from(byte) << (shift * 8);
    }
    let mut values = [0; 16];
    for (texel, value) in values.iter_mut().enumerate() {
        *value = table[((bits >> (texel * 3)) & 7) as usize];
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colors(a: u16, b: u16, selectors: u32) -> Vec<u8> {
        [
            a.to_le_bytes().as_slice(),
            b.to_le_bytes().as_slice(),
            selectors.to_le_bytes().as_slice(),
        ]
        .concat()
    }

    fn alpha_indices(indices: &[u8; 16]) -> [u8; 6] {
        let mut bits = 0_u64;
        for (texel, &selector) in indices.iter().enumerate() {
            bits |= u64::from(selector) << (texel * 3);
        }
        let bytes = bits.to_le_bytes();
        bytes[..6].try_into().unwrap()
    }

    #[test]
    fn bc1_color_selectors_and_transparency() {
        let decoded = decode(Format::Bc1, 4, 4, &colors(0xf800, 0x001f, 0xe4e4e4e4)).unwrap();
        assert_eq!(
            &decoded[..16],
            &[
                255, 0, 0, 255, 0, 0, 255, 255, 170, 0, 85, 255, 85, 0, 170, 255
            ]
        );
        assert!(decoded.chunks_exact(16).all(|row| row == &decoded[..16]));

        let decoded = decode(Format::Bc1, 4, 4, &colors(0, 0xffff, 0xe4e4e4e4)).unwrap();
        assert_eq!(
            &decoded[..16],
            &[
                0, 0, 0, 255, 255, 255, 255, 255, 127, 127, 127, 255, 0, 0, 0, 0
            ]
        );
        let equal = decode(Format::Bc1, 1, 1, &colors(0x07e0, 0x07e0, 3)).unwrap();
        assert_eq!(equal, [0, 0, 0, 0]);
    }

    #[test]
    fn bc2_alpha_nibbles_and_four_color_mode() {
        let mut encoded = vec![0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe];
        encoded.extend(colors(0, 0xffff, u32::MAX));
        let result = decode(Format::Bc2, 4, 4, &encoded).unwrap();
        for (index, rgba) in result.chunks_exact(4).enumerate() {
            assert_eq!(rgba, [170, 170, 170, (index * 17) as u8]);
        }
    }

    #[test]
    fn bc3_alpha_modes_and_little_endian_selector_stream() {
        let indices = [0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0];
        for (a, b, values) in [
            (255, 0, [255, 0, 219, 182, 146, 109, 73, 36]),
            (0, 255, [0, 255, 51, 102, 153, 204, 0, 255]),
            (91, 91, [91, 91, 91, 91, 91, 91, 0, 255]),
        ] {
            let mut encoded = vec![a, b];
            encoded.extend(alpha_indices(&indices));
            encoded.extend(colors(0, 0xffff, u32::MAX));
            let result = decode(Format::Bc3, 4, 4, &encoded).unwrap();
            for (texel, rgba) in result.chunks_exact(4).enumerate() {
                assert_eq!(rgba, [170, 170, 170, values[indices[texel] as usize]]);
            }
        }
    }

    #[test]
    fn partial_blocks_are_cropped_without_changing_row_stride() {
        let mut encoded = colors(0xf800, 0, 0);
        encoded.extend(colors(0x07e0, 0, 0));
        encoded.extend(colors(0x001f, 0, 0));
        encoded.extend(colors(0xffff, 0, 0));
        let result = decode(Format::Bc1, 5, 5, &encoded).unwrap();
        assert_eq!(result.len(), 100);
        for y in 0..5 {
            for x in 0..5 {
                let expected = match (x == 4, y == 4) {
                    (false, false) => [255, 0, 0, 255],
                    (true, false) => [0, 255, 0, 255],
                    (false, true) => [0, 0, 255, 255],
                    (true, true) => [255, 255, 255, 255],
                };
                assert_eq!(&result[(y * 5 + x) * 4..(y * 5 + x + 1) * 4], expected);
            }
        }
    }

    #[test]
    fn byte_counts_and_output_budgets_fail_before_allocation() {
        for format in [Format::Bc1, Format::Bc2, Format::Bc3] {
            let encoded = vec![0; format.block_bytes()];
            for short in 0..encoded.len() {
                assert!(decode(format, 1, 1, &encoded[..short]).is_err());
            }
            assert!(decode(format, 0, 1, &encoded).is_err());
            assert!(decode(format, 1, 0, &encoded).is_err());
            assert!(decode(format, 8, 4, &encoded).is_err());
            assert!(decode(format, u32::MAX, u32::MAX, &encoded).is_err());
            assert!(decode(format, MAX_RGBA_BYTES as u32, 1, &encoded).is_err());
            assert!(decode(format, 1, 1, &[encoded.as_slice(), &[0]].concat()).is_err());
        }
    }

    #[test]
    fn endpoint_bit_expansion_and_documented_rounding_are_explicit() {
        assert_eq!(expand_565(0x1843), [24, 8, 24, 255]);
        let palette = color_palette(0x0820, 0, false);
        assert_eq!(palette[0], [8, 4, 0, 255]);
        assert_eq!(palette[2], [5, 3, 0, 255]);
        assert_eq!(palette[3], [3, 1, 0, 255]);
    }
}
