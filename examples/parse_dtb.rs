// ABOUTME: Example demonstrating device tree parsing capabilities
// ABOUTME: Shows DTB header parsing, tree traversal, and device discovery

use device_tree_parser::{DeviceTreeParser, DtbError};
use core::convert::TryFrom;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let dtb_path = if args.len() > 1 {
        &args[1]
    } else {
        "test-data/virt.dtb"
    };

    println!("🌳 Device Tree Parser Example");
    println!("==============================");
    println!("Parsing DTB file: {}", dtb_path);
    println!();

    match parse_dtb_file(dtb_path) {
        Ok(_) => println!("✅ DTB parsing completed successfully!"),
        Err(e) => {
            eprintln!("❌ Error parsing DTB: {}", e);
            process::exit(1);
        }
    }
}

fn parse_dtb_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load DTB file
    let dtb_data =
        fs::read(path).map_err(|e| format!("Failed to read DTB file '{}': {}", path, e))?;

    println!("📄 File size: {} bytes", dtb_data.len());

    // Create parser
    let parser = DeviceTreeParser::new(&dtb_data);

    // Parse and display header information
    parse_header(&parser)?;

    // Parse and display memory reservations
    parse_memory_reservations(&parser)?;

    // Parse and explore the device tree
    parse_device_tree(&parser)?;

    // Demonstrate high-level API
    demonstrate_high_level_api(&parser)?;

    Ok(())
}

fn parse_header(parser: &DeviceTreeParser) -> Result<(), DtbError> {
    println!("📋 DTB Header Information");
    println!("─────────────────────────");

    let header = parser.parse_header()?;

    println!(
        "Magic:           0x{:08x} {}",
        header.magic,
        if header.magic == 0xd00d_feed {
            "✅ Valid"
        } else {
            "❌ Invalid"
        }
    );
    println!("Total size:      {} bytes", header.totalsize);
    println!("Version:         {}", header.version);
    println!("Last compatible: {}", header.last_comp_version);
    println!("Boot CPU ID:     {}", header.boot_cpuid_phys);
    println!("Struct offset:   0x{:x}", header.off_dt_struct);
    println!("Struct size:     {} bytes", header.size_dt_struct);
    println!("Strings offset:  0x{:x}", header.off_dt_strings);
    println!("Strings size:    {} bytes", header.size_dt_strings);
    println!("Memory rsv:      0x{:x}", header.off_mem_rsvmap);
    println!();

    Ok(())
}

fn parse_memory_reservations(parser: &DeviceTreeParser) -> Result<(), DtbError> {
    println!("💾 Memory Reservations");
    println!("─────────────────────");

    let reservations = parser.parse_memory_reservations()?;

    if reservations.is_empty() {
        println!("No memory reservations found");
    } else {
        for (i, reservation) in reservations.iter().enumerate() {
            println!(
                "Reservation {}: 0x{:016x} - 0x{:016x} (size: {} bytes)",
                i,
                reservation.address,
                reservation.address + reservation.size,
                reservation.size
            );
        }
    }
    println!();

    Ok(())
}

fn parse_device_tree(parser: &DeviceTreeParser) -> Result<(), DtbError> {
    println!("🌳 Device Tree Structure");
    println!("────────────────────────");

    let tree = parser.parse_tree()?;

    // Display root node information
    println!(
        "Root node: '{}'",
        if tree.name.is_empty() { "/" } else { tree.name }
    );

    // Show root properties
    if let Some(model) = tree.prop_string("model") {
        println!("  Model: {}", model);
    }
    if let Some(compatible) = tree.prop_string("compatible") {
        println!("  Compatible: {}", compatible);
    }

    // Count nodes and properties
    let node_count = tree.iter_nodes().count();
    let total_properties: usize = tree.iter_nodes().map(|node| node.properties.len()).sum();

    println!("  Total nodes: {}", node_count);
    println!("  Total properties: {}", total_properties);
    println!();

    // Show interesting nodes
    show_cpu_information(&tree);
    show_memory_information(&tree);
    show_device_summary(&tree);

    Ok(())
}

fn show_cpu_information(tree: &device_tree_parser::DeviceTreeNode) {
    println!("🖥️  CPU Information");
    println!("─────────────────");

    if let Some(cpus_node) = tree.find_node("/cpus") {
        println!("CPUs node found");

        // Show CPU-level properties
        if let Some(address_cells) = cpus_node.prop_u32("#address-cells") {
            println!("  Address cells: {}", address_cells);
        }
        if let Some(size_cells) = cpus_node.prop_u32("#size-cells") {
            println!("  Size cells: {}", size_cells);
        }

        // Find individual CPUs
        let mut cpu_count = 0;
        for cpu in cpus_node {
            if cpu.prop_string("device_type") == Some("cpu") {
                cpu_count += 1;
                println!("  CPU {}: {}", cpu_count - 1, cpu.name);

                if let Some(compatible) = cpu.prop_string("compatible") {
                    println!("    Compatible: {}", compatible);
                }
                if let Some(reg) = cpu.prop_u32("reg") {
                    println!("    Register: {}", reg);
                }
                if let Some(freq) = cpu.prop_u32("timebase-frequency") {
                    println!("    Timebase frequency: {} Hz", freq);
                }
            }
        }

        if cpu_count == 0 {
            println!("  No CPU nodes found");
        }
    } else {
        println!("No /cpus node found");
    }
    println!();
}

fn show_memory_information(tree: &device_tree_parser::DeviceTreeNode) {
    println!("💽 Memory Information");
    println!("────────────────────");

    // Look for memory nodes
    let memory_nodes: Vec<_> = tree
        .iter_nodes()
        .filter(|node| node.prop_string("device_type") == Some("memory"))
        .collect();

    if memory_nodes.is_empty() {
        println!("No memory nodes found");
    } else {
        for (i, memory) in memory_nodes.iter().enumerate() {
            println!("Memory node {}: {}", i, memory.name);

            if let Some(reg) = memory.prop_u32_array("reg") {
                // Parse address/size pairs
                for chunk in reg.chunks(2) {
                    if chunk.len() == 2 {
                        let address = chunk[0] as u64;
                        let size = chunk[1] as u64;
                        println!(
                            "  Range: 0x{:08x} - 0x{:08x} (size: {} MB)",
                            address,
                            address + size,
                            size / (1024 * 1024)
                        );
                    }
                }
            }
        }
    }
    println!();
}

fn show_device_summary(tree: &device_tree_parser::DeviceTreeNode) {
    println!("🔧 Device Summary");
    println!("────────────────");

    let mut device_types = std::collections::HashMap::new();

    // Count devices by compatible string
    for node in tree.iter_nodes() {
        if let Some(compatible) = node.prop_string("compatible") {
            // Take the first (most specific) compatible string
            let device_type = compatible.split(',').next().unwrap_or(compatible);
            *device_types.entry(device_type).or_insert(0) += 1;
        }
    }

    if device_types.is_empty() {
        println!("No devices with compatible strings found");
    } else {
        println!("Devices by compatible string:");
        let mut sorted_devices: Vec<_> = device_types.into_iter().collect();
        sorted_devices.sort_by(|a, b| a.0.cmp(b.0));

        for (device_type, count) in sorted_devices {
            println!(
                "  {}: {} instance{}",
                device_type,
                count,
                if count == 1 { "" } else { "s" }
            );
        }
    }
    println!();
}

fn demonstrate_high_level_api(parser: &DeviceTreeParser) -> Result<(), DtbError> {
    println!("🚀 High-Level API Demonstration");
    println!("──────────────────────────────");

    // UART discovery
    match parser.uart_addresses() {
        Ok(uart_addrs) => {
            if uart_addrs.is_empty() {
                println!("📡 No UART devices found");
            } else {
                println!("📡 UART devices found:");
                for (i, addr) in uart_addrs.iter().enumerate() {
                    println!("  UART {}: 0x{:08x}", i, addr);
                }
            }
        }
        Err(e) => println!("❌ Error finding UARTs: {}", e),
    }

    // Timebase frequency
    match parser.timebase_frequency() {
        Ok(Some(freq)) => println!("⏰ Timebase frequency: {} Hz", freq),
        Ok(None) => println!("⏰ No timebase frequency found"),
        Err(e) => println!("❌ Error getting timebase frequency: {}", e),
    }

    // MMIO regions
    match parser.discover_mmio_regions() {
        Ok(regions) => {
            if regions.is_empty() {
                println!("🗺️  No MMIO regions found");
            } else {
                println!("🗺️  MMIO regions discovered: {} regions", regions.len());
                for (i, (addr, size)) in regions.iter().take(5).enumerate() {
                    println!(
                        "  Region {}: 0x{:08x} - 0x{:08x} (size: {} bytes)",
                        i,
                        addr,
                        addr + size,
                        size
                    );
                }
                if regions.len() > 5 {
                    println!("  ... and {} more regions", regions.len() - 5);
                }
            }
        }
        Err(e) => println!("❌ Error discovering MMIO regions: {}", e),
    }

    // Node and property search examples
    println!();
    println!("🔍 Search Examples");
    println!("─────────────────");

    let tree = parser.parse_tree()?;

    // Demonstrate new ergonomic traits
    println!("🎯 Ergonomic API Examples");
    println!("─────────────────────────");
    
    // Using Index trait for property access
    if let Some(cpus_node) = tree.find_node("/cpus") {
        if cpus_node.has_property("#address-cells") {
            println!("✅ Using Index trait: #address-cells = {}", cpus_node["#address-cells"].value);
        }
    }
    
    // Using TryFrom for type conversions
    if let Some(memory_node) = tree.iter_nodes().find(|n| n.prop_string("device_type") == Some("memory")) {
        if let Some(reg_property) = memory_node.find_property("reg") {
            match Vec::<u32>::try_from(&reg_property.value) {
                Ok(reg_values) => println!("✅ Using TryFrom: parsed {} u32 values from reg property", reg_values.len()),
                Err(_) => println!("❌ Could not convert reg property to Vec<u32>"),
            }
        }
    }

    // Find specific nodes
    if let Some(chosen) = tree.find_node("/chosen") {
        println!("✅ Found /chosen node: {}", chosen.name);
        if let Some(bootargs) = chosen.prop_string("bootargs") {
            println!("  Boot arguments: {}", bootargs);
        }
    } else {
        println!("❌ No /chosen node found");
    }

    // Search by compatible string
    let compatible_nodes = tree.find_compatible_nodes("arm,pl011");
    if !compatible_nodes.is_empty() {
        println!("✅ Found {} ARM PL011 UART(s)", compatible_nodes.len());
    }

    // Count nodes with specific properties
    let nodes_with_reg = tree.find_nodes_with_property("reg");
    println!("📊 Nodes with 'reg' property: {}", nodes_with_reg.len());

    let nodes_with_interrupts = tree.find_nodes_with_property("interrupts");
    println!(
        "📊 Nodes with 'interrupts' property: {}",
        nodes_with_interrupts.len()
    );

    println!();
    Ok(())
}
