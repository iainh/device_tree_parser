// ABOUTME: Error types for device tree blob parsing
// ABOUTME: Provides no_std compatible error handling using nom's error system

use nom::error::{ErrorKind, ParseError};

/// Main error type for DTB parsing operations
#[derive(Debug, Clone, PartialEq)]
pub enum DtbError<I> {
    /// Invalid magic number in DTB header
    InvalidMagic,
    /// Malformed header structure
    MalformedHeader,
    /// Invalid token in structure block
    InvalidToken,
    /// Alignment error in data parsing
    AlignmentError,
    /// Nom parsing error
    ParseError(I, ErrorKind),
}

impl<I> ParseError<I> for DtbError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        DtbError::ParseError(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<nom::Err<nom::error::Error<I>>> for DtbError<I> {
    fn from(err: nom::Err<nom::error::Error<I>>) -> Self {
        match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => DtbError::ParseError(e.input, e.code),
            nom::Err::Incomplete(_) => {
                DtbError::ParseError(unsafe { core::mem::zeroed() }, ErrorKind::Complete)
            }
        }
    }
}
