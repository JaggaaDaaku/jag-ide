use std::path::{Path, PathBuf};
use jag_core::types::{Artifact, ArtifactId, ArtifactType};
use jag_core::errors::{JagError, Result};
use crate::generator::ArtifactGenerator;

/// Filesystem-backed artifact storage.
///
/// Artifacts are saved to `workspace_root/.jag/artifacts/<id>.<ext>`.
/// Metadata is tracked separately via `jag-db`.
pub struct ArtifactStore {
    artifacts_dir: PathBuf,
}

impl ArtifactStore {
    /// Create a new store rooted at the given workspace path.
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let artifacts_dir = workspace_root.join(".jag").join("artifacts");
        std::fs::create_dir_all(&artifacts_dir)?;
        Ok(Self { artifacts_dir })
    }

    /// Persist an artifact's content to disk.
    /// Returns the path where it was saved.
    pub fn save(&self, artifact: &Artifact) -> Result<PathBuf> {
        let ext = ArtifactGenerator::extension_for_type(&artifact.artifact_type);
        let filename = format!("{}.{}", artifact.id, ext);
        let filepath = self.artifacts_dir.join(&filename);

        std::fs::write(&filepath, &artifact.content)?;
        tracing::info!(
            artifact_id = %artifact.id,
            path = %filepath.display(),
            "Artifact saved to disk"
        );

        Ok(filepath)
    }

    /// Load an artifact's content from disk.
    pub fn load(&self, id: &ArtifactId, artifact_type: &ArtifactType) -> Result<Vec<u8>> {
        let ext = ArtifactGenerator::extension_for_type(artifact_type);
        let filename = format!("{}.{}", id, ext);
        let filepath = self.artifacts_dir.join(&filename);

        if !filepath.exists() {
            return Err(JagError::ArtifactNotFound(id.clone()));
        }

        let content = std::fs::read(&filepath)?;
        Ok(content)
    }

    /// List all artifact files in the store directory.
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(&self.artifacts_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                files.push(entry.path());
            }
        }
        Ok(files)
    }

    /// Delete an artifact from disk.
    pub fn delete(&self, id: &ArtifactId, artifact_type: &ArtifactType) -> Result<()> {
        let ext = ArtifactGenerator::extension_for_type(artifact_type);
        let filename = format!("{}.{}", id, ext);
        let filepath = self.artifacts_dir.join(&filename);

        if filepath.exists() {
            std::fs::remove_file(&filepath)?;
        }
        Ok(())
    }

    /// Get the base directory for artifacts.
    pub fn artifacts_dir(&self) -> &Path {
        &self.artifacts_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ArtifactGenerator;
    use jag_core::types::{AgentId, ArtifactType};
    use std::env;

    fn temp_store() -> (ArtifactStore, PathBuf) {
        let dir = env::temp_dir().join(format!("jag-test-{}", uuid::Uuid::new_v4()));
        let store = ArtifactStore::new(&dir).unwrap();
        (store, dir)
    }

    #[test]
    fn test_save_and_load() {
        let (store, dir) = temp_store();
        let generator = ArtifactGenerator::new();
        let artifact = generator.create("# Test PRD", ArtifactType::PRD, AgentId::new(), None);

        let path = store.save(&artifact).unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with(".md"));

        let loaded = store.load(&artifact.id, &ArtifactType::PRD).unwrap();
        assert_eq!(loaded, b"# Test PRD");

        // Cleanup
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_list_files() {
        let (store, dir) = temp_store();
        let generator = ArtifactGenerator::new();

        store.save(&generator.create("content1", ArtifactType::PRD, AgentId::new(), None)).unwrap();
        store.save(&generator.create("content2", ArtifactType::APISpecification, AgentId::new(), None)).unwrap();

        let files = store.list_files().unwrap();
        assert_eq!(files.len(), 2);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_delete() {
        let (store, dir) = temp_store();
        let generator = ArtifactGenerator::new();
        let art = generator.create("deleteme", ArtifactType::PRD, AgentId::new(), None);

        let path = store.save(&art).unwrap();
        assert!(path.exists());

        store.delete(&art.id, &ArtifactType::PRD).unwrap();
        assert!(!path.exists());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_load_nonexistent() {
        let (store, dir) = temp_store();
        let result = store.load(&ArtifactId::new(), &ArtifactType::PRD);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
