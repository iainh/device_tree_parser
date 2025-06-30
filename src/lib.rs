// ABOUTME: Device tree parser library using nom combinators
// ABOUTME: Provides parsing functionality for device tree files and formats

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

pub mod parser;

/// Main library interface for device tree parsing
pub struct DeviceTreeParser;

impl DeviceTreeParser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeviceTreeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = DeviceTreeParser::new();
        // Basic instantiation test
    }

    #[test]
    fn test_default_creation() {
        let parser = DeviceTreeParser::default();
        // Test default trait implementation
    }
}