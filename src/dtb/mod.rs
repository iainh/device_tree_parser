// ABOUTME: Device tree blob parsing module with nom combinators
// ABOUTME: Provides no_std compatible DTB parsing functionality

pub mod error;
pub mod header;
pub mod parser;

pub use error::DtbError;
pub use header::DtbHeader;
pub use parser::DeviceTreeParser;