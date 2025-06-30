// ABOUTME: DTB header structure definitions and parsing
// ABOUTME: Handles the 40-byte device tree blob header format

use super::error::DtbError;

/// Device Tree Blob header containing file metadata and block layout.
///
/// Fixed 40-byte structure at the beginning of every DTB file. Contains essential
/// information for parsing the file including block offsets, sizes, and version
/// information.
///
/// # Layout
///
/// The header follows this exact layout (all fields are big-endian u32):
/// ```text
/// Offset | Field              | Description
/// -------|--------------------|-----------------------------------------
/// 0x00   | magic              | Magic number (0xd00dfeed)
/// 0x04   | totalsize          | Total DTB file size in bytes
/// 0x08   | off_dt_struct      | Offset to structure block
/// 0x0C   | off_dt_strings     | Offset to strings block  
/// 0x10   | off_mem_rsvmap     | Offset to memory reservation block
/// 0x14   | version            | DTB format version
/// 0x18   | last_comp_version  | Last compatible DTB version
/// 0x1C   | boot_cpuid_phys    | Physical ID of boot CPU
/// 0x20   | size_dt_strings    | Size of strings block in bytes
/// 0x24   | size_dt_struct     | Size of structure block in bytes
/// ```
///
/// # Examples
///
/// ```rust
/// # use device_tree_parser::{DeviceTreeParser, DtbHeader, DtbError};
/// # fn example() -> Result<(), DtbError> {
/// # let dtb_data = vec![0u8; 64]; // Mock data
/// let parser = DeviceTreeParser::new(&dtb_data);
/// let header = parser.parse_header()?;
///
/// // Validate magic number
/// if header.magic == DtbHeader::MAGIC {
///     println!("Valid DTB file");
/// }
///
/// // Check version compatibility
/// if header.version >= 17 {
///     println!("Modern DTB format (v{})", header.version);
/// }
///
/// // Examine file layout
/// println!("DTB size: {} bytes", header.totalsize);
/// println!("Structure block: {} bytes at offset 0x{:x}",
///     header.size_dt_struct, header.off_dt_struct);
/// println!("Strings block: {} bytes at offset 0x{:x}",
///     header.size_dt_strings, header.off_dt_strings);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DtbHeader {
    /// Magic number identifying this as a DTB file (must be 0xd00dfeed).
    pub magic: u32,
    /// Total size of the entire DTB file in bytes.
    pub totalsize: u32,
    /// Byte offset from start of file to the structure block.
    pub off_dt_struct: u32,
    /// Byte offset from start of file to the strings block.
    pub off_dt_strings: u32,
    /// Byte offset from start of file to the memory reservation block.
    pub off_mem_rsvmap: u32,
    /// DTB format version number (typically 17 for modern files).
    pub version: u32,
    /// Last DTB version that this file is compatible with.
    pub last_comp_version: u32,
    /// Physical CPU ID of the boot processor.
    pub boot_cpuid_phys: u32,
    /// Size of the strings block in bytes.
    pub size_dt_strings: u32,
    /// Size of the structure block in bytes.
    pub size_dt_struct: u32,
}

impl DtbHeader {
    /// DTB magic number constant
    pub const MAGIC: u32 = 0xd00d_feed;

    /// Header size in bytes
    pub const SIZE: usize = 40;

    /// Parse DTB header from input bytes
    ///
    /// # Errors
    ///
    /// Returns `DtbError::MalformedHeader` if input is too short or contains invalid data.
    /// Returns `DtbError::InvalidMagic` if the magic number is incorrect.
    ///
    /// # Panics
    ///
    /// Panics if internal slice operations fail due to data corruption.
    pub fn parse(input: &[u8]) -> Result<(&[u8], Self), DtbError> {
        if input.len() < Self::SIZE {
            return Err(DtbError::MalformedHeader);
        }

        // Helper function to read a big-endian u32 from a 4-byte slice
        let read_be_u32 = |bytes: &[u8]| -> u32 {
            u32::from_be_bytes(bytes.try_into().expect("slice should be exactly 4 bytes"))
        };

        // Parse all header fields using chunked slices
        let mut chunks = input.chunks_exact(4);

        let magic = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        if magic != Self::MAGIC {
            return Err(DtbError::InvalidMagic);
        }

        let totalsize = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let off_dt_struct = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let off_dt_strings = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let off_mem_rsvmap = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let version = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let last_comp_version = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let boot_cpuid_phys = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let size_dt_strings = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);
        let size_dt_struct = read_be_u32(chunks.next().ok_or(DtbError::MalformedHeader)?);

        let header = DtbHeader {
            magic,
            totalsize,
            off_dt_struct,
            off_dt_strings,
            off_mem_rsvmap,
            version,
            last_comp_version,
            boot_cpuid_phys,
            size_dt_strings,
            size_dt_struct,
        };

        Ok((&input[Self::SIZE..], header))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_header_parse_valid() {
        let mut header_data = vec![0u8; 40];
        // Magic number in big-endian
        header_data[0..4].copy_from_slice(&0xd00dfeedu32.to_be_bytes());
        // Total size
        header_data[4..8].copy_from_slice(&1024u32.to_be_bytes());

        let result = DtbHeader::parse(&header_data);
        assert!(result.is_ok());
        let (_, header) = result.unwrap();
        assert_eq!(header.magic, DtbHeader::MAGIC);
        assert_eq!(header.totalsize, 1024);
    }

    #[test]
    fn test_header_parse_invalid_magic() {
        let mut header_data = vec![0u8; 40];
        // Wrong magic number
        header_data[0..4].copy_from_slice(&0x12345678u32.to_be_bytes());

        let result = DtbHeader::parse(&header_data);
        assert!(result.is_err());
    }
}
