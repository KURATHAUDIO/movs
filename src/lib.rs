//! MOVS: Music-Optimized Versioning System
//!
//! A high-performance versioning library for DAW project folders.
//!

pub mod error;
pub mod types;
pub mod hash;

// Public exports
pub use error::{MovsError, Result};
pub use types::{
    FileEntry, FileHash, SnapshotDiff, SnapshotId, SnapshotMetadata,
};

/// Library version constant
pub const VERSION: &str = env!("CARGO_PKG_VERSION");