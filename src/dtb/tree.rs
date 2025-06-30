// ABOUTME: Device tree node structure and property definitions
// ABOUTME: Provides tree building and traversal functionality

use super::error::DtbError;
use super::tokens::DtbToken;
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Property value types in device tree
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// Empty property
    Empty,
    /// String value
    String(String),
    /// Multiple string values
    StringList(Vec<String>),
    /// 32-bit unsigned integer
    U32(u32),
    /// Array of 32-bit unsigned integers
    U32Array(Vec<u32>),
    /// 64-bit unsigned integer
    U64(u64),
    /// Array of 64-bit unsigned integers
    U64Array(Vec<u64>),
    /// Raw byte array
    Bytes(Vec<u8>),
}

/// Device tree property
#[derive(Debug, Clone)]
pub struct Property {
    /// Property name
    pub name: String,
    /// Property value
    pub value: PropertyValue,
}

/// Device tree node
#[derive(Debug, Clone)]
pub struct DeviceTreeNode {
    /// Node name
    pub name: String,
    /// Node properties
    pub properties: Vec<Property>,
    /// Child nodes
    pub children: Vec<DeviceTreeNode>,
}

impl DeviceTreeNode {
    /// Create a new device tree node
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Add a property to the node
    pub fn add_property(&mut self, property: Property) {
        self.properties.push(property);
    }

    /// Add a child node
    pub fn add_child(&mut self, child: DeviceTreeNode) {
        self.children.push(child);
    }

    /// Find a property by name
    pub fn find_property(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// Find a child node by name
    pub fn find_child(&self, name: &str) -> Option<&DeviceTreeNode> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Find a node by path (e.g., "/cpus/cpu@0")
    pub fn find_node(&self, path: &str) -> Option<&DeviceTreeNode> {
        if path.is_empty() || path == "/" {
            return Some(self);
        }

        let path = path.strip_prefix('/').unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();

        self.find_node_by_parts(&parts)
    }

    /// Find a node by path parts
    fn find_node_by_parts(&self, parts: &[&str]) -> Option<&DeviceTreeNode> {
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
            PropertyValue::U32Array(arr) if !arr.is_empty() => Some(arr[0]),
            _ => None,
        })
    }

    /// Get property value as string
    pub fn prop_string(&self, name: &str) -> Option<&str> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::String(s) => Some(s.as_str()),
            PropertyValue::StringList(list) if !list.is_empty() => Some(list[0].as_str()),
            _ => None,
        })
    }

    /// Get property value as u32 array
    pub fn prop_u32_array(&self, name: &str) -> Option<&[u32]> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::U32Array(arr) => Some(arr.as_slice()),
            PropertyValue::U32(val) => Some(core::slice::from_ref(val)),
            _ => None,
        })
    }

    /// Get property value as u64
    pub fn prop_u64(&self, name: &str) -> Option<u64> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::U64(val) => Some(*val),
            PropertyValue::U64Array(arr) if !arr.is_empty() => Some(arr[0]),
            _ => None,
        })
    }

    /// Get property value as bytes
    pub fn prop_bytes(&self, name: &str) -> Option<&[u8]> {
        self.find_property(name).and_then(|p| match &p.value {
            PropertyValue::Bytes(bytes) => Some(bytes.as_slice()),
            _ => None,
        })
    }

    /// Check if property exists
    pub fn has_property(&self, name: &str) -> bool {
        self.find_property(name).is_some()
    }

    /// Get all nodes with a specific property
    pub fn find_nodes_with_property(&self, property_name: &str) -> Vec<&DeviceTreeNode> {
        let mut nodes = Vec::new();
        self.collect_nodes_with_property(property_name, &mut nodes);
        nodes
    }

    /// Recursively collect nodes with a specific property
    fn collect_nodes_with_property<'a>(
        &'a self,
        property_name: &str,
        nodes: &mut Vec<&'a DeviceTreeNode>,
    ) {
        if self.has_property(property_name) {
            nodes.push(self);
        }

        for child in &self.children {
            child.collect_nodes_with_property(property_name, nodes);
        }
    }

    /// Get all nodes with a specific compatible string
    pub fn find_compatible_nodes(&self, compatible: &str) -> Vec<&DeviceTreeNode> {
        let mut nodes = Vec::new();
        self.collect_compatible_nodes(compatible, &mut nodes);
        nodes
    }

    /// Recursively collect nodes with a specific compatible string
    fn collect_compatible_nodes<'a>(
        &'a self,
        compatible: &str,
        nodes: &mut Vec<&'a DeviceTreeNode>,
    ) {
        if let Some(compat_prop) = self.find_property("compatible") {
            match &compat_prop.value {
                PropertyValue::String(s) if s == compatible => {
                    nodes.push(self);
                }
                PropertyValue::StringList(list) if list.contains(&compatible.to_string()) => {
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
    pub fn iter_nodes(&self) -> NodeIterator<'_> {
        NodeIterator::new(self)
    }

    /// Get iterator over all properties
    pub fn iter_properties(&self) -> core::slice::Iter<'_, Property> {
        self.properties.iter()
    }

    /// Get iterator over child nodes
    pub fn iter_children(&self) -> core::slice::Iter<'_, DeviceTreeNode> {
        self.children.iter()
    }
}

/// Iterator for depth-first traversal of device tree nodes
pub struct NodeIterator<'a> {
    stack: Vec<&'a DeviceTreeNode>,
}

impl<'a> NodeIterator<'a> {
    fn new(root: &'a DeviceTreeNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a DeviceTreeNode;

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
pub fn parse_null_terminated_string(input: &[u8]) -> Result<(&[u8], String), DtbError<&[u8]>> {
    let null_pos = input
        .iter()
        .position(|&b| b == 0)
        .ok_or(DtbError::MalformedHeader)?;

    let string_bytes = &input[..null_pos];
    let string = core::str::from_utf8(string_bytes)
        .map_err(|_| DtbError::MalformedHeader)?
        .to_string();

    Ok((&input[null_pos + 1..], string))
}

/// Parse node name after FDT_BEGIN_NODE token
pub fn parse_node_name(input: &[u8]) -> Result<(&[u8], String), DtbError<&[u8]>> {
    let (remaining, name) = parse_null_terminated_string(input)?;

    // Skip padding to 4-byte alignment
    let name_len = input.len() - remaining.len();
    let padding = DtbToken::calculate_padding(name_len);

    if remaining.len() < padding {
        return Err(DtbError::MalformedHeader);
    }

    Ok((&remaining[padding..], name))
}

/// Parse property data after FDT_PROP token
pub fn parse_property_data<'a>(
    input: &'a [u8],
    strings_block: &'a [u8],
) -> Result<(&'a [u8], Property), DtbError<&'a [u8]>> {
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
fn resolve_property_name(strings_block: &[u8], offset: usize) -> Result<String, DtbError<&[u8]>> {
    if offset >= strings_block.len() {
        return Err(DtbError::MalformedHeader);
    }

    let string_data = &strings_block[offset..];
    let (_remaining, name) = parse_null_terminated_string(string_data)?;
    Ok(name)
}

/// Parse property value from raw bytes
fn parse_property_value(data: &[u8]) -> PropertyValue {
    if data.is_empty() {
        return PropertyValue::Empty;
    }

    // Try to parse as string(s) first
    if let Ok(string_value) = parse_as_strings(data) {
        return string_value;
    }

    // Try to parse as u32 array
    if data.len() % 4 == 0 && !data.is_empty() {
        let mut values = Vec::new();
        for chunk in data.chunks_exact(4) {
            let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            values.push(value);
        }

        if values.len() == 1 {
            return PropertyValue::U32(values[0]);
        } else {
            return PropertyValue::U32Array(values);
        }
    }

    // Try to parse as u64 array
    if data.len() % 8 == 0 && !data.is_empty() {
        let mut values = Vec::new();
        for chunk in data.chunks_exact(8) {
            let value = u64::from_be_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            values.push(value);
        }

        if values.len() == 1 {
            return PropertyValue::U64(values[0]);
        } else {
            return PropertyValue::U64Array(values);
        }
    }

    // Fall back to raw bytes
    PropertyValue::Bytes(data.to_vec())
}

/// Try to parse data as string or string list
fn parse_as_strings(data: &[u8]) -> Result<PropertyValue, ()> {
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
                    strings.push(s.to_string());
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
            strings.push(s.to_string());
        } else {
            return Err(());
        }
    }

    match strings.len() {
        0 => Ok(PropertyValue::Empty),
        1 => Ok(PropertyValue::String(strings.into_iter().next().unwrap())),
        _ => Ok(PropertyValue::StringList(strings)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tree_node_creation() {
        let node = DeviceTreeNode::new("test".to_string());
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
        assert_eq!(value, PropertyValue::String("hello".to_string()));
    }

    #[test]
    fn test_parse_property_value_empty() {
        let data = [];
        let value = parse_property_value(&data);
        assert_eq!(value, PropertyValue::Empty);
    }

    #[test]
    fn test_node_property_accessors() {
        let mut node = DeviceTreeNode::new("test".to_string());

        // Add u32 property
        node.add_property(Property {
            name: "test-u32".to_string(),
            value: PropertyValue::U32(42),
        });

        // Add string property
        node.add_property(Property {
            name: "test-string".to_string(),
            value: PropertyValue::String("hello".to_string()),
        });

        assert_eq!(node.prop_u32("test-u32"), Some(42));
        assert_eq!(node.prop_string("test-string"), Some("hello"));
        assert_eq!(node.prop_u32("nonexistent"), None);
    }

    #[test]
    fn test_node_path_lookup() {
        let mut root = DeviceTreeNode::new("".to_string());
        let mut cpus = DeviceTreeNode::new("cpus".to_string());
        let mut cpu0 = DeviceTreeNode::new("cpu@0".to_string());

        cpu0.add_property(Property {
            name: "device_type".to_string(),
            value: PropertyValue::String("cpu".to_string()),
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
        let mut root = DeviceTreeNode::new("".to_string());
        let mut uart1 = DeviceTreeNode::new("uart@1000".to_string());
        let mut uart2 = DeviceTreeNode::new("uart@2000".to_string());

        uart1.add_property(Property {
            name: "compatible".to_string(),
            value: PropertyValue::String("ns16550a".to_string()),
        });

        uart2.add_property(Property {
            name: "compatible".to_string(),
            value: PropertyValue::StringList(vec!["ns16550a".to_string(), "ns16550".to_string()]),
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
        let mut root = DeviceTreeNode::new("".to_string());
        let mut child1 = DeviceTreeNode::new("child1".to_string());
        let child2 = DeviceTreeNode::new("child2".to_string());
        let grandchild = DeviceTreeNode::new("grandchild".to_string());

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
        let mut node = DeviceTreeNode::new("test".to_string());

        // Add various property types
        node.add_property(Property {
            name: "u32-prop".to_string(),
            value: PropertyValue::U32(42),
        });

        node.add_property(Property {
            name: "u64-prop".to_string(),
            value: PropertyValue::U64(0x123456789),
        });

        node.add_property(Property {
            name: "bytes-prop".to_string(),
            value: PropertyValue::Bytes(vec![1, 2, 3, 4]),
        });

        node.add_property(Property {
            name: "empty-prop".to_string(),
            value: PropertyValue::Empty,
        });

        assert_eq!(node.prop_u32("u32-prop"), Some(42));
        assert_eq!(node.prop_u64("u64-prop"), Some(0x123456789));
        assert_eq!(node.prop_bytes("bytes-prop"), Some(&[1, 2, 3, 4][..]));
        assert!(node.has_property("empty-prop"));
        assert!(!node.has_property("nonexistent"));
    }
}
