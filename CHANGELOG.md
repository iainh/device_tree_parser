# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2025-01-07

### Added
- **Device Tree Address Translation**: Complete support for translating addresses between bus domains
  - `AddressSpec` struct for handling address/size cell specifications
  - `AddressRange` struct for parsing and managing address translation ranges
  - `translate_address()` method for single-level address translation
  - `translate_address_recursive()` method for multi-level bus hierarchy translation
  - `translate_reg_addresses()` helper for translating device register addresses
  - `mmio_regions()` helper for getting CPU-visible MMIO regions
  - `discover_mmio_regions_translated()` enhanced API for address translation
- Address translation infrastructure:
  - Support for variable address/size cell configurations (1-4 cells, 32-128 bits)
  - Parent node property inheritance for cell specifications
  - Ranges property parsing with proper field size handling
  - Empty ranges support for 1:1 address mapping
  - Overflow protection in address arithmetic
  - Cycle detection and recursion depth limits for safety
- Enhanced error handling:
  - `InvalidAddressCells`, `InvalidSizeCells` error variants
  - `AddressTranslationError`, `InvalidRangesFormat` error variants  
  - `TranslationCycle`, `MaxTranslationDepthExceeded` error variants
- Comprehensive integration tests with real-world DTB address translation scenarios

### Changed
- Enhanced MMIO region discovery to support both raw device addresses and CPU-translated addresses
- Updated high-level parser APIs to provide address translation options
- Improved documentation with comprehensive address translation examples
- Updated README with detailed address translation usage guide

### Performance
- Zero-copy address translation implementation
- Efficient range lookup for address translation
- No additional memory allocations for translation operations

### Documentation
- Added comprehensive address translation examples in README
- Detailed API documentation for all address translation types and methods
- Integration test examples demonstrating real-world usage

## [0.3.0] - 2024-06-30

### Added
- **NEW**: Ergonomic trait implementations for improved UX
  - `Index<&str>` trait for property access: `node["property_name"]`
  - `Index<usize>` trait for child access: `node[0]`
  - `IntoIterator` trait for natural iteration: `for child in &node`
  - `Display` trait for pretty-printing nodes and properties
  - `Default` trait for creating empty instances
  - `TryFrom` traits for type-safe property value conversions:
    - `u32::try_from(&property_value)`
    - `u64::try_from(&property_value)`
    - `Vec<u32>::try_from(&property_value)`
    - `<&str>::try_from(&property_value)`
    - `<&[u8]>::try_from(&property_value)`
- `std` feature support for standard library integration
- `std::error::Error` trait implementation for `DtbError` when `std` feature is enabled
- `Display` trait implementation for `DtbError` for better error formatting
- Comprehensive `parse_dtb` example demonstrating all library features
- Example showcases DTB header parsing, memory reservations, device tree traversal, and high-level APIs
- New test suite `test_ergonomic_traits()` covering all trait implementations

### Changed
- **ERGONOMIC**: Child iteration can now use `for child in &node` instead of `for child in node.iter_children()`
- **ERGONOMIC**: Property access supports bracket notation: `&node["reg"]` instead of `node.find_property("reg")`
- **ERGONOMIC**: Type conversions use standard Rust `TryFrom`: `u32::try_from(&value)` instead of custom methods
- Library now conditionally supports `std` with `#![cfg_attr(not(feature = "std"), no_std)]`
- Examples require `std` feature for file I/O and error handling
- Added example configuration in `Cargo.toml` with `required-features = ["std"]`
- Updated example code to demonstrate new ergonomic traits
- Updated parser internals to use `IntoIterator` where beneficial

### Documentation
- Added detailed device tree specification mapping
- Enhanced introduction documentation for new users
- Created comprehensive example with real DTB parsing demonstration
- Added documentation and examples for all new ergonomic traits

### Performance
- All trait implementations are zero-cost abstractions
- `no_std` compatibility maintained for all new traits
- No additional memory allocations introduced by trait implementations

## [0.2.0] - 2024-06-30

### Added
- Comprehensive benchmark suite using Criterion framework
- Zero-copy parsing implementation for improved performance
- HTML benchmark reports with performance graphs and statistics
- Benchmark runner script (`bench.sh`) for convenient testing
- Performance documentation in `BENCHMARKS.md` and `PERFORMANCE_IMPROVEMENTS.md`
- Migration guide for upgrading from v0.1.x

### Changed
- **BREAKING**: Added lifetime parameters to core types:
  - `PropertyValue` → `PropertyValue<'a>`
  - `Property` → `Property<'a>` 
  - `DeviceTreeNode` → `DeviceTreeNode<'a>`
  - `NodeIterator` → `NodeIterator<'a, 'b>`
- **BREAKING**: Property names and string values now use `&str` instead of `String`
- **BREAKING**: `DeviceTreeNode::new()` now takes `&str` instead of `String`
- **BREAKING**: String properties store borrowed references instead of owned strings:
  - `PropertyValue::String(&str)` instead of `PropertyValue::String(String)`
  - `PropertyValue::StringList(Vec<&str>)` instead of `PropertyValue::StringList(Vec<String>)`
- **BREAKING**: Byte properties use borrowed slices: `PropertyValue::Bytes(&[u8])` instead of `PropertyValue::Bytes(Vec<u8>)`
- **BREAKING**: Array properties stored as raw bytes for zero-copy parsing:
  - `PropertyValue::U32Array(&[u8])` stores raw bytes
  - `PropertyValue::U64Array(&[u8])` stores raw bytes
- **BREAKING**: `prop_u32_array()` returns `Option<Vec<u32>>` instead of `Option<&[u32]>` (values parsed on-demand)
- **BREAKING**: Parser method return types now include lifetime parameters:
  - `parse_tree()` → `Result<DeviceTreeNode<'a>, DtbError>`
  - `find_node()` → `Result<Option<DeviceTreeNode<'a>>, DtbError>`
  - `find_compatible_nodes()` → `Result<Vec<DeviceTreeNode<'a>>, DtbError>`

### Performance
- Header parsing: ~2.15ns (measured with Criterion)
- Tree parsing: ~6.85μs for typical DTB files (4-8KB)
- Property access: ~17.9ns for u32 array parsing
- Memory usage: Zero allocations for strings and byte arrays during parsing
- Cache efficiency: Better data locality by working with original buffer

### Fixed
- Improved code formatting and clippy compliance
- Removed redundant else blocks and unnecessary casts
- Better error handling with proper `From` trait usage

### Documentation
- Added comprehensive benchmark documentation
- Updated README with benchmark instructions  
- Added performance improvement details
- Created migration guide for API changes

### Migration Guide
See [MIGRATION.md](MIGRATION.md) for detailed instructions on upgrading from v0.1.x.

**Quick Migration Tips:**
- Add lifetime parameters to type annotations: `DeviceTreeNode` → `DeviceTreeNode<'_>`
- Use `&str` instead of `String` when creating nodes: `DeviceTreeNode::new("name")`
- Update pattern matching on `PropertyValue` to handle borrowed data
- Access to `prop_u32_array()` now returns owned `Vec<u32>` instead of borrowed slice

## [0.1.0] - 2024-06-XX

### Added
- Initial release with Device Tree Blob (DTB) parsing support
- Binary DTB format parsing with full structure validation
- Memory reservation block support
- Device tree traversal with `NodeIterator`
- Property value parsing and interpolation
- Integration tests with real QEMU-generated DTB files
- No-std compatibility with `alloc` support
- Support for embedded systems and bare-metal environments

### Core Features
- DTB header parsing and validation
- Structure block parsing with iterative approach
- Property name resolution from strings block
- Device tree node and property structures
- Error handling with comprehensive `DtbError` types
- High-level API for common operations:
  - UART device address discovery
  - CPU timebase frequency extraction
  - MMIO region discovery

### Documentation
- Comprehensive README with usage examples
- API documentation with examples
- Architecture overview
- Integration test suite with real DTB data

[unreleased]: https://github.com/iainh/device_tree_parser/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/iainh/device_tree_parser/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/iainh/device_tree_parser/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/iainh/device_tree_parser/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/iainh/device_tree_parser/releases/tag/v0.1.0