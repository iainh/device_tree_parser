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
    pub const MAGIC: u32 = 0xd00dfeed;
    
    /// Header size in bytes
    pub const SIZE: usize = 40;
    
    /// Parse DTB header from input bytes
    pub fn parse(input: &[u8]) -> Result<(&[u8], Self), DtbError<&[u8]>> {
        if input.len() < Self::SIZE {
            return Err(DtbError::MalformedHeader);
        }
        
        // Manual big-endian parsing to avoid nom type issues
        let mut offset = 0;
        
        // Parse magic number (bytes 0-3)
        let magic = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        if magic != Self::MAGIC {
            return Err(DtbError::InvalidMagic);
        }
        
        // Parse totalsize (bytes 4-7)
        let totalsize = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse off_dt_struct (bytes 8-11)
        let off_dt_struct = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse off_dt_strings (bytes 12-15)
        let off_dt_strings = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse off_mem_rsvmap (bytes 16-19)
        let off_mem_rsvmap = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse version (bytes 20-23)
        let version = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse last_comp_version (bytes 24-27)
        let last_comp_version = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse boot_cpuid_phys (bytes 28-31)
        let boot_cpuid_phys = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse size_dt_strings (bytes 32-35)
        let size_dt_strings = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;
        
        // Parse size_dt_struct (bytes 36-39)
        let size_dt_struct = u32::from_be_bytes([
            input[offset], input[offset + 1], input[offset + 2], input[offset + 3]
        ]);
        offset += 4;

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

        Ok((&input[offset..], header))
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