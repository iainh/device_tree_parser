// ABOUTME: Device tree node structure and property definitions
// ABOUTME: Provides tree building and traversal functionality

use super::error::DtbError;
use super::tokens::DtbToken;
use alloc::{vec, vec::Vec};
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::ops::Index;

/// Property value types in device tree
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue<'a> {
    /// Empty property
    Empty,
    /// String value
    String(&'a str),
    /// Multiple string values
    StringList(Vec<&'a str>),
    /// 32-bit unsigned integer
    U32(u32),
    /// Array of 32-bit unsigned integers (stored as raw bytes)
    U32Array(&'a [u8]),
    /// 64-bit unsigned integer
    U64(u64),
    /// Array of 64-bit unsigned integers (stored as raw bytes)
    U64Array(&'a [u8]),
    /// Raw byte array
    Bytes(&'a [u8]),
}

/// Device tree property
#[derive(Debug, Clone)]
pub struct Property<'a> {
    /// Property name
    pub name: &'a str,
    /// Property value
    pub value: PropertyValue<'a>,
}

/// Device tree node
#[derive(Debug, Clone)]
pub struct DeviceTreeNode<'a> {
    /// Node name
    pub name: &'a str,
    /// Node properties
    pub properties: Vec<Property<'a>>,
    /// Child nodes
    pub children: Vec<DeviceTreeNode<'a>>,
}

impl<'a> DeviceTreeNode<'a> {
    /// Create a new device tree node
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            properties: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Add a property to the node
    pub fn add_property(&mut self, property: Property<'a>) {
        self.properties.push(property);
    }

    /// Add a child node
    pub fn add_child(&mut self, child: DeviceTreeNode<'a>) {
        self.children.push(child);
    }

    /// Find a property by name
    pub fn find_property(&self, name: &str) -> Option<&Property<'a>> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// Find a child node by name
    pub fn find_child(&self, name: &str) -> Option<&DeviceTreeNode<'a>> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Find a node by path (e.g., "/cpus/cpu@0")
    pub fn find_node(&self, path: &str) -> Option<&DeviceTreeNode<'a>> {
        if path.is_empty() || path == "/" {
            return Some(self);
        }

        let path = path.strip_prefix('/').unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();

        self.find_node_by_parts(&parts)
    }

    /// Find a node by path parts
    fn find_node_by_parts(&self, parts: &[&str]) -> Option<&DeviceTreeNode<'a>> {
        if parts.is_empty() {
            return Some(self);
        }

        let current_part = parts[0];
        let remaining_parts = &parts[1..];

        // Look for exact match first
        if let Some(child) = self.find_child(current_part) {
            return child.find_node_by_parts(remaining_parts);
        }

        // Look for address-based match (e.g., "cpu@0")
        for child in &self.children {
            if child.name.starts_with(current_part)
                && let Some(at_pos) = child.name.find('@')
            {
                let base_name = &child.name[..at_pos];
                if base_name == current_part {
                    return child.find_node_by_parts(remaining_parts);
                }
            }
        }

        None
    }

    /// Get property value as u32
    pub fn prop_u32(&self, name: &str) -> Option<u32> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::U32(val) => Some(*val),
            PropertyValue::U32Array(bytes) if bytes.len() >= 4 => {
                Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            }
            _ => None,
        })
    }

    /// Get property value as string
    pub fn prop_string(&self, name: &str) -> Option<&str> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::String(s) => Some(*s),
            PropertyValue::StringList(list) if !list.is_empty() => Some(list[0]),
            _ => None,
        })
    }

    /// Get property value as u32 array
    pub fn prop_u32_array(&self, name: &str) -> Option<Vec<u32>> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::U32Array(bytes) => {
                let mut values = Vec::new();
                for chunk in bytes.chunks_exact(4) {
                    values.push(u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
                }
                Some(values)
            }
            PropertyValue::U32(val) => Some(vec![*val]),
            _ => None,
        })
    }

    /// Get property value as u64
    pub fn prop_u64(&self, name: &str) -> Option<u64> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::U64(val) => Some(*val),
            PropertyValue::U64Array(bytes) if bytes.len() >= 8 => Some(u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ])),
            _ => None,
        })
    }

    /// Get property value as bytes
    pub fn prop_bytes(&self, name: &str) -> Option<&[u8]> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::Bytes(bytes) => Some(*bytes),
            _ => None,
        })
    }

    /// Check if property exists
    pub fn has_property(&self, name: &str) -> bool {
        self.find_property(name).is_some()
    }

    /// Get all nodes with a specific property
    pub fn find_nodes_with_property(&self, property_name: &str) -> Vec<&DeviceTreeNode<'a>> {
        let mut nodes = Vec::new();
        self.collect_nodes_with_property(property_name, &mut nodes);
        nodes
    }

    /// Recursively collect nodes with a specific property
    fn collect_nodes_with_property<'b>(
        &'b self,
        property_name: &str,
        nodes: &mut Vec<&'b DeviceTreeNode<'a>>,
    ) {
        if self.has_property(property_name) {
            nodes.push(self);
        }

        for child in &self.children {
            child.collect_nodes_with_property(property_name, nodes);
        }
    }

    /// Get all nodes with a specific compatible string
    pub fn find_compatible_nodes(&self, compatible: &str) -> Vec<&DeviceTreeNode<'a>> {
        let mut nodes = Vec::new();
        self.collect_compatible_nodes(compatible, &mut nodes);
        nodes
    }

    /// Recursively collect nodes with a specific compatible string
    fn collect_compatible_nodes<'b>(
        &'b self,
        compatible: &str,
        nodes: &mut Vec<&'b DeviceTreeNode<'a>>,
    ) {
        if let Some(compat_prop) = self.find_property("compatible") {
            match &compat_prop.value {
                PropertyValue::String(s) if *s == compatible => {
                    nodes.push(self);
                }
                PropertyValue::StringList(list) if list.contains(&compatible) => {
                    nodes.push(self);
                }
                _ => {}
            }
        }

        for child in &self.children {
            child.collect_compatible_nodes(compatible, nodes);
        }
    }

    /// Get iterator over all nodes (depth-first traversal)
    pub fn iter_nodes(&self) -> NodeIterator<'a, '_> {
        NodeIterator::new(self)
    }

    /// Get iterator over all properties
    pub fn iter_properties(&self) -> core::slice::Iter<'_, Property<'a>> {
        self.properties.iter()
    }

    /// Get iterator over child nodes
    pub fn iter_children(&self) -> core::slice::Iter<'_, DeviceTreeNode<'a>> {
        self.children.iter()
    }
}

// Trait implementations for better UX

/// Index trait for property access by name
impl<'a> Index<&str> for DeviceTreeNode<'a> {
    type Output = Property<'a>;

    fn index(&self, property_name: &str) -> &Self::Output {
        self.find_property(property_name)
            .unwrap_or_else(|| panic!("Property '{}' not found", property_name))
    }
}

/// Index trait for child access by index
impl<'a> Index<usize> for DeviceTreeNode<'a> {
    type Output = DeviceTreeNode<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

/// IntoIterator trait for iterating over child nodes
impl<'a> IntoIterator for &'a DeviceTreeNode<'a> {
    type Item = &'a DeviceTreeNode<'a>;
    type IntoIter = core::slice::Iter<'a, DeviceTreeNode<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

/// Display trait for PropertyValue
impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Empty => write!(f, "<empty>"),
            PropertyValue::String(s) => write!(f, "\"{}\"", s),
            PropertyValue::StringList(list) => {
                write!(f, "[")?;
                for (i, s) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\"", s)?;
                }
                write!(f, "]")
            }
            PropertyValue::U32(val) => write!(f, "0x{:x}", val),
            PropertyValue::U32Array(bytes) => {
                write!(f, "[")?;
                for (i, chunk) in bytes.chunks_exact(4).enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    let val = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    write!(f, "0x{:x}", val)?;
                }
                write!(f, "]")
            }
            PropertyValue::U64(val) => write!(f, "0x{:x}", val),
            PropertyValue::U64Array(bytes) => {
                write!(f, "[")?;
                for (i, chunk) in bytes.chunks_exact(8).enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    let val = u64::from_be_bytes([
                        chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6],
                        chunk[7],
                    ]);
                    write!(f, "0x{:x}", val)?;
                }
                write!(f, "]")
            }
            PropertyValue::Bytes(bytes) => {
                write!(f, "[")?;
                for (i, byte) in bytes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "0x{:02x}", byte)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// Display trait for Property
impl<'a> Display for Property<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
    }
}

/// Display trait for DeviceTreeNode
impl<'a> Display for DeviceTreeNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

impl<'a> DeviceTreeNode<'a> {
    fn fmt_with_indent(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);

        if self.name.is_empty() {
            writeln!(f, "{}/ {{", indent_str)?;
        } else {
            writeln!(f, "{}{} {{", indent_str, self.name)?;
        }

        for property in &self.properties {
            writeln!(f, "{}  {}", indent_str, property)?;
        }

        for child in &self.children {
            child.fmt_with_indent(f, indent + 1)?;
        }

        writeln!(f, "{}}}", indent_str)
    }
}

/// Default trait for DeviceTreeNode
impl<'a> Default for DeviceTreeNode<'a> {
    fn default() -> Self {
        Self {
            name: "",
            properties: Vec::new(),
            children: Vec::new(),
        }
    }
}

/// Default trait for PropertyValue
impl<'a> Default for PropertyValue<'a> {
    fn default() -> Self {
        PropertyValue::Empty
    }
}

/// TryFrom trait for converting PropertyValue to u32
impl<'a> TryFrom<&PropertyValue<'a>> for u32 {
    type Error = DtbError;

    fn try_from(value: &PropertyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::U32(val) => Ok(*val),
            PropertyValue::U32Array(bytes) if bytes.len() >= 4 => {
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            }
            _ => Err(DtbError::InvalidToken),
        }
    }
}

/// TryFrom trait for converting PropertyValue to u64
impl<'a> TryFrom<&PropertyValue<'a>> for u64 {
    type Error = DtbError;

    fn try_from(value: &PropertyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::U64(val) => Ok(*val),
            PropertyValue::U64Array(bytes) if bytes.len() >= 8 => Ok(u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ])),
            PropertyValue::U32(val) => Ok(*val as u64),
            PropertyValue::U32Array(bytes) if bytes.len() >= 4 => {
                let val = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                Ok(val as u64)
            }
            _ => Err(DtbError::InvalidToken),
        }
    }
}

/// TryFrom trait for converting PropertyValue to &str
impl<'a> TryFrom<&PropertyValue<'a>> for &'a str {
    type Error = DtbError;

    fn try_from(value: &PropertyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::String(s) => Ok(*s),
            PropertyValue::StringList(list) if !list.is_empty() => Ok(list[0]),
            _ => Err(DtbError::InvalidToken),
        }
    }
}

/// TryFrom trait for converting PropertyValue to Vec<u32>
impl<'a> TryFrom<&PropertyValue<'a>> for Vec<u32> {
    type Error = DtbError;

    fn try_from(value: &PropertyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::U32Array(bytes) => {
                let mut values = Vec::new();
                for chunk in bytes.chunks_exact(4) {
                    values.push(u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
                }
                Ok(values)
            }
            PropertyValue::U32(val) => Ok(vec![*val]),
            _ => Err(DtbError::InvalidToken),
        }
    }
}

/// TryFrom trait for converting PropertyValue to &[u8]
impl<'a> TryFrom<&PropertyValue<'a>> for &'a [u8] {
    type Error = DtbError;

    fn try_from(value: &PropertyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Bytes(bytes) => Ok(*bytes),
            PropertyValue::U32Array(bytes) => Ok(*bytes),
            PropertyValue::U64Array(bytes) => Ok(*bytes),
            _ => Err(DtbError::InvalidToken),
        }
    }
}

/// Iterator for depth-first traversal of device tree nodes
pub struct NodeIterator<'a, 'b> {
    stack: Vec<&'b DeviceTreeNode<'a>>,
}

impl<'a, 'b> NodeIterator<'a, 'b> {
    fn new(root: &'b DeviceTreeNode<'a>) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a, 'b> Iterator for NodeIterator<'a, 'b> {
    type Item = &'b DeviceTreeNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // Add children to stack in reverse order for depth-first traversal
            for child in node.children.iter().rev() {
                self.stack.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}

/// Parse a null-terminated string from bytes
pub fn parse_null_terminated_string(input: &[u8]) -> Result<(&[u8], &str), DtbError> {
    let null_pos = input
        .iter()
        .position(|&b| b == 0)
        .ok_or(DtbError::MalformedHeader)?;

    let string_bytes = &input[..null_pos];
    let string = core::str::from_utf8(string_bytes).map_err(|_| DtbError::MalformedHeader)?;

    Ok((&input[null_pos + 1..], string))
}

/// Parse node name after `FDT_BEGIN_NODE` token
pub fn parse_node_name(input: &[u8]) -> Result<(&[u8], &str), DtbError> {
    let (remaining, name) = parse_null_terminated_string(input)?;

    // Skip padding to 4-byte alignment
    let name_len = input.len() - remaining.len();
    let padding = DtbToken::calculate_padding(name_len);

    if remaining.len() < padding {
        return Err(DtbError::MalformedHeader);
    }

    Ok((&remaining[padding..], name))
}

/// Parse property data after `FDT_PROP` token
pub fn parse_property_data<'a>(
    input: &'a [u8],
    strings_block: &'a [u8],
) -> Result<(&'a [u8], Property<'a>), DtbError> {
    if input.len() < 8 {
        return Err(DtbError::MalformedHeader);
    }

    // Parse property length (4 bytes)
    let prop_len = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;

    // Parse name offset (4 bytes)
    let name_offset = u32::from_be_bytes([input[4], input[5], input[6], input[7]]) as usize;

    // Skip the 8-byte header
    let remaining = &input[8..];

    if remaining.len() < prop_len {
        return Err(DtbError::MalformedHeader);
    }

    // Extract property data
    let prop_data = &remaining[..prop_len];

    // Calculate padding for 4-byte alignment
    let padding = DtbToken::calculate_padding(prop_len);
    let next_input = &remaining[prop_len + padding..];

    // Resolve property name from strings block
    let name = resolve_property_name(strings_block, name_offset)?;

    // Parse property value based on data
    let value = parse_property_value(prop_data);

    let property = Property { name, value };
    Ok((next_input, property))
}

/// Resolve property name from strings block using offset
fn resolve_property_name(strings_block: &[u8], offset: usize) -> Result<&str, DtbError> {
    if offset >= strings_block.len() {
        return Err(DtbError::MalformedHeader);
    }

    let string_data = &strings_block[offset..];
    let (_remaining, name) = parse_null_terminated_string(string_data)?;
    Ok(name)
}

/// Parse property value from raw bytes
fn parse_property_value(data: &[u8]) -> PropertyValue<'_> {
    if data.is_empty() {
        return PropertyValue::Empty;
    }

    // Try to parse as string(s) first
    if let Ok(string_value) = parse_as_strings(data) {
        return string_value;
    }

    // Try to parse as u32 array
    if data.len() % 4 == 0 && !data.is_empty() {
        // For single u32 value, parse it directly
        if data.len() == 4 {
            let value = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            return PropertyValue::U32(value);
        }
        // Store raw bytes for arrays
        return PropertyValue::U32Array(data);
    }

    // Try to parse as u64 array
    if data.len() % 8 == 0 && !data.is_empty() {
        // For single u64 value, parse it directly
        if data.len() == 8 {
            let value = u64::from_be_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]);
            return PropertyValue::U64(value);
        }
        // Store raw bytes for arrays
        return PropertyValue::U64Array(data);
    }

    // Fall back to raw bytes
    PropertyValue::Bytes(data)
}

/// Try to parse data as string or string list
fn parse_as_strings(data: &[u8]) -> Result<PropertyValue<'_>, ()> {
    // Check if all bytes are valid UTF-8 or null
    if !data
        .iter()
        .all(|&b| b == 0 || (32..=126).contains(&b) || b == 9 || b == 10 || b == 13)
    {
        return Err(());
    }

    let mut strings = Vec::new();
    let mut start = 0;

    for (i, &byte) in data.iter().enumerate() {
        if byte == 0 {
            if start < i {
                let string_bytes = &data[start..i];
                if let Ok(s) = core::str::from_utf8(string_bytes) {
                    strings.push(s);
                } else {
                    return Err(());
                }
            }
            start = i + 1;
        }
    }

    // Handle case where last string doesn't end with null
    if start < data.len() {
        let string_bytes = &data[start..];
        if let Ok(s) = core::str::from_utf8(string_bytes) {
            strings.push(s);
        } else {
            return Err(());
        }
    }

    match strings.len() {
        0 => Ok(PropertyValue::Empty),
        1 => Ok(PropertyValue::String(strings[0])),
        _ => Ok(PropertyValue::StringList(strings)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tree_node_creation() {
        let node = DeviceTreeNode::new("test");
        assert_eq!(node.name, "test");
        assert!(node.properties.is_empty());
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_parse_null_terminated_string() {
        let data = b"hello\0world";
        let result = parse_null_terminated_string(data);
        assert!(result.is_ok());
        let (remaining, string) = result.unwrap();
        assert_eq!(string, "hello");
        assert_eq!(remaining, b"world");
    }

    #[test]
    fn test_parse_node_name() {
        let data = b"root\0\0\0\0next";
        let result = parse_node_name(data);
        assert!(result.is_ok());
        let (remaining, name) = result.unwrap();
        assert_eq!(name, "root");
        assert_eq!(remaining, b"next");
    }

    #[test]
    fn test_parse_property_value_u32() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let value = parse_property_value(&data);
        assert_eq!(value, PropertyValue::U32(0x12345678));
    }

    #[test]
    fn test_parse_property_value_string() {
        let data = b"hello\0";
        let value = parse_property_value(data);
        match value {
            PropertyValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String value"),
        }
    }

    #[test]
    fn test_parse_property_value_empty() {
        let data = [];
        let value = parse_property_value(&data);
        assert_eq!(value, PropertyValue::Empty);
    }

    #[test]
    fn test_node_property_accessors() {
        let name1 = "test-u32";
        let name2 = "test-string";
        let value_str = "hello";
        let mut node = DeviceTreeNode::new("test");

        // Add u32 property
        node.add_property(Property {
            name: name1,
            value: PropertyValue::U32(42),
        });

        // Add string property
        node.add_property(Property {
            name: name2,
            value: PropertyValue::String(value_str),
        });

        assert_eq!(node.prop_u32("test-u32"), Some(42));
        assert_eq!(node.prop_string("test-string"), Some("hello"));
        assert_eq!(node.prop_u32("nonexistent"), None);
    }

    #[test]
    fn test_node_path_lookup() {
        let device_type = "device_type";
        let cpu_str = "cpu";
        let mut root = DeviceTreeNode::new("");
        let mut cpus = DeviceTreeNode::new("cpus");
        let mut cpu0 = DeviceTreeNode::new("cpu@0");

        cpu0.add_property(Property {
            name: device_type,
            value: PropertyValue::String(cpu_str),
        });

        cpus.add_child(cpu0);
        root.add_child(cpus);

        // Test root lookup
        assert!(root.find_node("/").is_some());
        assert!(root.find_node("").is_some());

        // Test path lookup
        assert!(root.find_node("/cpus").is_some());
        assert!(root.find_node("/cpus/cpu@0").is_some());
        assert!(root.find_node("/cpus/cpu").is_some()); // Should match cpu@0

        // Test non-existent path
        assert!(root.find_node("/nonexistent").is_none());
    }

    #[test]
    fn test_compatible_node_search() {
        let compatible = "compatible";
        let ns16550a = "ns16550a";
        let ns16550 = "ns16550";
        let mut root = DeviceTreeNode::new("");
        let mut uart1 = DeviceTreeNode::new("uart@1000");
        let mut uart2 = DeviceTreeNode::new("uart@2000");

        uart1.add_property(Property {
            name: compatible,
            value: PropertyValue::String(ns16550a),
        });

        uart2.add_property(Property {
            name: compatible,
            value: PropertyValue::StringList(vec![ns16550a, ns16550]),
        });

        root.add_child(uart1);
        root.add_child(uart2);

        let ns16550a_nodes = root.find_compatible_nodes("ns16550a");
        assert_eq!(ns16550a_nodes.len(), 2);

        let ns16550_nodes = root.find_compatible_nodes("ns16550");
        assert_eq!(ns16550_nodes.len(), 1);
    }

    #[test]
    fn test_node_iterator() {
        let mut root = DeviceTreeNode::new("");
        let mut child1 = DeviceTreeNode::new("child1");
        let child2 = DeviceTreeNode::new("child2");
        let grandchild = DeviceTreeNode::new("grandchild");

        child1.add_child(grandchild);
        root.add_child(child1);
        root.add_child(child2);

        let nodes: Vec<_> = root.iter_nodes().collect();
        assert_eq!(nodes.len(), 4); // root, child1, grandchild, child2

        // Check depth-first order
        assert_eq!(nodes[0].name, "");
        assert_eq!(nodes[1].name, "child1");
        assert_eq!(nodes[2].name, "grandchild");
        assert_eq!(nodes[3].name, "child2");
    }

    #[test]
    fn test_property_types() {
        let u32_prop = "u32-prop";
        let u64_prop = "u64-prop";
        let bytes_prop = "bytes-prop";
        let empty_prop = "empty-prop";
        let bytes_data = &[1u8, 2, 3, 4];
        let mut node = DeviceTreeNode::new("test");

        // Add various property types
        node.add_property(Property {
            name: u32_prop,
            value: PropertyValue::U32(42),
        });

        node.add_property(Property {
            name: u64_prop,
            value: PropertyValue::U64(0x123456789),
        });

        node.add_property(Property {
            name: bytes_prop,
            value: PropertyValue::Bytes(bytes_data),
        });

        node.add_property(Property {
            name: empty_prop,
            value: PropertyValue::Empty,
        });

        assert_eq!(node.prop_u32("u32-prop"), Some(42));
        assert_eq!(node.prop_u64("u64-prop"), Some(0x123456789));
        assert_eq!(node.prop_bytes("bytes-prop"), Some(&[1, 2, 3, 4][..]));
        assert!(node.has_property("empty-prop"));
        assert!(!node.has_property("nonexistent"));
    }

    #[test]
    fn test_ergonomic_traits() {
        use core::convert::TryFrom;

        let mut node = DeviceTreeNode::new("test");
        let mut child = DeviceTreeNode::new("child");

        // Add properties to test Index and TryFrom traits
        node.add_property(Property {
            name: "test-u32",
            value: PropertyValue::U32(42),
        });

        node.add_property(Property {
            name: "test-string",
            value: PropertyValue::String("hello"),
        });

        child.add_property(Property {
            name: "child-prop",
            value: PropertyValue::U32(100),
        });

        node.add_child(child);

        // Test Index trait for property access
        assert_eq!(node["test-u32"].name, "test-u32");
        assert_eq!(node["test-string"].name, "test-string");

        // Test Index trait for child access
        assert_eq!(node[0].name, "child");

        // Test IntoIterator trait
        let mut child_count = 0;
        for child in &node {
            child_count += 1;
            assert_eq!(child.name, "child");
        }
        assert_eq!(child_count, 1);

        // Test TryFrom trait
        let u32_val: u32 = u32::try_from(&node["test-u32"].value).unwrap();
        assert_eq!(u32_val, 42);

        let str_val: &str = <&str>::try_from(&node["test-string"].value).unwrap();
        assert_eq!(str_val, "hello");

        // Test Default trait
        let default_node = DeviceTreeNode::default();
        assert_eq!(default_node.name, "");
        assert!(default_node.properties.is_empty());
        assert!(default_node.children.is_empty());

        let default_value = PropertyValue::default();
        assert_eq!(default_value, PropertyValue::Empty);
    }
}
