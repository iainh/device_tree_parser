# Device Tree Specification Reference

This document explains how the `device_tree_parser` library implements the Device Tree Specification and provides references to the official specification for deeper understanding.

## What is a Device Tree?

A **Device Tree** is a data structure describing the hardware components of a computer system. Originally developed for Open Firmware, device trees are now widely used in embedded systems, particularly in Linux kernel development, to describe hardware that cannot be automatically discovered.

### Key Concepts

- **Device Tree Source (DTS)**: Human-readable text format describing hardware
- **Device Tree Blob (DTB)**: Compiled binary format used by bootloaders and kernels
- **Device Tree Compiler (DTC)**: Tool that converts DTS to DTB format
- **Flattened Device Tree (FDT)**: Another name for the DTB format

## Official Specification

This library implements the **Device Tree Specification** as defined by:

**📖 [Device Tree Specification v0.4](https://github.com/devicetree-org/devicetree-specification/releases/latest)**

**Alternative Sources:**
- [Linux Kernel Documentation](https://www.kernel.org/doc/html/latest/devicetree/index.html)
- [Device Tree Reference](https://elinux.org/Device_Tree_Reference)
- [Devicetree.org](https://www.devicetree.org/)

## Specification Sections and Implementation

### 1. Flattened Device Tree Format (Chapter 5)

**Specification Reference**: [Chapter 5 - Flattened Device Tree](https://github.com/devicetree-org/devicetree-specification/blob/main/source/flattened-device-tree.rst)

#### Our Implementation:

| Spec Component | Library Implementation | File |
|----------------|------------------------|------|
| **DTB Header** | `DtbHeader` struct | `src/dtb/header.rs` |
| **Memory Reservation Block** | `MemoryReservation` | `src/dtb/memory.rs` |
| **Structure Block** | `DtbToken`, tree parsing | `src/dtb/tokens.rs`, `src/dtb/parser.rs` |
| **Strings Block** | String resolution | `src/dtb/tree.rs` |

#### DTB Header Format (Spec Section 5.2)

```rust
use device_tree_parser::DtbHeader;

// Parse DTB header according to spec
let header = DtbHeader::parse(&dtb_data)?;
println!("Magic: 0x{:x}", header.magic);        // Should be 0xd00dfeed
println!("Version: {}", header.version);        // DTB version
println!("Total size: {}", header.totalsize);   // Total DTB size
```

**Spec Details**: The header is exactly 40 bytes and contains offsets to all major blocks.

#### Structure Block Tokens (Spec Section 5.4)

```rust
use device_tree_parser::DtbToken;

// The spec defines 4 token types:
// FDT_BEGIN_NODE (0x00000001) - Start of node
// FDT_END_NODE   (0x00000002) - End of node  
// FDT_PROP       (0x00000003) - Property
// FDT_END        (0x00000009) - End of structure block
```

### 2. Device Tree Structure (Chapter 2)

**Specification Reference**: [Chapter 2 - Device Tree Structure](https://github.com/devicetree-org/devicetree-specification/blob/main/source/devicetree-basics.rst)

#### Node Hierarchy

```rust
use device_tree_parser::DeviceTreeParser;

let parser = DeviceTreeParser::new(&dtb_data);
let tree = parser.parse_tree()?;

// Root node (spec: always present, name is empty string)
println!("Root node: '{}'", tree.name); // Empty string

// Traverse tree according to spec hierarchy
for node in tree.iter_nodes() {
    println!("Node: {}", node.name);
    
    // Standard properties defined in spec
    if let Some(compatible) = node.prop_string("compatible") {
        println!("  Compatible: {}", compatible);
    }
    if let Some(reg) = node.prop_u32_array("reg") {
        println!("  Registers: {:?}", reg);
    }
}
```

#### Standard Properties (Spec Section 2.3)

| Property Name | Spec Section | Our Method | Description |
|---------------|--------------|------------|-------------|
| `compatible` | 2.3.1 | `prop_string()` | Device compatibility strings |
| `model` | 2.3.2 | `prop_string()` | Device model name |
| `phandle` | 2.3.3 | `prop_u32()` | Unique node identifier |
| `status` | 2.3.4 | `prop_string()` | Device operational status |
| `#address-cells` | 2.3.5 | `prop_u32()` | Address representation |
| `#size-cells` | 2.3.6 | `prop_u32()` | Size representation |
| `reg` | 2.3.7 | `prop_u32_array()` | Register addresses/sizes |
| `ranges` | 2.3.8 | `prop_u32_array()` | Address space mapping |

### 3. Standard Nodes (Chapter 3)

**Specification Reference**: [Chapter 3 - Standard Nodes](https://github.com/devicetree-org/devicetree-specification/blob/main/source/standard-nodes.rst)

#### Root Node (Spec Section 3.2)

```rust
// Root node requirements per spec
let root = tree.find_node("/").unwrap();

// Required properties
if let Some(model) = root.prop_string("model") {
    println!("System model: {}", model);
}
if let Some(compatible) = root.prop_string("compatible") {
    println!("System compatible: {}", compatible);
}

// Address/size cell defaults (spec: default to 2 if not specified)
let address_cells = root.prop_u32("#address-cells").unwrap_or(2);
let size_cells = root.prop_u32("#size-cells").unwrap_or(1);
```

#### CPU Nodes (Spec Section 3.8)

```rust
// Find CPU nodes according to spec
if let Some(cpus) = tree.find_node("/cpus") {
    for cpu in cpus.iter_children() {
        if cpu.prop_string("device_type") == Some("cpu") {
            println!("CPU: {}", cpu.name);
            
            // Standard CPU properties per spec
            if let Some(reg) = cpu.prop_u32("reg") {
                println!("  CPU ID: {}", reg);
            }
            if let Some(freq) = cpu.prop_u32("timebase-frequency") {
                println!("  Timebase: {} Hz", freq);
            }
        }
    }
}
```

#### Memory Node (Spec Section 3.5)

```rust
// Memory node per spec requirements
if let Some(memory) = tree.find_node("/memory") {
    if memory.prop_string("device_type") == Some("memory") {
        if let Some(reg) = memory.prop_u32_array("reg") {
            // Parse memory regions (address/size pairs per parent's cell format)
            for chunk in reg.chunks(2) {
                println!("Memory: 0x{:x} size 0x{:x}", chunk[0], chunk[1]);
            }
        }
    }
}
```

### 4. Bindings and Compatible Strings

**Specification Reference**: [Chapter 4 - Device Bindings](https://github.com/devicetree-org/devicetree-specification/blob/main/source/device-bindings.rst)

#### Device Discovery

```rust
// Find devices by compatible string (spec-defined matching)
let uart_nodes = tree.find_compatible_nodes("ns16550a");
for uart in uart_nodes {
    println!("UART at: {}", uart.name);
    
    // Standard register property
    if let Some(reg) = uart.prop_u32_array("reg") {
        println!("  Registers: 0x{:x}", reg[0]);
    }
    
    // Standard interrupt property  
    if let Some(interrupts) = uart.prop_u32_array("interrupts") {
        println!("  Interrupts: {:?}", interrupts);
    }
}
```

## High-Level API Mapping to Specification

Our library provides convenience methods that implement common spec patterns:

### UART Discovery (Spec: Serial Device Bindings)

```rust
// Implements spec-compliant UART discovery
let uart_addresses = parser.uart_addresses()?;

// Searches for standard UART compatible strings:
// - "ns16550a", "ns16550", "arm,pl011", "arm,sbsa-uart", "snps,dw-apb-uart"
```

### CPU Information (Spec: CPU Binding)

```rust
// Extracts timebase frequency per CPU binding spec
let timebase = parser.timebase_frequency()?;

// Searches /cpus node and individual CPU nodes for timebase-frequency property
```

### MMIO Region Discovery

```rust
// Discovers memory-mapped I/O regions from reg properties
let mmio_regions = parser.discover_mmio_regions()?;

// Parses all reg properties according to address/size cell format
```

## Specification Compliance

### ✅ Implemented Features

- **DTB Header parsing** (Spec 5.2) - Complete
- **Memory reservation blocks** (Spec 5.3) - Complete  
- **Structure block parsing** (Spec 5.4) - Complete
- **Property parsing** (Spec 5.5) - Complete
- **String block resolution** (Spec 5.6) - Complete
- **Standard properties** (Spec 2.3) - Complete
- **Node hierarchy** (Spec 2.4) - Complete
- **Compatible string matching** (Spec 2.3.1) - Complete

### ⚠️ Limitations

- **DTS source parsing**: Not implemented (only DTB binary format)
- **Property references**: Basic phandle support, no automatic resolution
- **Overlays**: Not supported
- **Schema validation**: No automatic validation against binding schemas

### 🔮 Future Enhancements

- DTS source file parsing
- Device tree overlay support  
- Binding schema validation
- Automatic phandle resolution
- More standard node type helpers

## Debugging and Validation

### Comparing with Standard Tools

```bash
# Compare our parsing with official dtc tool
dtc -I dtb -O dts your.dtb > reference.dts

# Use our library
cargo run --example parse_dtb your.dtb
```

### Specification Violations

Our parser will return `DtbError` for specification violations:

- Invalid magic number (not 0xd00dfeed)
- Malformed headers or structure
- Invalid token sequences
- Truncated data

## References and Further Reading

### Official Specifications
- **[Device Tree Specification](https://github.com/devicetree-org/devicetree-specification)** - Primary reference
- **[Linux Kernel DT Documentation](https://www.kernel.org/doc/html/latest/devicetree/)** - Implementation examples
- **[IEEE 1275-1994](https://standards.ieee.org/standard/1275-1994.html)** - Open Firmware standard (historical)

### Device Tree Bindings
- **[Linux DT Bindings](https://github.com/torvalds/linux/tree/master/Documentation/devicetree/bindings)** - Comprehensive binding examples
- **[U-Boot DT](https://docs.u-boot.org/en/latest/develop/devicetree/intro.html)** - Bootloader perspective

### Tools and Utilities
- **[Device Tree Compiler (dtc)](https://git.kernel.org/pub/scm/utils/dtc/dtc.git)** - Reference implementation
- **[dtx_diff](https://github.com/torvalds/linux/blob/master/scripts/dtc/dtx_diff)** - Compare device trees
- **[QEMU](https://www.qemu.org/)** - Generate test DTB files

## Contributing to Specification Compliance

When contributing to this library:

1. **Reference the spec**: Always cite relevant specification sections
2. **Follow naming**: Use spec-defined property and node names  
3. **Maintain compatibility**: Don't break spec-compliant DTB files
4. **Add tests**: Include test cases from the specification
5. **Document extensions**: Clearly mark any non-standard extensions

This ensures our library remains a reliable, specification-compliant implementation for production use.