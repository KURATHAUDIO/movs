use crate::error::{MovsError, Result};
use crate::types::{SnapshotId};
use std::fs;
use std::path::{Path, PathBuf};

pub mod persistence;

/// The name of the MOVS repository directory
pub const MOVS_DIR: &str = ".movs";

/// Subdirectory for snapshot metadata
pub const SNAPSHOTS_DIR: &str = "snapshots";

/// Subdirectory for content-addressable storage
pub const OBJECTS_DIR: &str = "objects";

/// Configuration file name
pub const CONFIG_FILE: &str = "config.json";

/// Get the path to the .movs directory for a given project root
pub fn get_movs_dir(project_root: &Path) -> PathBuf {
    project_root.join(MOVS_DIR)
}

/// Get the path to the snapshots directory
pub fn get_snapshots_dir(project_root: &Path) -> PathBuf {
    get_movs_dir(project_root).join(SNAPSHOTS_DIR)
}

/// Get the path to the objects directory
pub fn get_objects_dir(project_root: &Path) -> PathBuf {
    get_movs_dir(project_root).join(OBJECTS_DIR)
}

/// Get the path to the config file
pub fn get_config_file(project_root: &Path) -> PathBuf {
    get_movs_dir(project_root).join(CONFIG_FILE)
}

/// Check if a MOVS repository exists at the given path
pub fn repository_exists(project_root: &Path) -> bool {
    get_movs_dir(project_root).exists()
}

/// Initialize a new MOVS repository structure
/// 
/// Creates the .movs directory and subdirectories:
/// - .movs/snapshots/ - stores snapshot metadata
/// - .movs/objects/ - stores file content (content-addressable storage)
/// - .movs/config.json - repository configuration
pub fn init_repository(project_root: &Path) -> Result<()> {
    let movs_dir = get_movs_dir(project_root);

    if movs_dir.exists() {
        return Err(MovsError::RepositoryAlreadyExists(movs_dir));
    }

    // Create directory structure
    fs::create_dir(&movs_dir)?;
    fs::create_dir(get_snapshots_dir(project_root))?;
    fs::create_dir(get_objects_dir(project_root))?;

    // Create default config
    let default_config = serde_json::json!({
        "version": crate::VERSION,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    fs::write(
        get_config_file(project_root),
        serde_json::to_string_pretty(&default_config)?,
    )?;

    Ok(())
}

/// List all snapshot IDs in the repository
pub fn list_snapshots(project_root: &Path) -> Result<Vec<SnapshotId>> {
    let snapshots_dir = get_snapshots_dir(project_root);

    if !snapshots_dir.exists() {
        return Err(MovsError::RepositoryNotFound(
            get_movs_dir(project_root),
        ));
    }

    let mut snapshot_ids = Vec::new();

    for entry in fs::read_dir(&snapshots_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Some(file_stem) = path.file_stem() {
                if let Some(name) = file_stem.to_str() {
                    snapshot_ids.push(SnapshotId::new(name.to_string()));
                }
            }
        }
    }

    // Sort by name (which includes timestamp)
    snapshot_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));

    Ok(snapshot_ids)
}

/// Get the file path for a snapshot's metadata
pub fn get_snapshot_path(project_root: &Path, snapshot_id: &SnapshotId) -> PathBuf {
    get_snapshots_dir(project_root).join(format!("{}.json", snapshot_id.as_str()))
}

/// Check if a snapshot exists
pub fn snapshot_exists(project_root: &Path, snapshot_id: &SnapshotId) -> bool {
    get_snapshot_path(project_root, snapshot_id).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_paths() {
        let project_root = Path::new("/test/project");

        assert_eq!(
            get_movs_dir(project_root),
            PathBuf::from("/test/project/.movs")
        );

        assert_eq!(
            get_snapshots_dir(project_root),
            PathBuf::from("/test/project/.movs/snapshots")
        );

        assert_eq!(
            get_objects_dir(project_root),
            PathBuf::from("/test/project/.movs/objects")
        );
    }

    #[test]
    fn test_init_repository() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Should not exist initially
        assert!(!repository_exists(project_root));

        // Initialize
        init_repository(project_root).unwrap();

        // Should now exist
        assert!(repository_exists(project_root));

        // Directories should be created
        assert!(get_movs_dir(project_root).exists());
        assert!(get_snapshots_dir(project_root).exists());
        assert!(get_objects_dir(project_root).exists());
        assert!(get_config_file(project_root).exists());

        // Can't initialize twice
        let result = init_repository(project_root);
        assert!(matches!(result, Err(MovsError::RepositoryAlreadyExists(_))));
    }

    #[test]
    fn test_list_snapshots_empty() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        init_repository(project_root).unwrap();

        let snapshots = list_snapshots(project_root).unwrap();
        assert_eq!(snapshots.len(), 0);
    }

    #[test]
    fn test_snapshot_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        init_repository(project_root).unwrap();

        let snapshot_id = SnapshotId::new("test_snapshot".to_string());

        // Should not exist initially
        assert!(!snapshot_exists(project_root, &snapshot_id));

        // Create an empty snapshot file
        let snapshot_path = get_snapshot_path(project_root, &snapshot_id);
        fs::write(snapshot_path, "{}").unwrap();

        // Should now exist
        assert!(snapshot_exists(project_root, &snapshot_id));
    }
}