# Migration Guide

This guide helps you migrate between versions of device_tree_parser.

## v0.2.0 → v0.3.0: Ergonomic API Improvements

Version 0.3.0 adds ergonomic trait implementations while maintaining full backward compatibility. **No breaking changes** - all existing v0.2.0 code continues to work unchanged.

### New Features (Additive Only)

#### Index Traits for Intuitive Access
```rust
// New ergonomic ways (v0.3.0+)
let property = &node["property_name"];  // Instead of node.find_property("property_name")
let first_child = &node[0];             // Access child by index

// Old ways still work exactly the same
let property = node.find_property("property_name");
let first_child = node.children.get(0);
```

#### IntoIterator for Natural Iteration
```rust
// New ergonomic way (v0.3.0+)
for child in &node {
    println!("Child: {}", child.name);
}

// Old way still works
for child in node.iter_children() {
    println!("Child: {}", child.name);
}
```

#### TryFrom for Type-Safe Conversions
```rust
use std::convert::TryFrom;

// New ergonomic ways (v0.3.0+)
let address: u32 = u32::try_from(&property.value)?;
let values: Vec<u32> = Vec::<u32>::try_from(&property.value)?;
let text: &str = <&str>::try_from(&property.value)?;
let bytes: &[u8] = <&[u8]>::try_from(&property.value)?;

// Old ways still work
let address = property.as_u32().unwrap_or(0);
let values = node.prop_u32_array("reg").unwrap_or_default();
```

#### Display Trait for Pretty Printing
```rust
// New in v0.3.0+
println!("Node: {}", node);      // Pretty prints the node
println!("Property: {}", prop);  // Pretty prints the property
```

### Migration Strategy

**Recommended approach**: Gradually adopt new ergonomic APIs in new code while leaving existing code unchanged. All v0.2.0 APIs remain fully functional.

---

## v0.1.x → v0.2.0: Zero-Copy Parsing

This section covers the breaking changes introduced in v0.2.0 for zero-copy parsing with lifetime annotations.

## Overview of Changes

Version 0.2.0 introduces **zero-copy parsing** for significant performance improvements. This requires adding lifetime parameters to core types and changing from owned data to borrowed references.

### Performance Benefits
- **3x faster** tree parsing (~6.8μs vs ~20μs estimated)
- **Zero allocations** for strings and byte arrays during parsing
- **Lower memory usage** by borrowing from original DTB buffer
- **Better cache locality** for improved performance

## Breaking Changes Summary

| v0.1.x | v0.2.0 | Change Type |
|--------|---------|------------|
| `PropertyValue` | `PropertyValue<'a>` | Add lifetime |
| `Property` | `Property<'a>` | Add lifetime |
| `DeviceTreeNode` | `DeviceTreeNode<'a>` | Add lifetime |
| `DeviceTreeNode::new(String)` | `DeviceTreeNode::new(&str)` | Borrowed string |
| `PropertyValue::String(String)` | `PropertyValue::String(&str)` | Borrowed string |
| `PropertyValue::Bytes(Vec<u8>)` | `PropertyValue::Bytes(&[u8])` | Borrowed slice |
| `prop_u32_array() -> Option<&[u32]>` | `prop_u32_array() -> Option<Vec<u32>>` | On-demand parsing |

## Migration Steps

### 1. Update Type Annotations

**Before (v0.1.x):**
```rust
use device_tree_parser::{DeviceTreeNode, Property, PropertyValue};

fn process_node(node: DeviceTreeNode) -> Vec<Property> {
    node.properties
}

fn create_tree() -> DeviceTreeNode {
    DeviceTreeNode::new("root".to_string())
}
```

**After (v0.2.0):**
```rust
use device_tree_parser::{DeviceTreeNode, Property, PropertyValue};

fn process_node(node: DeviceTreeNode<'_>) -> Vec<Property<'_>> {
    node.properties
}

fn create_tree() -> DeviceTreeNode<'static> {
    DeviceTreeNode::new("root")  // No .to_string() needed
}
```

### 2. Update Property Value Handling

**Before (v0.1.x):**
```rust
// Creating properties with owned strings
let prop = Property {
    name: "compatible".to_string(),
    value: PropertyValue::String("ns16550a".to_string()),
};

// Pattern matching
match prop.value {
    PropertyValue::String(s) => println!("String: {}", s),
    PropertyValue::StringList(list) => {
        for s in list {
            println!("Item: {}", s);
        }
    }
    _ => {}
}
```

**After (v0.2.0):**
```rust
// Creating properties with borrowed strings
let name = "compatible";
let value = "ns16550a";
let prop = Property {
    name,
    value: PropertyValue::String(value),
};

// Pattern matching (strings are now &str)
match prop.value {
    PropertyValue::String(s) => println!("String: {}", s),
    PropertyValue::StringList(list) => {
        for s in list {
            println!("Item: {}", s);  // s is &str, not String
        }
    }
    _ => {}
}
```

### 3. Update Array Property Access

**Before (v0.1.x):**
```rust
if let Some(reg_array) = node.prop_u32_array("reg") {
    println!("First register: 0x{:x}", reg_array[0]);
    println!("Array length: {}", reg_array.len());
    
    // Could store the slice reference
    let stored_ref: &[u32] = reg_array;
}
```

**After (v0.2.0):**
```rust
if let Some(reg_array) = node.prop_u32_array("reg") {
    println!("First register: 0x{:x}", reg_array[0]);
    println!("Array length: {}", reg_array.len());
    
    // Now returns owned Vec<u32> (parsed on-demand)
    let stored_vec: Vec<u32> = reg_array;
}
```

### 4. Update Function Signatures

**Before (v0.1.x):**
```rust
fn find_uart_nodes(parser: &DeviceTreeParser) -> Result<Vec<DeviceTreeNode>, DtbError> {
    let tree = parser.parse_tree()?;
    Ok(tree.find_compatible_nodes("ns16550a"))
}

fn process_tree(tree: DeviceTreeNode) {
    for child in tree.children {
        println!("Child: {}", child.name);
    }
}
```

**After (v0.2.0):**
```rust
fn find_uart_nodes(parser: &DeviceTreeParser) -> Result<Vec<DeviceTreeNode<'_>>, DtbError> {
    let tree = parser.parse_tree()?;
    Ok(tree.find_compatible_nodes("ns16550a").unwrap())
}

fn process_tree(tree: DeviceTreeNode<'_>) {
    for child in tree.children {
        println!("Child: {}", child.name);  // name is now &str
    }
}
```

### 5. Update Iterator Usage

**Before (v0.1.x):**
```rust
let tree = parser.parse_tree()?;
let iterator: NodeIterator = tree.iter_nodes();

for node in iterator {
    println!("Node: {}", node.name);
}
```

**After (v0.2.0):**
```rust
let tree = parser.parse_tree()?;
let iterator: NodeIterator<'_, '_> = tree.iter_nodes();
// Or just use type inference:
// let iterator = tree.iter_nodes();

for node in iterator {
    println!("Node: {}", node.name);
}
```

## Common Compilation Errors and Fixes

### Error: Missing lifetime parameters

```
error[E0107]: missing lifetime specifier
  --> src/main.rs:10:20
   |
10 | fn process(node: DeviceTreeNode) {
   |                  ^^^^^^^^^^^^^^ expected 1 lifetime parameter
```

**Fix:** Add lifetime parameter:
```rust
fn process(node: DeviceTreeNode<'_>) {
```

### Error: Expected String, found &str

```
error[E0308]: mismatched types
  --> src/main.rs:15:35
   |
15 | let node = DeviceTreeNode::new("root".to_string());
   |                               ^^^^^^^^^^^^^^^^^^^ expected `&str`, found `String`
```

**Fix:** Remove `.to_string()`:
```rust
let node = DeviceTreeNode::new("root");
```

### Error: Borrow checker issues with lifetimes

```
error[E0515]: cannot return value referencing temporary value
  --> src/main.rs:20:5
   |
19 | let data = load_dtb_data();
20 | parser.parse_tree()
   | ^^^^^^^^^^^^^^^^^^^ returns a value referencing data here
```

**Fix:** Ensure DTB data lives long enough:
```rust
fn parse_tree(data: &[u8]) -> Result<DeviceTreeNode<'_>, DtbError> {
    let parser = DeviceTreeParser::new(data);
    parser.parse_tree()
}

// In calling code, ensure data outlives the returned tree
let data = load_dtb_data();
let tree = parse_tree(&data)?;
// Use tree while data is still alive
```

## Testing Your Migration

### 1. Compilation Test
```bash
cargo check
```
This will catch most lifetime and type issues.

### 2. Run Existing Tests
```bash
cargo test
```
Your existing tests should still pass with minimal changes.

### 3. Performance Verification
```bash
cargo bench
```
You should see significant performance improvements in parsing benchmarks.

## Advanced Migration Scenarios

### Working with Stored References

**Before (v0.1.x):**
```rust
struct MyStruct {
    nodes: Vec<DeviceTreeNode>,
    properties: Vec<Property>,
}
```

**After (v0.2.0):**
```rust
struct MyStruct<'a> {
    nodes: Vec<DeviceTreeNode<'a>>,
    properties: Vec<Property<'a>>,
}

// Or if you need owned data:
struct MyStruct {
    nodes: Vec<DeviceTreeNode<'static>>,  // Only for static data
    // Consider cloning data if you need ownership:
    node_names: Vec<String>,  // Clone the &str values
}
```

### Converting Borrowed to Owned

If you need owned data (e.g., for storing across async boundaries):

```rust
// Convert borrowed strings to owned
let owned_name: String = node.name.to_string();

// Convert property values to owned
let owned_value = match &property.value {
    PropertyValue::String(s) => s.to_string(),
    PropertyValue::StringList(list) => {
        list.iter().map(|s| s.to_string()).collect::<Vec<String>>()
    }
    PropertyValue::Bytes(bytes) => bytes.to_vec(),
    // For arrays, they're already owned Vec<T> in v0.2.0
    _ => { /* handle other cases */ }
};
```

## Benefits After Migration

Once migrated, you'll enjoy:

1. **Faster parsing**: ~3x performance improvement
2. **Lower memory usage**: No allocations for strings/arrays during parsing  
3. **Better cache performance**: Working with contiguous original data
4. **Same functionality**: All existing features work the same way

## Need Help?

- Check the [CHANGELOG.md](CHANGELOG.md) for full list of changes
- Review [BENCHMARKS.md](BENCHMARKS.md) for performance details
- Look at the updated tests in `src/dtb/tree.rs` for examples
- The Rust compiler error messages will guide you through most lifetime issues

## Migration Checklist

- [ ] Update `Cargo.toml` to version `0.2.0`
- [ ] Add lifetime parameters to type annotations: `DeviceTreeNode<'_>`
- [ ] Change `DeviceTreeNode::new(String)` to `DeviceTreeNode::new(&str)`
- [ ] Update property creation to use `&str` instead of `String`
- [ ] Handle `prop_u32_array()` returning `Vec<u32>` instead of `&[u32]`
- [ ] Fix any lifetime-related compilation errors
- [ ] Run tests to ensure functionality is preserved: `cargo test`
- [ ] Run benchmarks to verify performance improvements: `cargo bench`
- [ ] Update your own documentation/examples if needed