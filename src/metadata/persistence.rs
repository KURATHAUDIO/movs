use crate::error::{MovsError, Result};
use crate::metadata::{get_snapshot_path, snapshot_exists};
use crate::types::{SnapshotId, SnapshotMetadata};
use std::fs;
use std::path::Path;

/// Save snapshot metadata to disk
/// 
/// Serializes the metadata to JSON and writes it to the snapshots directory.
/// 
/// # Arguments
/// 
/// * `project_root` - Root directory of the project
/// * `metadata` - Snapshot metadata to save
pub fn save_snapshot(project_root: &Path, metadata: &SnapshotMetadata) -> Result<()> {
    let snapshot_path = get_snapshot_path(project_root, &metadata.id);

    // Serialize to pretty JSON for human readability
    let json = serde_json::to_string_pretty(metadata)?;

    // Write to file
    fs::write(&snapshot_path, json)?;

    Ok(())
}

/// Load snapshot metadata from disk
/// 
/// # Arguments
/// 
/// * `project_root` - Root directory of the project
/// * `snapshot_id` - ID of the snapshot to load
/// 
/// # Returns
/// 
/// The deserialized snapshot metadata
pub fn load_snapshot(project_root: &Path, snapshot_id: &SnapshotId) -> Result<SnapshotMetadata> {
    if !snapshot_exists(project_root, snapshot_id) {
        return Err(MovsError::SnapshotNotFound(snapshot_id.to_string()));
    }

    let snapshot_path = get_snapshot_path(project_root, snapshot_id);

    // Read file content
    let json = fs::read_to_string(&snapshot_path)?;

    // Deserialize
    let metadata: SnapshotMetadata = serde_json::from_str(&json)?;

    Ok(metadata)
}

/// Delete a snapshot from disk
/// 
/// # Arguments
/// 
/// * `project_root` - Root directory of the project
/// * `snapshot_id` - ID of the snapshot to delete
pub fn delete_snapshot(project_root: &Path, snapshot_id: &SnapshotId) -> Result<()> {
    if !snapshot_exists(project_root, snapshot_id) {
        return Err(MovsError::SnapshotNotFound(snapshot_id.to_string()));
    }

    let snapshot_path = get_snapshot_path(project_root, snapshot_id);
    fs::remove_file(snapshot_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::init_repository;
    use crate::types::{FileEntry, FileHash};
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_metadata() -> SnapshotMetadata {
        let files = vec![
            FileEntry::new(
                PathBuf::from("test.txt"),
                FileHash::new(vec![1, 2, 3, 4]),
                100,
                Utc::now(),
            ),
            FileEntry::new(
                PathBuf::from("audio.wav"),
                FileHash::new(vec![5, 6, 7, 8]),
                50000,
                Utc::now(),
            ),
        ];

        SnapshotMetadata::new(
            SnapshotId::new("test_snapshot_001".to_string()),
            "Test snapshot".to_string(),
            Some("TestUser".to_string()),
            None,
            files,
        )
    }

    #[test]
    fn test_save_and_load_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        init_repository(project_root).unwrap();

        let original_metadata = create_test_metadata();

        // Save snapshot
        save_snapshot(project_root, &original_metadata).unwrap();

        // Load it back
        let loaded_metadata = load_snapshot(project_root, &original_metadata.id).unwrap();

        // Verify contents match
        assert_eq!(loaded_metadata.id, original_metadata.id);
        assert_eq!(loaded_metadata.message, original_metadata.message);
        assert_eq!(loaded_metadata.author, original_metadata.author);
        assert_eq!(loaded_metadata.file_count(), original_metadata.file_count());
    }

    #[test]
    fn test_load_nonexistent_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        init_repository(project_root).unwrap();

        let snapshot_id = SnapshotId::new("nonexistent".to_string());
        let result = load_snapshot(project_root, &snapshot_id);

        assert!(matches!(result, Err(MovsError::SnapshotNotFound(_))));
    }

    #[test]
    fn test_delete_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        init_repository(project_root).unwrap();

        let metadata = create_test_metadata();

        // Save snapshot
        save_snapshot(project_root, &metadata).unwrap();
        assert!(snapshot_exists(project_root, &metadata.id));

        // Delete it
        delete_snapshot(project_root, &metadata.id).unwrap();
        assert!(!snapshot_exists(project_root, &metadata.id));

        // Can't delete twice
        let result = delete_snapshot(project_root, &metadata.id);
        assert!(matches!(result, Err(MovsError::SnapshotNotFound(_))));
    }

    #[test]
    fn test_snapshot_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        init_repository(project_root).unwrap();

        let metadata = create_test_metadata();
        save_snapshot(project_root, &metadata).unwrap();

        // Read the raw JSON file
        let snapshot_path = get_snapshot_path(project_root, &metadata.id);
        let json_content = fs::read_to_string(snapshot_path).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Check structure
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("timestamp").is_some());
        assert!(parsed.get("message").is_some());
        assert!(parsed.get("files").is_some());
    }
}