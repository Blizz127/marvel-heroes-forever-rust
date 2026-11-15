//! Original byte preparation for the selected native context diagnostic.
//!
//! The fixed format, one-LF removal and 256-byte destination are grounded in
//! docs/REA_CONTEXT_DIAGNOSTIC.md. This does not select a current Environment,
//! bind a log sink, process printf conversions or issue a host/OS operation.

use std::ffi::{CStr, CString};
use std::fmt;

pub const CONTEXT_DIAGNOSTIC_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatError {
    /// Native reads message[length - 1]; an empty message is not admitted.
    EmptyMessage,
    LengthOverflow,
}
impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyMessage => "context diagnostic requires a nonempty message",
            Self::LengthOverflow => "context diagnostic length exceeds the supported address space",
        })
    }
}
impl std::error::Error for FormatError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormattedDiagnostic {
    value: CString,
    untruncated_len: usize,
}
impl FormattedDiagnostic {
    /// Only the initialized visible prefix and its terminator are exposed.
    pub fn as_c_str(&self) -> &CStr {
        &self.value
    }

    /// Native sum of the four selected byte-span lengths, before truncation.
    pub fn untruncated_len(&self) -> usize {
        self.untruncated_len
    }

    pub fn truncated(&self) -> bool {
        self.untruncated_len >= CONTEXT_DIAGNOSTIC_CAPACITY
    }
}

/// Prepare the supported fixed-format diagnostic as bytes.
///
/// The actual native inputs must select the nonnull context/log branch with
/// its formatting flag set. This API does not establish those identities.
/// The message must be nonempty; one terminal LF is removed, preserving CR
/// and any preceding LF. The optional context is the native nullable char
/// pointer's C-string contents, not an inferred file name or UTF-8 string.
///
/// Native copies the four spans in order into 256 bytes and then writes NUL
/// at min(full_length, 255). The result exposes exactly that visible prefix
/// plus NUL. No unused tail, percent substitution, encoding conversion or
/// logger side effect is modeled. Ordinary imported byte-copy behavior,
/// including its zero-length case, is part of the selected native contract.
pub fn format_with_context(
    message: &CStr,
    context: Option<&CStr>,
) -> Result<FormattedDiagnostic, FormatError> {
    let message = message.to_bytes();
    if message.is_empty() {
        return Err(FormatError::EmptyMessage);
    }
    let message = message.strip_suffix(b"\n").unwrap_or(message);
    let context = context.map_or(&[][..], CStr::to_bytes);
    let untruncated_len = message
        .len()
        .checked_add(context.len())
        .and_then(|length| length.checked_add(4))
        .ok_or(FormatError::LengthOverflow)?;
    let visible_len = untruncated_len.min(CONTEXT_DIAGNOSTIC_CAPACITY - 1);
    let mut output = Vec::with_capacity(visible_len + 1);
    for span in [message, b" : ", context, b"\n"] {
        let copy = span.len().min(visible_len - output.len());
        output.extend_from_slice(&span[..copy]);
    }
    output.push(0);
    // CStr inputs contain no interior NUL; the two literal spans do not either.
    let value = CString::from_vec_with_nul(output).expect("diagnostic spans are NUL-free");
    Ok(FormattedDiagnostic {
        value,
        untruncated_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_one_lf_and_keeps_raw_bytes_and_percent_sequences() {
        let message = CStr::from_bytes_with_nul(b"message%p\r\n\n\0").unwrap();
        let context = CStr::from_bytes_with_nul(b"\xff%08x\0").unwrap();
        let output = format_with_context(message, Some(context)).unwrap();
        assert_eq!(output.as_c_str().to_bytes_with_nul(), b"message%p\r\n : \xff%08x\n\0");
        assert!(!output.truncated());
    }

    #[test]
    fn null_and_empty_context_match_but_empty_message_is_not_admitted() {
        let message = CStr::from_bytes_with_nul(b"\n\0").unwrap();
        let empty = CStr::from_bytes_with_nul(b"\0").unwrap();
        let absent = format_with_context(message, None).unwrap();
        assert_eq!(absent.as_c_str().to_bytes_with_nul(), b" : \n\0");
        assert_eq!(absent, format_with_context(message, Some(empty)).unwrap());
        assert_eq!(format_with_context(empty, None), Err(FormatError::EmptyMessage));
    }

    #[test]
    fn exact_capacity_overwrites_last_byte_and_can_split_utf8() {
        let message = CString::new(vec![b'm'; 249]).unwrap();
        let context = CString::new([0xf0, 0x9f, 0x92, 0xa5]).unwrap();
        let output = format_with_context(&message, Some(&context)).unwrap();
        assert_eq!(output.untruncated_len(), 257);
        assert!(output.truncated());
        assert_eq!(output.as_c_str().to_bytes().len(), 255);
        assert_eq!(&output.as_c_str().to_bytes_with_nul()[249..], &[b' ', b':', b' ', 0xf0, 0x9f, 0x92, 0]);
        assert!(output.as_c_str().to_str().is_err());
        let boundary = CString::new(vec![b'x'; 252]).unwrap();
        let output = format_with_context(&boundary, None).unwrap();
        assert_eq!(output.untruncated_len(), 256);
        assert!(output.truncated());
        assert!(output.as_c_str().to_bytes().ends_with(b" : "));
    }
}
