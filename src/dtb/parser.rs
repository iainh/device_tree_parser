// ABOUTME: Core DTB parser implementation using nom combinators
// ABOUTME: Provides the main DeviceTreeParser struct and parsing logic

use super::error::DtbError;
use super::header::DtbHeader;
use super::memory::MemoryReservation;
use super::tokens::DtbToken;
use super::tree::{DeviceTreeNode, parse_node_name, parse_property_data};
use alloc::vec::Vec;

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

    /// Parse the DTB header
    pub fn parse_header(&self) -> Result<DtbHeader, DtbError> {
        let (_remaining, header) = DtbHeader::parse(self.data)?;
        Ok(header)
    }

    /// Parse memory reservations
    pub fn parse_memory_reservations(&self) -> Result<Vec<MemoryReservation>, DtbError> {
        let header = self.parse_header()?;
        let reservation_data = &self.data[header.off_mem_rsvmap as usize..];
        let (_remaining, reservations) = MemoryReservation::parse_all(reservation_data)?;
        Ok(reservations)
    }

    /// Parse the complete device tree structure
    pub fn parse_tree(&self) -> Result<DeviceTreeNode, DtbError> {
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

        self.parse_structure_block(struct_block, strings_block)
    }

    /// Find UART device addresses
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
                    addresses.push(reg[0] as u64);
                }
            }
        }

        Ok(addresses)
    }

    /// Get timebase frequency from CPU node
    pub fn timebase_frequency(&self) -> Result<Option<u32>, DtbError> {
        let root = self.parse_tree()?;

        // Look in /cpus node first
        if let Some(cpus_node) = root.find_node("/cpus") {
            if let Some(freq) = cpus_node.prop_u32("timebase-frequency") {
                return Ok(Some(freq));
            }

            // Check individual CPU nodes
            for cpu in cpus_node.iter_children() {
                if let Some(freq) = cpu.prop_u32("timebase-frequency") {
                    return Ok(Some(freq));
                }
            }
        }

        Ok(None)
    }

    /// Discover MMIO regions from device tree
    pub fn discover_mmio_regions(&self) -> Result<Vec<(u64, u64)>, DtbError> {
        let root = self.parse_tree()?;
        let mut regions = Vec::new();

        // Traverse all nodes and collect reg properties
        for node in root.iter_nodes() {
            if let Some(reg) = node.prop_u32_array("reg") {
                // Parse reg property as address/size pairs
                let mut i = 0;
                while i + 1 < reg.len() {
                    let address = reg[i] as u64;
                    let size = reg[i + 1] as u64;
                    regions.push((address, size));
                    i += 2;
                }
            }
        }

        Ok(regions)
    }

    /// Find node by path
    pub fn find_node(&self, path: &str) -> Result<Option<DeviceTreeNode>, DtbError> {
        let root = self.parse_tree()?;
        Ok(root.find_node(path).cloned())
    }

    /// Find all nodes with a specific compatible string
    pub fn find_compatible_nodes(&self, compatible: &str) -> Result<Vec<DeviceTreeNode>, DtbError> {
        let root = self.parse_tree()?;
        let nodes = root.find_compatible_nodes(compatible);
        Ok(nodes.into_iter().cloned().collect())
    }

    /// Parse the structure block to build the device tree
    fn parse_structure_block<'b>(
        &self,
        struct_block: &'b [u8],
        strings_block: &'b [u8],
    ) -> Result<DeviceTreeNode, DtbError> {
        parse_device_tree_iterative(struct_block, strings_block)
    }
}

/// Parse device tree structure using an iterative approach with a stack
fn parse_device_tree_iterative<'a>(
    mut input: &'a [u8],
    strings_block: &'a [u8],
) -> Result<DeviceTreeNode, DtbError> {
    use alloc::vec::Vec;

    // Stack to keep track of node hierarchy
    let mut node_stack: Vec<DeviceTreeNode> = Vec::new();

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
                    } else {
                        // Add as child to the parent node
                        if let Some(parent_node) = node_stack.last_mut() {
                            parent_node.add_child(completed_node);
                        }
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
