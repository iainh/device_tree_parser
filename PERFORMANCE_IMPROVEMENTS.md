# Zero-Copy Performance Improvements

## Summary

We've implemented zero-copy parsing for the device tree parser, which significantly reduces memory allocations and improves performance.

## Changes Made

### 1. **Property Values Now Use Borrowed Data**
- `PropertyValue::String(&'a str)` instead of `PropertyValue::String(String)`
- `PropertyValue::StringList(Vec<&'a str>)` instead of `PropertyValue::StringList(Vec<String>)`
- `PropertyValue::Bytes(&'a [u8])` instead of `PropertyValue::Bytes(Vec<u8>)`
- Arrays are stored as raw bytes and parsed on-demand

### 2. **Property Names Use Borrowed Data**
- `Property.name: &'a str` instead of `Property.name: String`

### 3. **Node Names Use Borrowed Data**
- `DeviceTreeNode.name: &'a str` instead of `DeviceTreeNode.name: String`

### 4. **On-Demand Parsing for Arrays**
- U32 and U64 arrays are stored as raw byte slices
- Values are parsed from big-endian bytes when accessed
- Single values are still parsed immediately for efficiency

## Performance Benefits

1. **Reduced Allocations**: No heap allocations for strings and byte arrays during parsing
2. **Lower Memory Usage**: Data is borrowed from the original DTB buffer
3. **Faster Parsing**: Less time spent allocating and copying data
4. **Cache Efficiency**: Better data locality since we're working with the original buffer

## API Changes

The main API change is that `prop_u32_array()` now returns `Option<Vec<u32>>` instead of `Option<&[u32]>`. This is because the u32 values need to be parsed from the raw bytes on-demand. For single values, the API remains the same.

## Code Quality

The implementation follows idiomatic Rust patterns:
- Uses proper lifetime annotations for zero-copy borrowing
- Passes `cargo clippy` with pedantic lints
- Follows Rust naming conventions and formatting standards
- Properly uses `Result` types for error handling
- Avoids redundant casts and uses `From` trait where appropriate

## Future Improvements

1. Create custom iterators for array types to avoid allocating Vec
2. Implement lazy parsing for the entire tree structure
3. Add benchmarks to quantify the performance improvements
4. Consider using `SmallVec` for collections that are typically small