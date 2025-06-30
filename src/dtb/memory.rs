// ABOUTME: Memory reservation block parsing for device tree blobs
// ABOUTME: Handles (address, size) pairs with 8-byte alignment requirements

use super::error::DtbError;
use alloc::vec::Vec;

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
    pub fn parse_all(input: &[u8]) -> Result<(&[u8], Vec<Self>), DtbError> {
        // Ensure 8-byte alignment
        if (input.as_ptr() as usize) % 8 != 0 {
            return Err(DtbError::AlignmentError);
        }

        let mut reservations = Vec::new();
        let mut chunks = input.chunks_exact(Self::SIZE);

        for chunk in &mut chunks {
            // Parse address and size using array slicing
            let address_bytes: [u8; 8] = chunk[0..8]
                .try_into()
                .map_err(|_| DtbError::MalformedHeader)?;
            let size_bytes: [u8; 8] = chunk[8..16]
                .try_into()
                .map_err(|_| DtbError::MalformedHeader)?;

            let address = u64::from_be_bytes(address_bytes);
            let size = u64::from_be_bytes(size_bytes);

            // Check for terminating entry (0, 0)
            if address == 0 && size == 0 {
                break;
            }

            reservations.push(MemoryReservation { address, size });
        }

        // Calculate remaining input after parsing complete reservation entries
        let consumed = reservations.len() * Self::SIZE + Self::SIZE; // +SIZE for terminating entry
        let remaining = if consumed <= input.len() {
            &input[consumed..]
        } else {
            &input[input.len()..]
        };

        Ok((remaining, reservations))
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
            0, 0, 0, 0, 0, 0, 0, 0, // address = 0
            0, 0, 0, 0, 0, 0, 0, 0, // size = 0
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
            0, 0, 0, 0, 0, 0, 0x10, 0, // address = 0x1000
            0, 0, 0, 0, 0, 0, 0x20, 0, // size = 0x2000
            // Terminating entry (0, 0)
            0, 0, 0, 0, 0, 0, 0, 0, // address = 0
            0, 0, 0, 0, 0, 0, 0, 0, // size = 0
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
            0, 0, 0, 0, 0, 0, 0x10, 0, // address = 0x1000
            0, 0, 0, 0, 0, 0, 0x20, 0, // size = 0x2000
            // Second entry: address=0x3000, size=0x4000
            0, 0, 0, 0, 0, 0, 0x30, 0, // address = 0x3000
            0, 0, 0, 0, 0, 0, 0x40, 0, // size = 0x4000
            // Terminating entry (0, 0)
            0, 0, 0, 0, 0, 0, 0, 0, // address = 0
            0, 0, 0, 0, 0, 0, 0, 0, // size = 0
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
