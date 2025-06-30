// ABOUTME: Error types for device tree blob parsing
// ABOUTME: Provides no_std compatible error handling for DTB operations

use core::fmt;

/// Comprehensive error type for Device Tree Blob parsing operations.
///
/// Covers all possible failures during DTB parsing, from file format issues
/// to data alignment problems. All errors implement `std::error::Error` when
/// the `std` feature is enabled.
///
/// # Examples
///
/// ```rust
/// # use device_tree_parser::{DeviceTreeParser, DtbError};
/// # fn example() {
/// let invalid_data = vec![0u8; 10]; // Too small for valid DTB
/// let parser = DeviceTreeParser::new(&invalid_data);
///
/// match parser.parse_header() {
///     Ok(header) => println!("Valid DTB with version {}", header.version),
///     Err(DtbError::InvalidMagic) => println!("File is not a valid DTB"),
///     Err(DtbError::MalformedHeader) => println!("DTB header is corrupted"),
///     Err(e) => println!("Other error: {}", e),
/// }
/// # }
/// ```
///
/// # Error Recovery
///
/// Some errors may be recoverable depending on use case:
/// - `InvalidMagic`: File is not a DTB, try other formats
/// - `MalformedHeader`: File may be truncated or corrupted
/// - `InvalidToken`: Specific node/property may be malformed
/// - `AlignmentError`: Data corruption or non-standard formatting
#[derive(Debug, Clone, PartialEq)]
pub enum DtbError {
    /// Invalid magic number in DTB header (expected 0xd00dfeed).
    ///
    /// This typically indicates that the file is not a valid Device Tree Blob,
    /// or the data is corrupted. The magic number is the first 4 bytes of
    /// every DTB file.
    InvalidMagic,

    /// Malformed or corrupted DTB header structure.
    ///
    /// The DTB header contains critical metadata about file layout. This error
    /// occurs when header fields contain invalid values, such as offsets
    /// pointing outside the file or impossibly large sizes.
    MalformedHeader,

    /// Invalid or unexpected token in the structure block.
    ///
    /// The DTB structure block uses specific token values to represent nodes,
    /// properties, and tree structure. This error indicates corruption or
    /// non-standard formatting in the structure data.
    InvalidToken,

    /// Data alignment error during parsing.
    ///
    /// DTB format requires specific alignment for different data types.
    /// This error occurs when data is not properly aligned, typically
    /// indicating file corruption or truncation.
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
