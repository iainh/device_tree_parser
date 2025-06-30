// ABOUTME: Device tree parser library with zero-copy parsing and ergonomic APIs
// ABOUTME: Provides comprehensive DTB parsing for embedded systems and hardware discovery

//! # Device Tree Parser
//!
//! Parse Device Tree Blob (DTB) files with zero-copy performance and ergonomic APIs.
//! Designed for embedded systems with `no_std` compatibility.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # use device_tree_parser::{DeviceTreeParser, DtbError};
//! # fn main() -> Result<(), DtbError> {
//! // Load your DTB data  
//! let dtb_data = std::fs::read("path/to/your.dtb").unwrap();
//!
//! // Create parser and parse the device tree
//! let parser = DeviceTreeParser::new(&dtb_data);
//! let tree = parser.parse_tree()?;
//!
//! // Use ergonomic APIs (v0.3.0+)
//! for child in &tree {
//!     println!("Node: {}", child.name);
//!     
//!     // Access properties using Index trait
//!     if child.has_property("reg") {
//!         println!("Register: {}", child["reg"].value);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Zero-copy parsing**: Borrows from original DTB buffer for performance
//! - **Ergonomic APIs**: Index traits, `IntoIterator`, `TryFrom` conversions
//! - **`no_std` compatible**: Works in embedded environments with `alloc`
//! - **Type-safe**: Strong typing for device tree structures and properties
//! - **Real-world tested**: Validated against QEMU-generated DTB files
//!
//! ## Main Types
//!
//! - [`DeviceTreeParser`] - Main parser interface
//! - [`DeviceTreeNode`] - Device tree nodes with ergonomic access
//! - [`Property`] - Device tree properties with type-safe values
//! - [`PropertyValue`] - Strongly-typed property values
//! - [`DtbHeader`] - DTB file header information
//! - [`MemoryReservation`] - Memory reservation entries

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod dtb;

#[cfg(test)]
mod integration_tests;

// Re-export main types
pub use dtb::{
    AddressRange, AddressSpec, DeviceTreeNode, DeviceTreeParser, DtbError, DtbHeader, DtbToken,
    MemoryReservation, NodeIterator, Property, PropertyValue,
};

// Re-export utility functions
pub use dtb::tree::parse_address_from_bytes;

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
