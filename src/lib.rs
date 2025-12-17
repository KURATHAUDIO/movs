//! MOVS-Core: Music-Optimized Versioning System
//!
//! A high-performance versioning library for DAW project folders.
//!
//! # Example
//!
//! ```ignore
//! use movs_core::{Repository, SnapshotId};
//!
//! // Initialize a new repository
//! let repo = Repository::init("/path/to/project")?;
//!
//! // Create a snapshot
//! let snapshot_id = repo.create_snapshot("Initial version", None)?;
//!
//! // Restore a previous version
//! repo.restore(&snapshot_id)?;
//! ```

pub mod error;
pub mod types;

// Public exports
pub use error::{MovsError, Result};
pub use types::{FileEntry, FileHash, SnapshotDiff, SnapshotId, SnapshotMetadata};

/// Library version constant
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
