// ABOUTME: DTB header structure definitions and parsing
// ABOUTME: Handles the 40-byte device tree blob header format

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
}