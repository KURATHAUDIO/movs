# MOVS
A Music-Optimized Versioning System for DAWs

## Project Overview
This is the heart of the Music-Optimized Versioning System (MOVS). It is a standalone Rust library responsible for all the low-level logic related to snapshotting, managing versions, and restoring DAW project folders. This library provides the robust, efficient, and custom versioning capabilities that the `MOVS-TauriApp` and `MOVS-JuceVST3` will utilize.

## Key Features (MVP Goals)
-   **File Hashing:** Efficiently calculates cryptographic hashes for files to detect changes.
-   **Folder Snapshotting:** Creates logical "snapshots" of specified directories.
-   **Version Metadata Management:** Stores information about each snapshot (timestamp, message, changed files, etc.).
-   **Content Addressable Storage (Simplified):** Manages file content, avoiding redundant storage of unchanged files (e.g., by linking or intelligent copying).
-   **Restore Operations:** Copies files from a specified snapshot back to a target directory.

## Technologies Used
-   **Rust:** For high performance, memory safety, and efficient file system operations.
    -   `std::fs`
    -   `sha2` (or similar for hashing)
    -   `serde` (for JSON serialization of metadata)
    -   `walkdir` (for efficient directory traversal)

## Getting Started (for Developers)

### Prerequisites
-   [Rustup](https://rustup.rs/) (to install Rust and Cargo)

### Building & Testing
```bash
# Build the library
cargo build

# Run tests
cargo test
```

## How to Integrate (for MOVS-TauriApp, MOVS-JuceVST3)
This library will be integrated into other MOVS components primarily as a dependency or through IPC calls to the `MOVS-TauriApp`. The `MOVS-TauriApp` will typically expose the `MOVS-Core` functionalities.

## [License](LICENSE)
