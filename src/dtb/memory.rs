// ABOUTME: Memory reservation block parsing for device tree blobs
// ABOUTME: Handles (address, size) pairs with 8-byte alignment requirements

use alloc::vec::Vec;
use super::error::DtbError;

/// Memory reservation entry with address and size
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservation {
    /// Physical address of reserved memory region
    pub address: u64,
    /// Size of reserved memory region
    pub size: u64,
}

impl MemoryReservation {
    /// Size of each reservation entry in bytes (address + size)
    pub const SIZE: usize = 16;
    
    /// Parse memory reservations from input bytes
    pub fn parse_all(input: &[u8]) -> Result<(&[u8], Vec<Self>), DtbError<&[u8]>> {
        let mut reservations = Vec::new();
        let mut offset = 0;
        
        // Ensure 8-byte alignment
        if (input.as_ptr() as usize) % 8 != 0 {
            return Err(DtbError::AlignmentError);
        }
        
        loop {
            // Check if we have enough bytes for another entry
            if offset + Self::SIZE > input.len() {
                return Err(DtbError::MalformedHeader);
            }
            
            // Parse address (bytes 0-7)
            let address = u64::from_be_bytes([
                input[offset], input[offset + 1], input[offset + 2], input[offset + 3],
                input[offset + 4], input[offset + 5], input[offset + 6], input[offset + 7]
            ]);
            offset += 8;
            
            // Parse size (bytes 8-15)
            let size = u64::from_be_bytes([
                input[offset], input[offset + 1], input[offset + 2], input[offset + 3],
                input[offset + 4], input[offset + 5], input[offset + 6], input[offset + 7]
            ]);
            offset += 8;
            
            // Check for terminating entry (0, 0)
            if address == 0 && size == 0 {
                break;
            }
            
            reservations.push(MemoryReservation { address, size });
        }
        
        Ok((&input[offset..], reservations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_memory_reservation_parse_empty() {
        let data = vec![
            // Terminating entry (0, 0)
            0, 0, 0, 0, 0, 0, 0, 0,  // address = 0
            0, 0, 0, 0, 0, 0, 0, 0,  // size = 0
        ];
        
        let result = MemoryReservation::parse_all(&data);
        assert!(result.is_ok());
        let (_, reservations) = result.unwrap();
        assert_eq!(reservations.len(), 0);
    }

    #[test]
    fn test_memory_reservation_parse_single() {
        let data = vec![
            // First entry: address=0x1000, size=0x2000
            0, 0, 0, 0, 0, 0, 0x10, 0,  // address = 0x1000
            0, 0, 0, 0, 0, 0, 0x20, 0,  // size = 0x2000
            // Terminating entry (0, 0)
            0, 0, 0, 0, 0, 0, 0, 0,     // address = 0
            0, 0, 0, 0, 0, 0, 0, 0,     // size = 0
        ];
        
        let result = MemoryReservation::parse_all(&data);
        assert!(result.is_ok());
        let (_, reservations) = result.unwrap();
        assert_eq!(reservations.len(), 1);
        assert_eq!(reservations[0].address, 0x1000);
        assert_eq!(reservations[0].size, 0x2000);
    }

    #[test]
    fn test_memory_reservation_parse_multiple() {
        let data = vec![
            // First entry: address=0x1000, size=0x2000
            0, 0, 0, 0, 0, 0, 0x10, 0,  // address = 0x1000
            0, 0, 0, 0, 0, 0, 0x20, 0,  // size = 0x2000
            // Second entry: address=0x3000, size=0x4000
            0, 0, 0, 0, 0, 0, 0x30, 0,  // address = 0x3000
            0, 0, 0, 0, 0, 0, 0x40, 0,  // size = 0x4000
            // Terminating entry (0, 0)
            0, 0, 0, 0, 0, 0, 0, 0,     // address = 0
            0, 0, 0, 0, 0, 0, 0, 0,     // size = 0
        ];
        
        let result = MemoryReservation::parse_all(&data);
        assert!(result.is_ok());
        let (_, reservations) = result.unwrap();
        assert_eq!(reservations.len(), 2);
        assert_eq!(reservations[0].address, 0x1000);
        assert_eq!(reservations[0].size, 0x2000);
        assert_eq!(reservations[1].address, 0x3000);
        assert_eq!(reservations[1].size, 0x4000);
    }
}