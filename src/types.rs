use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Represents a cryptographic hash of file content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileHash {
    /// Raw hash bytes (SHA-256 = 32 bytes)
    bytes: Vec<u8>,
}

impl FileHash {
    /// Create a new FileHash from raw bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert hash to hex string representation
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create FileHash from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        Ok(Self::new(bytes))
    }
}

impl fmt::Display for FileHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Unique identifier for a snapshot
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(String);

impl SnapshotId {
    /// Create a new SnapshotId from a string
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a new unique snapshot ID based on timestamp
    pub fn generate() -> Self {
        let now = Utc::now();
        Self(format!("snapshot_{}", now.format("%Y%m%d_%H%M%S_%f")))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SnapshotId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Represents a file entry in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative path from project root
    pub path: PathBuf,

    /// Content hash of the file
    pub hash: FileHash,

    /// File size in bytes
    pub size: u64,

    /// Last modified time (from file system)
    pub modified: DateTime<Utc>,
}

impl FileEntry {
    pub fn new(path: PathBuf, hash: FileHash, size: u64, modified: DateTime<Utc>) -> Self {
        Self {
            path,
            hash,
            size,
            modified,
        }
    }
}

/// Metadata about a snapshot version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Unique identifier for this snapshot
    pub id: SnapshotId,

    /// When this snapshot was created
    pub timestamp: DateTime<Utc>,

    /// User-provided message describing this version
    pub message: String,

    /// Optional author information
    pub author: Option<String>,

    /// Parent snapshot ID (if any)
    pub parent: Option<SnapshotId>,

    /// List of all files in this snapshot
    pub files: Vec<FileEntry>,
}

impl SnapshotMetadata {
    pub fn new(
        id: SnapshotId,
        message: String,
        author: Option<String>,
        parent: Option<SnapshotId>,
        files: Vec<FileEntry>,
    ) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            message,
            author,
            parent,
            files,
        }
    }

    /// Get the number of files in this snapshot
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Calculate total size of all files
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Find a file entry by path
    pub fn find_file(&self, path: &Path) -> Option<&FileEntry> {
        self.files.iter().find(|f| f.path == path)
    }
}

/// Represents changes between two snapshots
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    /// Files added in the new snapshot
    pub added: Vec<PathBuf>,

    /// Files modified between snapshots
    pub modified: Vec<PathBuf>,

    /// Files removed in the new snapshot
    pub removed: Vec<PathBuf>,
}

impl SnapshotDiff {
    pub fn new() -> Self {
        Self {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.removed.is_empty()
    }

    /// Get total number of changed files
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}

impl Default for SnapshotDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_hash_hex_conversion() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78];
        let hash = FileHash::new(bytes.clone());
        
        assert_eq!(hash.to_hex(), "12345678");
        assert_eq!(hash.as_bytes(), &bytes);
        
        let from_hex = FileHash::from_hex("12345678").unwrap();
        assert_eq!(hash, from_hex);
    }

    #[test]
    fn test_snapshot_id_generation() {
        let id1 = SnapshotId::generate();
        let id2 = SnapshotId::generate();
        
        // IDs should be different
        assert_ne!(id1, id2);
        
        // Should start with "snapshot_"
        assert!(id1.as_str().starts_with("snapshot_"));
    }

    #[test]
    fn test_snapshot_metadata_helpers() {
        let files = vec![
            FileEntry::new(
                PathBuf::from("test.txt"),
                FileHash::new(vec![1, 2, 3]),
                100,
                Utc::now(),
            ),
            FileEntry::new(
                PathBuf::from("audio.wav"),
                FileHash::new(vec![4, 5, 6]),
                5000,
                Utc::now(),
            ),
        ];

        let metadata = SnapshotMetadata::new(
            SnapshotId::generate(),
            "Test snapshot".to_string(),
            Some("TestUser".to_string()),
            None,
            files,
        );

        assert_eq!(metadata.file_count(), 2);
        assert_eq!(metadata.total_size(), 5100);
        assert!(metadata.find_file(&PathBuf::from("test.txt")).is_some());
        assert!(metadata.find_file(&PathBuf::from("nonexistent.txt")).is_none());
    }

    #[test]
    fn test_snapshot_diff() {
        let mut diff = SnapshotDiff::new();
        assert!(!diff.has_changes());
        assert_eq!(diff.total_changes(), 0);

        diff.added.push(PathBuf::from("new.txt"));
        diff.modified.push(PathBuf::from("changed.txt"));

        assert!(diff.has_changes());
        assert_eq!(diff.total_changes(), 2);
    }
}