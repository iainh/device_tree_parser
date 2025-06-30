# Device Tree Parser Benchmarks

This document explains how to run and interpret benchmarks for the device tree parser.

## Setup

The benchmarks use [Criterion](https://bheisler.github.io/criterion.rs/book/), a statistics-driven micro-benchmarking framework for Rust.

### Prerequisites

1. Ensure you have the test data file:
   ```bash
   ls test-data/virt.dtb
   ```

2. Install the benchmark dependencies (automatically handled by Cargo):
   ```bash
   cargo bench --help
   ```

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

This will run all benchmarks and generate reports in `target/criterion/`.

### Run Specific Benchmark Groups

```bash
# Run only header parsing benchmarks
cargo bench parse_header

# Run property access benchmarks
cargo bench property_access

# Run high-level API benchmarks  
cargo bench high_level_api
```

### Run with Custom Options

```bash
# Run with shorter measurement time (faster)
cargo bench -- --quick

# Run specific benchmark with verbose output
cargo bench parse_tree -- --verbose

# Generate HTML reports (requires plotters feature)
cargo bench -- --output-format html
```

## Benchmark Categories

### 1. Core Parsing Performance

- **`parse_header`**: Parsing the 40-byte DTB header
- **`parse_memory_reservations`**: Parsing memory reservation entries
- **`parse_tree`**: Full device tree structure parsing (main performance test)

### 2. Property Access Performance

- **`find_compatible_nodes`**: Finding nodes by compatible string
- **`iter_all_nodes`**: Iterating through all nodes in the tree
- **`prop_u32_array`**: Accessing and parsing u32 array properties (tests zero-copy performance)

### 3. High-Level API Performance

- **`uart_addresses`**: Finding UART device addresses
- **`timebase_frequency`**: Extracting CPU timebase frequency
- **`discover_mmio_regions`**: Discovering MMIO regions from reg properties

### 4. Full Pipeline Performance

- **`full_parsing_pipeline`**: End-to-end parsing including all steps

### 5. Scaling Analysis

- **`data_size_scaling`**: How performance scales with DTB size

## Understanding Results

### Sample Output

```
parse_header            time:   [2.15 ns 2.15 ns 2.15 ns]
parse_tree              time:   [6.83 µs 6.85 µs 6.88 µs]
property_access/prop_u32_array
                        time:   [17.8 ns 17.9 ns 18.0 ns]
```

### Key Metrics

- **Time**: Lower is better. Shows the time taken per iteration.
- **Range**: [min mean max] - The confidence interval for the measurement.
- **Outliers**: Statistical outliers that may indicate measurement noise.

### Performance Targets

Based on the zero-copy implementation:

- **Header parsing**: Should be ~2-3 ns (just copying data)
- **Tree parsing**: Should be ~6-8 µs for typical DTB files (4-8KB)
- **Property access**: Should be ~10-20 ns (zero-copy string/array access)

## Comparing Performance

### Before/After Optimization

To compare performance before and after optimizations:

1. Save baseline results:
   ```bash
   cargo bench > baseline_results.txt
   ```

2. Make changes to the code

3. Run benchmarks again and compare:
   ```bash
   cargo bench > optimized_results.txt
   ```

### Using Criterion's Built-in Comparison

Criterion automatically tracks performance over time:

```bash
# First run establishes baseline
cargo bench

# Make changes to code

# Second run compares against baseline
cargo bench
```

Look for output like:
```
Performance has improved    [+5.2345% +6.1234% +7.8901%]
Performance has regressed   [-5.2345% -6.1234% -7.8901%]
No change in performance    [-1.2345% +0.1234% +1.2345%]
```

## HTML Reports

To generate detailed HTML reports with graphs:

1. Ensure you have the `html_reports` feature enabled (already configured)
2. Run benchmarks:
   ```bash
   cargo bench
   ```
3. Open the generated reports:
   ```bash
   open target/criterion/report/index.html
   ```

## Continuous Integration

For CI environments, use:

```bash
# Quick benchmarks (less accurate but faster)
cargo bench -- --quick

# Or disable HTML generation
cargo bench -- --output-format pretty
```

## Zero-Copy Performance Benefits

The benchmarks specifically test the zero-copy optimizations:

1. **String properties**: No heap allocation for property names/values
2. **Array properties**: Raw bytes stored, parsed on-demand
3. **Node traversal**: References to original data, no copying

Key benchmarks that demonstrate zero-copy benefits:
- `prop_u32_array`: ~18ns shows efficient on-demand parsing
- `parse_tree`: ~6.8µs shows minimal allocation overhead
- `find_compatible_nodes`: ~200ns shows efficient string comparison

## Troubleshooting

### Common Issues

1. **Missing test data**:
   ```
   Error: Failed to load test DTB file
   ```
   Solution: Ensure `test-data/virt.dtb` exists

2. **Long compilation time**:
   - Criterion has many dependencies
   - Use `cargo bench --no-default-features` if needed

3. **Inconsistent results**:
   - Ensure system is not under load
   - Run multiple times and look at trends
   - Use `--quick` for faster but less accurate results

### Performance Analysis Tips

1. **Profile with `perf`** (Linux):
   ```bash
   cargo bench --bench dtb_parsing -- --profile-time=10
   ```

2. **Memory usage analysis**:
   ```bash
   valgrind --tool=massif cargo bench parse_tree
   ```

3. **Compare with other parsers**: The benchmarks provide a baseline for comparing against other device tree parsing libraries.