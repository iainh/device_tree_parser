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

    /// Invalid address cell specification.
    ///
    /// Address cells must be between 1 and 4 (typically 1 or 2).
    /// This error occurs when `#address-cells` property contains an
    /// invalid value outside this range.
    InvalidAddressCells(u32),

    /// Invalid size cell specification.
    ///
    /// Size cells must be between 0 and 4 (typically 1 or 2).
    /// This error occurs when `#size-cells` property contains an
    /// invalid value outside this range.
    InvalidSizeCells(u32),

    /// Address translation error.
    ///
    /// Occurs when an address cannot be translated between bus domains.
    /// This can happen when no matching range is found or when address
    /// arithmetic would overflow.
    AddressTranslationError(u64),

    /// Invalid ranges property format.
    ///
    /// The ranges property must contain entries that are a multiple of
    /// (`child_address_cells` + `address_cells` + `size_cells`) * 4 bytes.
    /// This error indicates malformed ranges data.
    InvalidRangesFormat,

    /// Translation cycle detected.
    ///
    /// Occurs when multi-level address translation encounters a circular
    /// reference in the device tree hierarchy, which would cause infinite
    /// recursion.
    TranslationCycle,

    /// Maximum translation depth exceeded.
    ///
    /// Occurs when multi-level address translation exceeds the maximum
    /// allowed recursion depth, preventing potential stack overflow.
    MaxTranslationDepthExceeded,
}

impl fmt::Display for DtbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DtbError::InvalidMagic => write!(f, "Invalid magic number in DTB header"),
            DtbError::MalformedHeader => write!(f, "Malformed DTB header structure"),
            DtbError::InvalidToken => write!(f, "Invalid token in structure block"),
            DtbError::AlignmentError => write!(f, "Data alignment error"),
            DtbError::InvalidAddressCells(cells) => {
                write!(f, "Invalid #address-cells value: {cells} (must be 1-4)")
            }
            DtbError::InvalidSizeCells(cells) => {
                write!(f, "Invalid #size-cells value: {cells} (must be 0-4)")
            }
            DtbError::AddressTranslationError(addr) => {
                write!(f, "Cannot translate address 0x{addr:x}")
            }
            DtbError::InvalidRangesFormat => {
                write!(f, "Invalid ranges property format")
            }
            DtbError::TranslationCycle => {
                write!(f, "Translation cycle detected in device tree hierarchy")
            }
            DtbError::MaxTranslationDepthExceeded => {
                write!(f, "Maximum translation depth exceeded")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DtbError {}
