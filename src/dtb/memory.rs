// ABOUTME: Memory reservation block parsing for device tree blobs
// ABOUTME: Handles (address, size) pairs with 8-byte alignment requirements

use super::error::DtbError;
use alloc::vec::Vec;

/// Memory reservation entry specifying regions that must not be used by the OS.
///
/// Commonly used in embedded systems to protect regions used by firmware,
/// bootloaders, or hardware that cannot be relocated. Each reservation specifies
/// a physical address range.
///
/// # Format
///
/// Each reservation is 16 bytes:
/// - 8 bytes: 64-bit physical address (big-endian)
/// - 8 bytes: 64-bit size in bytes (big-endian)
///
/// The reservation list is terminated by an entry with both address
/// and size set to zero.
///
/// # Examples
///
/// ```rust
/// # use device_tree_parser::{DeviceTreeParser, DtbError};
/// # fn example() -> Result<(), DtbError> {
/// # let dtb_data = vec![0u8; 64]; // Mock data
/// let parser = DeviceTreeParser::new(&dtb_data);
/// let reservations = parser.parse_memory_reservations()?;
///
/// for (i, reservation) in reservations.iter().enumerate() {
///     println!("Reservation {}: 0x{:016x} - 0x{:016x}",
///         i,
///         reservation.address,
///         reservation.address + reservation.size
///     );
///     
///     // Check if this overlaps with our intended memory usage
///     let our_start = 0x40000000u64;
///     let our_end = 0x48000000u64;
///     let res_end = reservation.address + reservation.size;
///     
///     if reservation.address < our_end && res_end > our_start {
///         println!("  ⚠️  Overlaps with our memory region!");
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservation {
    /// Physical address of the start of the reserved memory region.
    pub address: u64,
    /// Size of the reserved memory region in bytes.
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
