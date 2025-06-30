// ABOUTME: Core DTB parser implementation using nom combinators
// ABOUTME: Provides the main DeviceTreeParser struct and parsing logic

use super::error::DtbError;
use super::header::DtbHeader;
use super::memory::MemoryReservation;
use super::tokens::DtbToken;
use super::tree::{DeviceTreeNode, parse_node_name, parse_property_data};
use alloc::vec::Vec;

/// High-performance Device Tree Blob (DTB) parser with zero-copy parsing.
///
/// Provides comprehensive interface for parsing DTB files commonly used in embedded
/// systems for hardware description. Uses zero-copy parsing to minimize memory
/// allocations and maximize performance.
///
/// # Examples
///
/// ## Basic Parsing
///
/// ```rust
/// # use device_tree_parser::{DeviceTreeParser, DtbError};
/// # fn example() -> Result<(), DtbError> {
/// // Load DTB data (typically from file system or embedded in binary)
/// let dtb_data = std::fs::read("my_device.dtb").unwrap();
/// let parser = DeviceTreeParser::new(&dtb_data);
///
/// // Parse the device tree structure
/// let tree = parser.parse_tree()?;
/// println!("Root node has {} children", tree.children.len());
/// # Ok(())
/// # }
/// ```
///
/// ## Hardware Discovery
///
/// ```rust
/// # use device_tree_parser::{DeviceTreeParser, DtbError};
/// # fn example() -> Result<(), DtbError> {
/// # let dtb_data = vec![0u8; 64]; // Mock data
/// let parser = DeviceTreeParser::new(&dtb_data);
///
/// // Find UART devices for serial communication
/// let uart_addresses = parser.uart_addresses()?;
/// for (i, addr) in uart_addresses.iter().enumerate() {
///     println!("UART {}: 0x{:08x}", i, addr);
/// }
///
/// // Get CPU timing information
/// if let Some(freq) = parser.timebase_frequency()? {
///     println!("CPU timebase: {} Hz", freq);
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Memory Layout Analysis
///
/// ```rust
/// # use device_tree_parser::{DeviceTreeParser, DtbError};
/// # fn example() -> Result<(), DtbError> {
/// # let dtb_data = vec![0u8; 64]; // Mock data
/// let parser = DeviceTreeParser::new(&dtb_data);
///
/// // Check for memory reservations
/// let reservations = parser.parse_memory_reservations()?;
/// for reservation in reservations {
///     println!("Reserved: 0x{:016x} - 0x{:016x}",
///         reservation.address,
///         reservation.address + reservation.size
///     );
/// }
///
/// // Discover memory-mapped I/O regions
/// let mmio_regions = parser.discover_mmio_regions()?;
/// println!("Found {} MMIO regions", mmio_regions.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct DeviceTreeParser<'a> {
    data: &'a [u8],
}

impl<'a> DeviceTreeParser<'a> {
    /// Creates a new parser from raw DTB data.
    ///
    /// Borrows the DTB data for zero-copy parsing. The data must remain valid for
    /// the lifetime of the parser and any structures parsed from it.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw DTB file data as a byte slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::DeviceTreeParser;
    /// // From file system (in real code)
    /// # let dtb_data = vec![0u8; 100]; // Mock data for example
    /// # /*
    /// let dtb_data = std::fs::read("device.dtb").unwrap();
    /// # */
    /// let parser = DeviceTreeParser::new(&dtb_data);
    ///
    /// // From embedded data in your binary (in real code)
    /// # /*
    /// const EMBEDDED_DTB: &[u8] = include_bytes!("path/to/your.dtb");
    /// let parser = DeviceTreeParser::new(EMBEDDED_DTB);
    /// # */
    /// ```
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Returns a reference to the underlying DTB data.
    ///
    /// Provides access to the raw DTB bytes, useful for debugging
    /// or passing the data to other parsers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::DeviceTreeParser;
    /// let dtb_data = vec![0u8; 100];
    /// let parser = DeviceTreeParser::new(&dtb_data);
    /// assert_eq!(parser.data().len(), 100);
    /// ```
    #[must_use]
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Parses and returns the DTB file header.
    ///
    /// Contains metadata about the file structure including version information,
    /// block offsets, and sizes. Typically the first step in DTB analysis.
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if the header is malformed or has an invalid magic number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    /// let header = parser.parse_header()?;
    ///
    /// println!("DTB version: {}", header.version);
    /// println!("Total size: {} bytes", header.totalsize);
    /// println!("Boot CPU: {}", header.boot_cpuid_phys);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_header(&self) -> Result<DtbHeader, DtbError> {
        let (_remaining, header) = DtbHeader::parse(self.data)?;
        Ok(header)
    }

    /// Parses and returns all memory reservation entries.
    ///
    /// Memory reservations specify regions of physical memory that should not
    /// be used by the operating system for general allocation. Common in embedded
    /// systems where certain memory regions are reserved for firmware, hardware
    /// buffers, or other special purposes.
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if the reservation block is malformed.
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
    ///     println!("Reservation {}: 0x{:016x} - 0x{:016x} (size: {} bytes)",
    ///         i,
    ///         reservation.address,
    ///         reservation.address + reservation.size,
    ///         reservation.size
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_memory_reservations(&self) -> Result<Vec<MemoryReservation>, DtbError> {
        let header = self.parse_header()?;
        let reservation_data = &self.data[header.off_mem_rsvmap as usize..];
        let (_remaining, reservations) = MemoryReservation::parse_all(reservation_data)?;
        Ok(reservations)
    }

    /// Parses and returns the complete device tree structure.
    ///
    /// Main parsing function that builds the entire device tree hierarchy starting
    /// from the root node. The returned tree supports ergonomic access patterns
    /// including indexing, iteration, and type-safe property value extraction.
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if the structure is malformed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # use std::convert::TryFrom;
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    /// let tree = parser.parse_tree()?;
    ///
    /// // Access root properties
    /// if let Some(model) = tree.prop_string("model") {
    ///     println!("Device model: {}", model);
    /// }
    ///
    /// // Iterate through child nodes using ergonomic API
    /// for child in &tree {
    ///     println!("Child node: {}", child.name);
    ///     
    ///     // Use Index trait for property access
    ///     if child.has_property("reg") {
    ///         println!("  Register: {}", child["reg"].value);
    ///     }
    ///     
    ///     // Type-safe property value extraction
    ///     if let Some(prop) = child.find_property("reg") {
    ///         if let Ok(values) = Vec::<u32>::try_from(&prop.value) {
    ///             println!("  Register values: {:?}", values);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_tree(&self) -> Result<DeviceTreeNode<'a>, DtbError> {
        let header = self.parse_header()?;

        let struct_block_start = header.off_dt_struct as usize;
        let struct_block_end = struct_block_start + header.size_dt_struct as usize;
        let strings_block_start = header.off_dt_strings as usize;

        if struct_block_start >= self.data.len()
            || struct_block_end > self.data.len()
            || strings_block_start >= self.data.len()
        {
            return Err(DtbError::MalformedHeader);
        }

        let struct_block = &self.data[struct_block_start..struct_block_end];
        let strings_block = &self.data[strings_block_start..];

        Self::parse_structure_block(struct_block, strings_block)
    }

    /// Discovers UART device base addresses from the device tree.
    ///
    /// Searches for common UART device types and extracts their base addresses
    /// from the `reg` property. Useful for setting up serial communication in
    /// embedded systems.
    ///
    /// Searches for these compatible strings:
    /// - `ns16550a`, `ns16550` - PC-style 16550 UARTs
    /// - `arm,pl011` - ARM `PrimeCell` UART
    /// - `arm,sbsa-uart` - ARM Server Base System Architecture UART
    /// - `snps,dw-apb-uart` - Synopsys `DesignWare` APB UART
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if parsing fails.
    ///
    /// # Returns
    ///
    /// Returns a vector of UART base addresses. An empty vector indicates no UART devices were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    /// let uart_addresses = parser.uart_addresses()?;
    ///
    /// for (i, addr) in uart_addresses.iter().enumerate() {
    ///     println!("UART {}: base address 0x{:08x}", i, addr);
    /// }
    ///
    /// // Use first UART for system console
    /// if let Some(&console_addr) = uart_addresses.first() {
    ///     println!("Console UART at: 0x{:08x}", console_addr);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn uart_addresses(&self) -> Result<Vec<u64>, DtbError> {
        let root = self.parse_tree()?;
        let mut addresses = Vec::new();

        // Look for common UART compatible strings
        let uart_compatibles = [
            "ns16550a",
            "ns16550",
            "arm,pl011",
            "arm,sbsa-uart",
            "snps,dw-apb-uart",
        ];

        for compatible in &uart_compatibles {
            let uart_nodes = root.find_compatible_nodes(compatible);
            for node in uart_nodes {
                if let Some(reg) = node.prop_u32_array("reg")
                    && reg.len() >= 2
                {
                    // First cell is typically the address
                    addresses.push(u64::from(reg[0]));
                }
            }
        }

        Ok(addresses)
    }

    /// Retrieves the CPU timebase frequency from the device tree.
    ///
    /// Timebase frequency is used by CPU timers and is critical for accurate timing
    /// in embedded systems. Searches the `/cpus` node and individual CPU nodes for
    /// the `timebase-frequency` property.
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if parsing fails.
    ///
    /// # Returns
    ///
    /// Returns `Some(frequency)` if found, `None` if no timebase frequency is specified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    ///
    /// match parser.timebase_frequency()? {
    ///     Some(freq) => {
    ///         println!("CPU timebase: {} Hz", freq);
    ///         println!("Timer resolution: {:.2} ns", 1_000_000_000.0 / freq as f64);
    ///     }
    ///     None => println!("No timebase frequency found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn timebase_frequency(&self) -> Result<Option<u32>, DtbError> {
        let root = self.parse_tree()?;

        // Look in /cpus node first
        if let Some(cpus_node) = root.find_node("/cpus") {
            if let Some(freq) = cpus_node.prop_u32("timebase-frequency") {
                return Ok(Some(freq));
            }

            // Check individual CPU nodes
            for cpu in cpus_node {
                if let Some(freq) = cpu.prop_u32("timebase-frequency") {
                    return Ok(Some(freq));
                }
            }
        }

        Ok(None)
    }

    /// Discovers memory-mapped I/O (MMIO) regions from the device tree.
    ///
    /// Traverses all device nodes and extracts address/size pairs from their `reg`
    /// properties. MMIO regions represent hardware devices mapped into the system's
    /// physical address space.
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if parsing fails.
    ///
    /// # Returns
    ///
    /// Returns a vector of `(address, size)` tuples representing MMIO regions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    /// let mmio_regions = parser.discover_mmio_regions()?;
    ///
    /// for (i, (addr, size)) in mmio_regions.iter().enumerate() {
    ///     println!("MMIO Region {}: 0x{:08x} - 0x{:08x} (size: {} bytes)",
    ///         i, addr, addr + size, size);
    /// }
    ///
    /// // Find regions larger than 1MB
    /// let large_regions: Vec<_> = mmio_regions
    ///     .iter()
    ///     .filter(|(_, size)| *size > 1024 * 1024)
    ///     .collect();
    /// println!("Found {} large MMIO regions", large_regions.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover_mmio_regions(&self) -> Result<Vec<(u64, u64)>, DtbError> {
        let root = self.parse_tree()?;
        let mut regions = Vec::new();

        // Traverse all nodes and collect reg properties
        for node in root.iter_nodes() {
            if let Some(reg) = node.prop_u32_array("reg") {
                // Parse reg property as address/size pairs
                let mut i = 0;
                while i + 1 < reg.len() {
                    let address = u64::from(reg[i]);
                    let size = u64::from(reg[i + 1]);
                    regions.push((address, size));
                    i += 2;
                }
            }
        }

        Ok(regions)
    }

    /// Finds a device tree node by its absolute path.
    ///
    /// Device tree paths use Unix-style notation starting from the root (`/`).
    /// Provides convenient access to specific nodes when you know their location
    /// in the tree hierarchy.
    ///
    /// # Arguments
    ///
    /// * `path` - Absolute path to the node (e.g., `/cpus/cpu@0`, `/chosen`)
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if parsing fails.
    ///
    /// # Returns
    ///
    /// Returns `Some(node)` if found, `None` if the path doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    ///
    /// // Find specific system nodes
    /// if let Some(chosen) = parser.find_node("/chosen")? {
    ///     if let Some(bootargs) = chosen.prop_string("bootargs") {
    ///         println!("Boot arguments: {}", bootargs);
    ///     }
    /// }
    ///
    /// // Find CPU information
    /// if let Some(cpu0) = parser.find_node("/cpus/cpu@0")? {
    ///     if let Some(compatible) = cpu0.prop_string("compatible") {
    ///         println!("CPU type: {}", compatible);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_node(&self, path: &str) -> Result<Option<DeviceTreeNode<'a>>, DtbError> {
        let root = self.parse_tree()?;
        Ok(root.find_node(path).cloned())
    }

    /// Finds all device tree nodes with a specific compatible string.
    ///
    /// The `compatible` property lists the devices that a node is compatible with,
    /// typically in most-specific to least-specific order. Searches for nodes that
    /// contain the specified string in their compatible property.
    ///
    /// # Arguments
    ///
    /// * `compatible` - Compatible string to search for (e.g., `"arm,pl011"`)
    ///
    /// # Errors
    ///
    /// Returns [`DtbError`] if parsing fails.
    ///
    /// # Returns
    ///
    /// Returns a vector of matching nodes. An empty vector indicates no matching nodes were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use device_tree_parser::{DeviceTreeParser, DtbError};
    /// # fn example() -> Result<(), DtbError> {
    /// # let dtb_data = vec![0u8; 64]; // Mock data
    /// let parser = DeviceTreeParser::new(&dtb_data);
    ///
    /// // Find all ARM PL011 UART devices
    /// let uart_nodes = parser.find_compatible_nodes("arm,pl011")?;
    /// for (i, node) in uart_nodes.iter().enumerate() {
    ///     println!("UART {}: {}", i, node.name);
    ///     if let Some(reg) = node.prop_u32_array("reg") {
    ///         println!("  Base address: 0x{:08x}", reg[0]);
    ///     }
    /// }
    ///
    /// // Find all Virtio devices
    /// let virtio_nodes = parser.find_compatible_nodes("virtio,mmio")?;
    /// println!("Found {} Virtio devices", virtio_nodes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_compatible_nodes(
        &self,
        compatible: &str,
    ) -> Result<Vec<DeviceTreeNode<'a>>, DtbError> {
        let root = self.parse_tree()?;
        let nodes = root.find_compatible_nodes(compatible);
        Ok(nodes.into_iter().cloned().collect())
    }

    /// Parse the structure block to build the device tree
    fn parse_structure_block(
        struct_block: &'a [u8],
        strings_block: &'a [u8],
    ) -> Result<DeviceTreeNode<'a>, DtbError> {
        parse_device_tree_iterative(struct_block, strings_block)
    }
}

/// Parse device tree structure using an iterative approach with a stack
fn parse_device_tree_iterative<'a>(
    mut input: &'a [u8],
    strings_block: &'a [u8],
) -> Result<DeviceTreeNode<'a>, DtbError> {
    use alloc::vec::Vec;

    // Stack to keep track of node hierarchy
    let mut node_stack: Vec<DeviceTreeNode<'a>> = Vec::new();

    loop {
        let (remaining, token) = DtbToken::parse(input)?;
        input = remaining;

        match token {
            DtbToken::BeginNode => {
                // Parse node name
                let (remaining, name) = parse_node_name(input)?;
                input = remaining;

                // Create new node and push to stack
                let node = DeviceTreeNode::new(name);
                node_stack.push(node);
            }
            DtbToken::Property => {
                // Parse property and add to current node
                let (remaining, property) = parse_property_data(input, strings_block)?;
                input = remaining;

                // Add property to the current (top) node
                if let Some(current_node) = node_stack.last_mut() {
                    current_node.add_property(property);
                } else {
                    return Err(DtbError::InvalidToken);
                }
            }
            DtbToken::EndNode => {
                // Pop the completed node from stack
                if let Some(completed_node) = node_stack.pop() {
                    if node_stack.is_empty() {
                        // This is the root node, we're done
                        return Ok(completed_node);
                    }
                    // Add as child to the parent node
                    if let Some(parent_node) = node_stack.last_mut() {
                        parent_node.add_child(completed_node);
                    }
                } else {
                    return Err(DtbError::InvalidToken);
                }
            }
            DtbToken::End => {
                // Should not reach here with a well-formed DTB if we properly handle EndNode
                if let Some(root_node) = node_stack.pop()
                    && node_stack.is_empty()
                {
                    return Ok(root_node);
                }
                return Err(DtbError::InvalidToken);
            }
        }
    }
}
