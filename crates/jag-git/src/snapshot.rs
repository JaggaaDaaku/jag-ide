use jag_core::errors::{JagError, Result};
use crate::repository::GitRepository;
use tracing::info;

/// Creates tagged snapshots for workspace rollback.
///
/// Each snapshot is a lightweight git tag pointing to a specific commit.
/// This allows agents to checkpoint their work and rollback if needed.
pub struct SnapshotManager;

impl SnapshotManager {
    /// Create a snapshot of the current workspace state.
    ///
    /// Stages all changes, commits them, and creates a lightweight tag.
    /// Returns the snapshot (tag) name.
    pub fn create_snapshot(repo: &GitRepository, label: &str) -> Result<String> {
        let tag_name = format!("jag-snapshot/{}", label);

        // Stage all changes
        repo.add_all()?;

        // Create a commit
        let message = format!("[jag-snapshot] {}", label);
        let commit_id = repo.commit(&message)?;

        // Create a lightweight tag
        let oid = git2::Oid::from_str(&commit_id)
            .map_err(|e| JagError::Internal(format!("Invalid OID: {}", e)))?;
        let commit = repo.inner().find_commit(oid)
            .map_err(|e| JagError::Internal(format!("Failed to find commit: {}", e)))?;

        repo.inner().tag_lightweight(
            &tag_name,
            commit.as_object(),
            false, // don't force overwrite
        ).map_err(|e| JagError::Internal(format!("Failed to create tag: {}", e)))?;

        info!(tag = %tag_name, commit = %commit_id, "Created snapshot");
        Ok(tag_name)
    }

    /// Rollback the workspace to a specific snapshot.
    ///
    /// WARNING: This hard-resets the working directory. All uncommitted changes are lost.
    pub fn rollback_to(repo: &GitRepository, snapshot_name: &str) -> Result<()> {
        // Find the tagged commit
        let reference = repo.inner().find_reference(&format!("refs/tags/{}", snapshot_name))
            .map_err(|e| JagError::Internal(format!("Snapshot not found: {}", e)))?;

        let target = reference.peel_to_commit()
            .map_err(|e| JagError::Internal(format!("Failed to resolve snapshot: {}", e)))?;

        // Hard reset to the snapshot commit
        repo.inner().reset(
            target.as_object(),
            git2::ResetType::Hard,
            None,
        ).map_err(|e| JagError::Internal(format!("Failed to rollback: {}", e)))?;

        info!(snapshot = snapshot_name, "Rolled back to snapshot");
        Ok(())
    }

    /// List all jag snapshots.
    pub fn list_snapshots(repo: &GitRepository) -> Result<Vec<String>> {
        let tags = repo.inner().tag_names(Some("jag-snapshot/*"))
            .map_err(|e| JagError::Internal(format!("Failed to list tags: {}", e)))?;

        let mut snapshots = Vec::new();
        for name in tags.iter().flatten() {
            snapshots.push(name.to_string());
        }

        Ok(snapshots)
    }

    /// Delete a snapshot tag.
    pub fn delete_snapshot(repo: &GitRepository, snapshot_name: &str) -> Result<()> {
        let ref_name = format!("refs/tags/{}", snapshot_name);
        let mut reference = repo.inner().find_reference(&ref_name)
            .map_err(|e| JagError::Internal(format!("Snapshot not found: {}", e)))?;
        reference.delete()
            .map_err(|e| JagError::Internal(format!("Failed to delete snapshot: {}", e)))?;
        info!(snapshot = snapshot_name, "Deleted snapshot");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::GitRepository;

    fn temp_repo_with_commit() -> (GitRepository, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("jag-snap-test-{}", uuid::Uuid::new_v4()));
        let repo = GitRepository::init(&dir).unwrap();

        // Create initial file and commit
        std::fs::write(dir.join("init.txt"), "initial content").unwrap();
        repo.add("init.txt").unwrap();
        repo.commit("Initial commit").unwrap();

        (repo, dir)
    }

    #[test]
    fn test_create_and_list_snapshot() {
        let (repo, dir) = temp_repo_with_commit();

        // Make a change
        std::fs::write(dir.join("feature.txt"), "feature code").unwrap();

        // Create snapshot
        let tag = SnapshotManager::create_snapshot(&repo, "before-refactor").unwrap();
        assert!(tag.contains("jag-snapshot"));

        // List snapshots
        let snapshots = SnapshotManager::list_snapshots(&repo).unwrap();
        assert!(!snapshots.is_empty());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_rollback() {
        let (repo, dir) = temp_repo_with_commit();

        // Create snapshot
        std::fs::write(dir.join("v1.txt"), "version 1").unwrap();
        let tag = SnapshotManager::create_snapshot(&repo, "v1").unwrap();

        // Make more changes
        std::fs::write(dir.join("v2.txt"), "version 2").unwrap();
        repo.add_all().unwrap();
        repo.commit("V2 changes").unwrap();

        // Rollback to v1
        SnapshotManager::rollback_to(&repo, &tag).unwrap();

        // v2.txt should no longer exist
        assert!(!dir.join("v2.txt").exists());

        let _ = std::fs::remove_dir_all(dir);
    }
}
