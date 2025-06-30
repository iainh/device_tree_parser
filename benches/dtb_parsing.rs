// ABOUTME: Benchmarks for device tree parsing performance
// ABOUTME: Measures zero-copy parsing performance using Criterion

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use device_tree_parser::DeviceTreeParser;
use std::fs;

fn load_test_dtb() -> Vec<u8> {
    fs::read("test-data/virt.dtb").expect("Failed to load test DTB file")
}

fn bench_header_parsing(c: &mut Criterion) {
    let dtb_data = load_test_dtb();
    let parser = DeviceTreeParser::new(&dtb_data);

    c.bench_function("parse_header", |b| {
        b.iter(|| parser.parse_header().unwrap())
    });
}

fn bench_memory_reservations(c: &mut Criterion) {
    let dtb_data = load_test_dtb();
    let parser = DeviceTreeParser::new(&dtb_data);

    c.bench_function("parse_memory_reservations", |b| {
        b.iter(|| parser.parse_memory_reservations().unwrap())
    });
}

fn bench_tree_parsing(c: &mut Criterion) {
    let dtb_data = load_test_dtb();
    let parser = DeviceTreeParser::new(&dtb_data);

    c.bench_function("parse_tree", |b| b.iter(|| parser.parse_tree().unwrap()));
}

fn bench_property_access(c: &mut Criterion) {
    let dtb_data = load_test_dtb();
    let parser = DeviceTreeParser::new(&dtb_data);
    let tree = parser.parse_tree().unwrap();

    let mut group = c.benchmark_group("property_access");

    // Benchmark different property access patterns
    group.bench_function("find_compatible_nodes", |b| {
        b.iter(|| tree.find_compatible_nodes("arm,pl011"))
    });

    group.bench_function("iter_all_nodes", |b| b.iter(|| tree.iter_nodes().count()));

    // Find a node with reg property for array access benchmarking
    if let Some(node_with_reg) = tree.iter_nodes().find(|n| n.has_property("reg")) {
        group.bench_function("prop_u32_array", |b| {
            b.iter(|| node_with_reg.prop_u32_array("reg"))
        });
    }

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let dtb_data = load_test_dtb();

    c.bench_function("full_parsing_pipeline", |b| {
        b.iter(|| {
            let parser = DeviceTreeParser::new(&dtb_data);
            let _header = parser.parse_header().unwrap();
            let _reservations = parser.parse_memory_reservations().unwrap();
            let tree = parser.parse_tree().unwrap();

            // Do some work with the parsed tree
            let _uart_addrs = tree.find_compatible_nodes("arm,pl011");
            tree.iter_nodes().count()
        })
    });
}

fn bench_high_level_api(c: &mut Criterion) {
    let dtb_data = load_test_dtb();
    let parser = DeviceTreeParser::new(&dtb_data);

    let mut group = c.benchmark_group("high_level_api");

    group.bench_function("uart_addresses", |b| {
        b.iter(|| parser.uart_addresses().unwrap())
    });

    group.bench_function("timebase_frequency", |b| {
        b.iter(|| parser.timebase_frequency().unwrap())
    });

    group.bench_function("discover_mmio_regions", |b| {
        b.iter(|| parser.discover_mmio_regions().unwrap())
    });

    group.finish();
}

fn bench_data_sizes(c: &mut Criterion) {
    let dtb_data = load_test_dtb();

    let mut group = c.benchmark_group("data_size_scaling");

    // Benchmark with different portions of the DTB to see how performance scales
    for &size_fraction in &[0.25, 0.5, 0.75, 1.0] {
        let size = (dtb_data.len() as f64 * size_fraction) as usize;
        let truncated_data = &dtb_data[..size.min(dtb_data.len())];

        group.bench_with_input(
            BenchmarkId::new(
                "parse_header",
                format!("{}%", (size_fraction * 100.0) as u32),
            ),
            &truncated_data,
            |b, data| {
                b.iter(|| {
                    let parser = DeviceTreeParser::new(data);
                    // Only parse header since truncated data might not have valid tree structure
                    let _ = parser.parse_header();
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_header_parsing,
    bench_memory_reservations,
    bench_tree_parsing,
    bench_property_access,
    bench_full_pipeline,
    bench_high_level_api,
    bench_data_sizes
);
criterion_main!(benches);
