// ABOUTME: Error types for device tree blob parsing
// ABOUTME: Provides no_std compatible error handling for DTB operations

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
