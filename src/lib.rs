// ABOUTME: Device tree parser library using nom combinators
// ABOUTME: Provides parsing functionality for device tree files and formats

#![no_std]

extern crate alloc;

pub mod dtb;

#[cfg(test)]
mod integration_tests;

// Re-export main types
pub use dtb::{
    DeviceTreeNode, DeviceTreeParser, DtbError, DtbHeader, DtbToken, MemoryReservation,
    NodeIterator, Property, PropertyValue,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let data = &[0u8; 40];
        let parser = DeviceTreeParser::new(data);
        assert_eq!(parser.data().len(), 40);
    }
}
