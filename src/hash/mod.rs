use crate::error::{MovsError, Result};
use crate::types::FileHash;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

const BUFFER_SIZE: usize = 1024 * 1024; // 1 MB 

/// Calculate SHA-256 hash of a file
/// 
/// This function streams the file content to avoid loading large files into memory.
pub fn hash_file(path: &Path) -> Result<FileHash> {
    let file = File::open(path).map_err(|e| MovsError::HashError {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| MovsError::HashError {
            path: path.to_path_buf(),
            source: e,
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let hash_bytes = hasher.finalize().to_vec();
    Ok(FileHash::new(hash_bytes))
}

/// Calculate hashes for multiple files in parallel
/// 
/// Uses rayon for parallel processing to speed up hashing of multiple files.
pub fn hash_files_parallel<'a, I>(paths: I) -> Vec<(std::path::PathBuf, Result<FileHash>)>
where
    I: IntoIterator<Item = &'a Path>,
    I::IntoIter: Send,
{
    use rayon::prelude::*;

    paths
        .into_iter()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&path| {
            let hash_result = hash_file(path);
            (path.to_path_buf(), hash_result)
        })
        .collect()
}

/// Check if two files have the same content by comparing their hashes
pub fn files_identical(path1: &Path, path2: &Path) -> Result<bool> {
    let hash1 = hash_file(path1)?;
    let hash2 = hash_file(path2)?;
    Ok(hash1 == hash2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, MOVS!").unwrap();
        drop(file);

        // Hash the file
        let hash = hash_file(&file_path).unwrap();

        // Verify hash is correct length (32 bytes for SHA-256)
        assert_eq!(hash.as_bytes().len(), 32);

        // Hash should be deterministic
        let hash2 = hash_file(&file_path).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");

        // Create an empty file
        File::create(&file_path).unwrap();

        // Should successfully hash empty file
        let hash = hash_file(&file_path).unwrap();
        assert_eq!(hash.as_bytes().len(), 32);

        // Known SHA-256 hash of empty file
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash.to_hex(), expected);
    }

    #[test]
    fn test_hash_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.bin");

        // Create a file larger than buffer size (1MB)
        let mut file = File::create(&file_path).unwrap();
        let data = vec![0xAB; 2 * 1024 * 1024]; // 2 MB of data
        file.write_all(&data).unwrap();
        drop(file);

        // Should successfully hash large file
        let hash = hash_file(&file_path).unwrap();
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_hash_nonexistent_file() {
        let result = hash_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
        
        match result {
            Err(MovsError::HashError { path, .. }) => {
                assert_eq!(path, Path::new("/nonexistent/file.txt"));
            }
            _ => panic!("Expected HashError"),
        }
    }

    #[test]
    fn test_files_identical() {
        let temp_dir = TempDir::new().unwrap();

        // Create two identical files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, b"identical content").unwrap();
        fs::write(&file2, b"identical content").unwrap();

        assert!(files_identical(&file1, &file2).unwrap());

        // Modify one file
        fs::write(&file2, b"different content").unwrap();

        assert!(!files_identical(&file1, &file2).unwrap());
    }

    #[test]
    fn test_hash_files_parallel() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple test files
        let mut paths = Vec::new();
        for i in 0..5 {
            let path = temp_dir.path().join(format!("file{}.txt", i));
            fs::write(&path, format!("content {}", i).as_bytes()).unwrap();
            paths.push(path);
        }

        // Hash them in parallel
        let results = hash_files_parallel(paths.iter().map(|p| p.as_path()));

        assert_eq!(results.len(), 5);
        
        // All should succeed
        for (path, result) in results {
            assert!(result.is_ok(), "Failed to hash {:?}", path);
        }
    }

    #[test]
    fn test_deterministic_hashing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("deterministic.txt");

        fs::write(&file_path, b"test data for determinism").unwrap();

        // Hash multiple times
        let hash1 = hash_file(&file_path).unwrap();
        let hash2 = hash_file(&file_path).unwrap();
        let hash3 = hash_file(&file_path).unwrap();

        // All hashes should be identical
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }
}