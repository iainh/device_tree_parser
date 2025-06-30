// ABOUTME: Error types for device tree blob parsing
// ABOUTME: Provides no_std compatible error handling for DTB operations

use core::fmt;

/// Main error type for DTB parsing operations
#[derive(Debug, Clone, PartialEq)]
pub enum DtbError {
    /// Invalid magic number in DTB header
    InvalidMagic,
    /// Malformed header structure
    MalformedHeader,
    /// Invalid token in structure block
    InvalidToken,
    /// Alignment error in data parsing
    AlignmentError,
}

impl fmt::Display for DtbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DtbError::InvalidMagic => write!(f, "Invalid magic number in DTB header"),
            DtbError::MalformedHeader => write!(f, "Malformed DTB header structure"),
            DtbError::InvalidToken => write!(f, "Invalid token in structure block"),
            DtbError::AlignmentError => write!(f, "Data alignment error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DtbError {}
