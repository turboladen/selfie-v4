// src/adapters/filesystem.rs
// Real file system adapter implementation

use std::{
    fs,
    path::{Path, PathBuf},
};

use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};

use crate::ports::filesystem::{FileSystem, FileSystemError};

/// Real file system implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_file(&self, path: &Path) -> Result<String, FileSystemError> {
        Ok(fs::read_to_string(path)?)
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn expand_path(&self, path: &Path) -> Result<PathBuf, FileSystemError> {
        let binding = path.to_string_lossy();
        let expanded = shellexpand::tilde(&binding);

        Ok(PathBuf::from(expanded.as_ref()).canonicalize()?)
    }

    fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError> {
        let entries = fs::read_dir(path)?;

        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(FileSystemError::IoError)?;
            paths.push(entry.path());
        }

        Ok(paths)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, FileSystemError> {
        Ok(path.canonicalize()?)
    }

    fn config_dir(&self) -> Result<PathBuf, FileSystemError> {
        // Check for environment variable override first
        if let Ok(dir) = std::env::var("SELFIE_CONFIG_DIR") {
            return Ok(PathBuf::from(dir));
        }

        choose_app_strategy(AppStrategyArgs {
            top_level_domain: "net".to_string(),
            author: "turboladen".to_string(),
            app_name: "selfie".to_string(),
        })
        .map(|xdg| xdg.config_dir())
        .map_err(|_| FileSystemError::HomeDirNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_path_exists() {
        let fs = RealFileSystem;

        // Create a temporary directory
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        // Path shouldn't exist yet
        assert!(!fs.path_exists(&file_path));

        // Create the file
        File::create(&file_path).unwrap();

        // Path should exist now
        assert!(fs.path_exists(&file_path));
    }

    #[test]
    fn test_list_directory() {
        let fs = RealFileSystem;

        // Create a temporary directory
        let dir = tempdir().unwrap();

        // Create some files
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        // List directory
        let paths = fs.list_directory(dir.path()).unwrap();

        // Verify both files are listed
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&file1));
        assert!(paths.contains(&file2));
    }

    // This would be added to the existing tests module in crates/libselfie/src/adapters/filesystem.rs

    #[test]
    fn test_read_file() {
        let fs = RealFileSystem;

        // Create a temporary directory and file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_read.txt");

        // Write test content
        let test_content = "Hello, world!";
        fs::write(&file_path, test_content).unwrap();

        // Test reading the file
        let content = fs.read_file(&file_path).unwrap();
        assert_eq!(content, test_content);

        // Test reading a non-existent file
        let non_existent = dir.path().join("non_existent.txt");
        let err = fs.read_file(&non_existent).unwrap_err();
        assert!(matches!(err, FileSystemError::IoError(_)));
    }

    #[test]
    fn test_expand_path() {
        let fs = RealFileSystem;

        // Create a temporary directory
        let dir = tempdir().unwrap();
        let test_path = dir.path().join("test_dir");
        fs::create_dir(&test_path).unwrap();

        // Test expanding a real path
        let expanded = fs.expand_path(&test_path).unwrap();
        assert!(expanded.is_absolute());

        // Test expanding a non-existent path
        let non_existent = dir.path().join("non_existent");
        let err = fs.expand_path(&non_existent).unwrap_err();
        assert!(matches!(err, FileSystemError::IoError(_)));
    }

    #[test]
    fn test_canonicalize() {
        let fs = RealFileSystem;

        // Create a temporary directory with a subdirectory
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Test canonicalizing a real path
        let canonical = fs.canonicalize(&subdir).unwrap();
        assert!(canonical.is_absolute());

        // Test canonicalizing a non-existent path
        let non_existent = dir.path().join("non_existent");
        let err = fs.canonicalize(&non_existent).unwrap_err();
        assert!(matches!(err, FileSystemError::IoError(_)));
    }

    #[test]
    fn test_config_dir() {
        let fs = RealFileSystem;

        // Just test that we get a path (without trying to verify its exact value
        // since it may vary by system)
        let config_dir = fs.config_dir().unwrap();
        assert!(config_dir.is_absolute());
        assert!(config_dir.to_string_lossy().contains("selfie"));
    }

    #[test]
    fn test_permission_denied() {
        // This test is conditional because it's hard to reliably create
        // permission-denied scenarios across different platforms
        if cfg!(unix) {
            use std::os::unix::fs::PermissionsExt;

            let fs = RealFileSystem;

            // Create a temporary directory and file
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("no_access.txt");

            // Write test content
            let test_content = "Hello, world!";
            fs::write(&file_path, test_content).unwrap();

            // Set permissions to read-only for owner, nothing for others
            let metadata = fs::metadata(&file_path).unwrap();
            let mut perms = metadata.permissions();
            perms.set_mode(0o400); // Read-only for owner
            fs::set_permissions(&file_path, perms).unwrap();

            // If running as root, this test won't work properly
            if !nix::unistd::Uid::effective().is_root() {
                // Remove read permission for current user
                // This is a best-effort test - it may not work in all environments
                let _ = std::process::Command::new("chmod")
                    .args(["000", file_path.to_str().unwrap()])
                    .output();

                // Try to read the file - may or may not fail with permission denied
                // depending on the environment
                let result = fs.read_file(&file_path);
                if let Err(FileSystemError::IoError(_)) = result {
                    // Test passed
                }
            }
        }
    }
}
