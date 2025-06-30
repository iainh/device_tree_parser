// ABOUTME: DTB header structure definitions and parsing
// ABOUTME: Handles the 40-byte device tree blob header format

use super::error::DtbError;

/// DTB header structure (40 bytes total)
#[derive(Debug, Clone, PartialEq)]
pub struct DtbHeader {
    /// Magic number (should be 0xd00dfeed)
    pub magic: u32,
    /// Total size of the DTB
    pub totalsize: u32,
    /// Offset to structure block
    pub off_dt_struct: u32,
    /// Offset to strings block
    pub off_dt_strings: u32,
    /// Offset to memory reservation block
    pub off_mem_rsvmap: u32,
    /// Version of the DTB format
    pub version: u32,
    /// Last compatible version
    pub last_comp_version: u32,
    /// Boot CPU ID
    pub boot_cpuid_phys: u32,
    /// Size of strings block
    pub size_dt_strings: u32,
    /// Size of structure block
    pub size_dt_struct: u32,
}

impl DtbHeader {
    /// DTB magic number constant
    pub const MAGIC: u32 = 0xd00d_feed;

    /// Header size in bytes
    pub const SIZE: usize = 40;

    /// Parse DTB header from input bytes
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
