// ABOUTME: Core DTB parser implementation using nom combinators
// ABOUTME: Provides the main DeviceTreeParser struct and parsing logic

/// Main device tree parser struct
#[derive(Debug)]
pub struct DeviceTreeParser<'a> {
    data: &'a [u8],
}

impl<'a> DeviceTreeParser<'a> {
    /// Create a new parser from DTB data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Get the underlying data slice
    pub fn data(&self) -> &[u8] {
        self.data
    }
}
