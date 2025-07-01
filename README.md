# Device Tree Parser

A Rust library for parsing device tree files, supporting both binary Device Tree Blob (DTB) format and device tree source files. This library is designed for embedded systems and provides `no_std` compatibility with `alloc` support.

## Features

- **Binary DTB Parsing**: Parse Device Tree Blob files with full structure validation
- **Memory Reservation Support**: Handle memory reservation blocks in DTB files
- **Property Interpolation**: Support for device tree property value parsing and interpolation
- **Address Translation**: Full support for device tree address translation between bus domains
- **Ergonomic API (v0.3.0+)**: Modern Rust trait implementations for intuitive usage
  - `Index` traits for property and child access: `node["property"]`, `node[0]`
  - `IntoIterator` for natural iteration: `for child in &node`
  - `TryFrom` for type-safe value conversions: `u32::try_from(&value)`
  - `Display` for pretty-printing nodes and properties
- **Iterator Interface**: Convenient tree traversal with `NodeIterator`
- **No-std Compatible**: Works in embedded environments with heap allocation
- **Integration Tested**: Validated against real QEMU-generated DTB files

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
device_tree_parser = "0.3.0"
```

## Usage

### Basic DTB Parsing

```rust
use device_tree_parser::{DeviceTreeParser, DtbHeader};

// Load your DTB data
let dtb_data = std::fs::read("path/to/your.dtb")?;

// Create a parser
let parser = DeviceTreeParser::new(&dtb_data);

// Parse the header
let (_remaining, header) = DtbHeader::parse(&dtb_data)?;
println!("DTB Version: {}", header.version());
```

### Tree Traversal

```rust
use device_tree_parser::{DeviceTreeParser, NodeIterator};

let dtb_data = std::fs::read("your.dtb")?;
let parser = DeviceTreeParser::new(&dtb_data);

// Parse the device tree
let tree = parser.parse_tree()?;

// Iterate through child nodes using ergonomic IntoIterator trait
for child in &tree {
    println!("Node: {}", child.name);
    
    // Access properties using Index trait
    if child.has_property("reg") {
        println!("  Register: {}", child["reg"].value);
    }
    
    // Type-safe property value conversion using TryFrom
    if let Some(prop) = child.find_property("reg") {
        if let Ok(values) = Vec::<u32>::try_from(&prop.value) {
            println!("  Register values: {:?}", values);
        }
    }
}
```

### Ergonomic API Features (v0.3.0+)

```rust
use std::convert::TryFrom;
use device_tree_parser::DeviceTreeParser;

let dtb_data = std::fs::read("your.dtb")?;
let parser = DeviceTreeParser::new(&dtb_data);
let tree = parser.parse_tree()?;

// Property access using Index trait
let model = &tree["model"];  // Instead of tree.find_property("model")

// Child access using Index trait  
let first_child = &tree[0];  // Access first child node

// Natural iteration over children
for child in &tree {
    println!("Child: {}", child.name);
}

// Type-safe conversions using TryFrom
if let Some(prop) = tree.find_property("reg") {
    let address: u32 = u32::try_from(&prop.value)?;
    let byte_data: &[u8] = <&[u8]>::try_from(&prop.value)?;
    let string_val: &str = <&str>::try_from(&prop.value)?;
}
```

### Device Tree Address Translation

Device trees often contain complex bus hierarchies where device addresses need to be translated to CPU-visible addresses. This library provides comprehensive address translation support:

```rust
use device_tree_parser::{DeviceTreeParser, DtbError};

let dtb_data = std::fs::read("your.dtb")?;
let parser = DeviceTreeParser::new(&dtb_data);
let tree = parser.parse_tree()?;

// Enhanced MMIO region discovery with address translation
let raw_regions = parser.discover_mmio_regions_translated(false)?;      // Device addresses
let cpu_regions = parser.discover_mmio_regions_translated(true)?;       // CPU addresses

for ((device_addr, size), (cpu_addr, _)) in raw_regions.iter().zip(cpu_regions.iter()) {
    println!("Device 0x{:x} -> CPU 0x{:x} (size: {} bytes)", 
             device_addr, cpu_addr, size);
}

// Find a specific device and translate its registers
let uart_nodes = tree.find_compatible_nodes("arm,pl011");
for uart in uart_nodes {
    // Get CPU-visible MMIO regions for this UART
    let mmio_regions = uart.mmio_regions(None)?;
    for (addr, size) in mmio_regions {
        println!("UART MMIO: 0x{:x} - 0x{:x}", addr, addr + size - 1);
    }
    
    // Manual address translation for specific addresses
    if uart.has_property("ranges") {
        // Try translating a specific device address
        let device_addr = 0x1000;
        match uart.translate_address(device_addr, None, 2) {
            Ok(cpu_addr) => println!("Device 0x{:x} -> CPU 0x{:x}", device_addr, cpu_addr),
            Err(e) => println!("Translation failed: {}", e),
        }
        
        // For complex hierarchies, use recursive translation
        match uart.translate_address_recursive(device_addr, 2, 10) {
            Ok(cpu_addr) => println!("Recursive: 0x{:x} -> 0x{:x}", device_addr, cpu_addr),
            Err(e) => println!("Recursive translation failed: {}", e),
        }
    }
}

// Parse address/size cell specifications
for node in tree.iter_nodes() {
    if node.has_property("ranges") {
        let address_cells = node.address_cells()?;
        let size_cells = node.size_cells()?;
        println!("Node '{}': {}+{} address cells", 
                 node.name, address_cells, size_cells);
        
        // Parse the ranges property
        let ranges = node.ranges(None, address_cells)?;
        for range in ranges {
            println!("  Range: 0x{:x} -> 0x{:x} (size: 0x{:x})",
                     range.child_address(), range.parent_address(), range.size());
        }
    }
}
```

### Address Translation Features

- **Single-level translation**: `translate_address()` for direct parent-child translation
- **Multi-level translation**: `translate_address_recursive()` for complex bus hierarchies  
- **Automatic cell parsing**: Handles variable address/size cell configurations
- **Range validation**: Comprehensive bounds checking and overflow protection
- **Error handling**: Detailed error types for translation failures
- **Helper methods**: Convenient APIs for common translation scenarios
- **Integration tested**: Validated against real ARM SoC device trees

## Documentation

### 📚 Complete Documentation Suite

- **[Device Tree Introduction](DEVICE_TREE_INTRO.md)** - New to device trees? Start here for a comprehensive introduction to concepts, terminology, and real-world examples.

- **[Device Tree Specification Reference](DEVICE_TREE_SPECIFICATION.md)** - Detailed mapping of our implementation to the official Device Tree Specification, including compliance notes and specification references.

- **[Migration Guide](MIGRATION.md)** - Upgrading from v0.1.x? Complete guide with before/after examples for the v0.2.0 zero-copy API changes.

- **[Performance & Benchmarks](BENCHMARKS.md)** - Comprehensive benchmark documentation and performance analysis of the zero-copy implementation.

### 🔗 External References

- **[Device Tree Specification v0.4](https://github.com/devicetree-org/devicetree-specification)** - Official specification
- **[Linux Kernel DT Documentation](https://www.kernel.org/doc/html/latest/devicetree/)** - Real-world usage examples
- **[Device Tree Compiler (dtc)](https://git.kernel.org/pub/scm/utils/dtc/dtc.git)** - Reference tools

## Building

### Prerequisites

This project uses:
- Rust 2024 edition
- Nix flake for reproducible development environment (optional)

### Build Commands

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Quick syntax checking
cargo check
```

### Development Environment (Nix)

If you have Nix installed, you can use the provided flake:

```bash
# Enter development shell
nix develop

# Or run commands directly
nix run .#cargo -- build
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parser_creation

# Compile tests without running
cargo test --no-run
```

### Integration Tests

The project includes integration tests using real DTB files generated by QEMU:

```bash
# Run integration tests specifically
cargo test integration_tests

# Run tests with output
cargo test -- --nocapture
```

### Test Data

Test data is located in `test-data/` and includes:
- `virt.dtb`: QEMU virtual machine device tree for testing

## Benchmarks

The library includes comprehensive benchmarks to measure parsing performance:

```bash
# Run all benchmarks
cargo bench

# Quick benchmarks (faster, less accurate)
./bench.sh quick

# Specific benchmark categories
./bench.sh parsing    # Core parsing performance
./bench.sh properties # Property access performance
./bench.sh api        # High-level API performance

# View HTML reports
./bench.sh report
```

See [BENCHMARKS.md](BENCHMARKS.md) for detailed information on benchmark categories and interpretation.

## Code Quality

### Pre-commit Hook

A pre-commit hook is automatically set up to ensure code quality. It runs:
- `cargo fmt --check` to verify code formatting
- `cargo clippy` to catch common issues and enforce best practices

The hook automatically runs when you commit and will prevent commits if checks fail.

### Manual Code Quality Checks

```bash
# Format code
cargo fmt

# Run lints (if clippy is available)
cargo clippy --all-targets --all-features

# Check formatting without modifying files
cargo fmt --check
```

## Architecture

The library is structured as follows:

- `src/lib.rs` - Main library interface and public API
- `src/dtb/` - Device Tree Blob parsing implementation
  - `header.rs` - DTB header parsing and validation
  - `parser.rs` - Core DTB parsing logic
  - `tokens.rs` - DTB token structure parsing
  - `tree.rs` - Device tree node and property structures
  - `memory.rs` - Memory reservation block handling
  - `error.rs` - Error types and handling
- `src/parser.rs` - General parsing utilities
- `src/integration_tests.rs` - Real-world DTB parsing tests

## API Reference

### Main Types

- `DeviceTreeParser` - Main parser interface
- `DtbHeader` - Device tree blob header
- `DeviceTreeNode` - Individual device tree nodes
- `Property` - Device tree properties
- `PropertyValue` - Typed property values with ergonomic trait implementations
- `MemoryReservation` - Memory reservation entries
- `NodeIterator` - Iterator for tree traversal
- `AddressSpec` - Address/size cell specifications for device tree addressing
- `AddressRange` - Address translation range mapping between bus domains

### Ergonomic Traits (v0.3.0+)

- `Index<&str>` for `DeviceTreeNode` - Property access: `node["property_name"]`
- `Index<usize>` for `DeviceTreeNode` - Child access: `node[0]`
- `IntoIterator` for `&DeviceTreeNode` - Natural iteration: `for child in &node`
- `TryFrom<&PropertyValue>` for `u32`, `u64`, `Vec<u32>`, `&str`, `&[u8]` - Type-safe conversions
- `Display` for `DeviceTreeNode` and `Property` - Pretty-printing
- `Default` for creating empty instances

### Error Handling

The library uses `DtbError` for comprehensive error reporting during parsing operations.

## No-std Support

This library is `no_std` compatible and only requires `alloc` for heap allocation. It's suitable for embedded systems and bare-metal environments.

## Contributing

1. Ensure all tests pass: `cargo test`
2. Format code: `cargo fmt`
3. Run lints: `cargo clippy`
4. Follow the existing code style and patterns
5. Add tests for new functionality

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed version history and release notes.