use std::path::{Path, PathBuf};
use jag_core::errors::{JagError, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Represents a file entry in the workspace tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

/// Manages file operations within a sandboxed workspace directory.
///
/// All paths are validated to prevent traversal outside the workspace root.
pub struct WorkspaceManager {
    root: PathBuf,
}

impl WorkspaceManager {
    pub fn new(workspace_root: PathBuf) -> Result<Self> {
        if !workspace_root.exists() {
            std::fs::create_dir_all(&workspace_root)?;
        }
        let root = workspace_root.canonicalize().unwrap_or(workspace_root);
        Ok(Self { root })
    }

    /// Validate and resolve a path relative to the workspace root.
    /// Prevents path traversal attacks.
    fn resolve_path(&self, relative_path: &str) -> Result<PathBuf> {
        // Reject obvious traversal patterns
        if relative_path.contains("..") {
            return Err(JagError::PathTraversal(relative_path.to_string()));
        }

        let target = self.root.join(relative_path);

        // Canonicalize what we can (existing paths)
        // For new files, ensure the parent exists and is inside root
        let check_path = if target.exists() {
            target.canonicalize()?
        } else if let Some(parent) = target.parent() {
            if parent.exists() {
                let canon_parent = parent.canonicalize()?;
                canon_parent.join(target.file_name().unwrap_or_default())
            } else {
                target.clone()
            }
        } else {
            target.clone()
        };

        let canon_root = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
        if !check_path.starts_with(&canon_root) {
            return Err(JagError::PathTraversal(relative_path.to_string()));
        }

        Ok(target)
    }

    /// Read a file's content as a UTF-8 string.
    pub fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.resolve_path(relative_path)?;
        if !path.exists() {
            return Err(JagError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", relative_path),
            )));
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(content)
    }

    /// Read a file's content as raw bytes.
    pub fn read_file_bytes(&self, relative_path: &str) -> Result<Vec<u8>> {
        let path = self.resolve_path(relative_path)?;
        let content = std::fs::read(&path)?;
        Ok(content)
    }

    /// Write content to a file, creating parent directories as needed.
    pub fn write_file(&self, relative_path: &str, content: &str) -> Result<()> {
        let path = self.resolve_path(relative_path)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        info!(path = relative_path, "File written");
        Ok(())
    }

    /// Delete a file.
    pub fn delete_file(&self, relative_path: &str) -> Result<()> {
        let path = self.resolve_path(relative_path)?;
        if path.is_file() {
            std::fs::remove_file(&path)?;
            info!(path = relative_path, "File deleted");
        } else if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
            info!(path = relative_path, "Directory deleted");
        }
        Ok(())
    }

    /// Create a directory.
    pub fn create_dir(&self, relative_path: &str) -> Result<()> {
        let path = self.resolve_path(relative_path)?;
        std::fs::create_dir_all(&path)?;
        info!(path = relative_path, "Directory created");
        Ok(())
    }

    /// List files and directories in the given path.
    pub fn list_files(&self, relative_path: &str) -> Result<Vec<FileEntry>> {
        let path = self.resolve_path(relative_path)?;
        if !path.is_dir() {
            return Err(JagError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Not a directory: {}", relative_path),
            )));
        }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files (starting with .)
            if name.starts_with('.') {
                continue;
            }

            entries.push(FileEntry {
                path: entry.path()
                    .strip_prefix(&self.root)
                    .unwrap_or(entry.path().as_path())
                    .to_string_lossy()
                    .to_string()
                    .replace('\\', "/"),
                name,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified()
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Utc> = t.into();
                        dt.to_rfc3339()
                    })
                    .unwrap_or_default(),
            });
        }

        // Sort: directories first, then by name
        entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
        });

        Ok(entries)
    }

    /// Check if a file or directory exists.
    pub fn exists(&self, relative_path: &str) -> Result<bool> {
        let path = self.resolve_path(relative_path)?;
        Ok(path.exists())
    }

    /// Get the workspace root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// List files recursively (for tree view).
    pub fn list_files_recursive(&self, relative_path: &str, max_depth: usize) -> Result<Vec<FileEntry>> {
        let path = self.resolve_path(relative_path)?;
        let mut entries = Vec::new();
        self.list_recursive_inner(&path, &mut entries, 0, max_depth)?;
        Ok(entries)
    }

    fn list_recursive_inner(
        &self,
        dir: &Path,
        entries: &mut Vec<FileEntry>,
        depth: usize,
        max_depth: usize,
    ) -> Result<()> {
        if depth > max_depth || !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            entries.push(FileEntry {
                path: entry.path()
                    .strip_prefix(&self.root)
                    .unwrap_or(entry.path().as_path())
                    .to_string_lossy()
                    .to_string()
                    .replace('\\', "/"),
                name: name.clone(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified()
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Utc> = t.into();
                        dt.to_rfc3339()
                    })
                    .unwrap_or_default(),
            });

            if metadata.is_dir() {
                self.list_recursive_inner(&entry.path(), entries, depth + 1, max_depth)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace() -> (WorkspaceManager, PathBuf) {
        let dir = std::env::temp_dir().join(format!("jag-ws-test-{}", uuid::Uuid::new_v4()));
        let mgr = WorkspaceManager::new(dir.clone()).unwrap();
        (mgr, dir)
    }

    #[test]
    fn test_write_and_read() {
        let (mgr, dir) = temp_workspace();
        mgr.write_file("hello.txt", "Hello, World!").unwrap();

        let content = mgr.read_file("hello.txt").unwrap();
        assert_eq!(content, "Hello, World!");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_nested_write() {
        let (mgr, dir) = temp_workspace();
        mgr.write_file("src/main.rs", "fn main() {}").unwrap();

        assert!(mgr.exists("src/main.rs").unwrap());
        let content = mgr.read_file("src/main.rs").unwrap();
        assert_eq!(content, "fn main() {}");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_list_files() {
        let (mgr, dir) = temp_workspace();
        mgr.write_file("a.txt", "a").unwrap();
        mgr.write_file("b.txt", "b").unwrap();
        mgr.create_dir("src").unwrap();

        let entries = mgr.list_files("").unwrap();
        assert_eq!(entries.len(), 3);
        // Directories first
        assert!(entries[0].is_dir);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let (mgr, dir) = temp_workspace();
        let result = mgr.read_file("../../etc/passwd");
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_delete_file() {
        let (mgr, dir) = temp_workspace();
        mgr.write_file("deleteme.txt", "bye").unwrap();
        assert!(mgr.exists("deleteme.txt").unwrap());

        mgr.delete_file("deleteme.txt").unwrap();
        assert!(!mgr.exists("deleteme.txt").unwrap());

        let _ = std::fs::remove_dir_all(dir);
    }
}
