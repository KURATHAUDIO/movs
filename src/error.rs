use std::path::PathBuf;
use thiserror::Error;

/// Main error type for MOVS operations
#[derive(Error, Debug)]
pub enum MovsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to hash file at '{path}': {source}")]
    HashError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to serialize metadata: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Invalid snapshot ID: {0}")]
    InvalidSnapshotId(String),

    #[error("Repository not initialized at '{0}'")]
    RepositoryNotFound(PathBuf),

    #[error("Repository already exists at '{0}'")]
    RepositoryAlreadyExists(PathBuf),

    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Restore error: {0}")]
    RestoreError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("File not found in snapshot: {0}")]
    FileNotFoundInSnapshot(PathBuf),

    #[error("Checksum mismatch for file '{path}': expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
}

/// Convenience Result type for MOVS operations
pub type Result<T> = std::result::Result<T, MovsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MovsError::SnapshotNotFound("test123".to_string());
        assert_eq!(err.to_string(), "Snapshot not found: test123");

        let err = MovsError::RepositoryNotFound(PathBuf::from("/test/path"));
        assert!(err.to_string().contains("/test/path"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let movs_err: MovsError = io_err.into();
        
        assert!(matches!(movs_err, MovsError::Io(_)));
    }
}