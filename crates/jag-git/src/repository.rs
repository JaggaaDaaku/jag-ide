use std::path::Path;
use git2::{Repository, StatusOptions, Signature, DiffOptions};
use jag_core::errors::{JagError, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Represents a file's git status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
}

/// Represents a git commit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLogEntry {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

/// Wrapper around git2::Repository for safe git operations.
pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    /// Open an existing repository.
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)
            .map_err(|e| JagError::Internal(format!("Failed to open git repo: {}", e)))?;
        Ok(Self { repo })
    }

    /// Initialize a new repository at the given path.
    pub fn init(path: &Path) -> Result<Self> {
        let repo = Repository::init(path)
            .map_err(|e| JagError::Internal(format!("Failed to init git repo: {}", e)))?;
        info!(path = %path.display(), "Initialized git repository");
        Ok(Self { repo })
    }

    /// Get the status of all files in the working directory.
    pub fn status(&self) -> Result<Vec<GitFileStatus>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true);

        let statuses = self.repo.statuses(Some(&mut opts))
            .map_err(|e| JagError::Internal(format!("Failed to get git status: {}", e)))?;

        let mut result = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = format!("{:?}", entry.status());
            result.push(GitFileStatus { path, status });
        }

        Ok(result)
    }

    /// Stage a file for commit.
    pub fn add(&self, path: &str) -> Result<()> {
        let mut index = self.repo.index()
            .map_err(|e| JagError::Internal(format!("Failed to get index: {}", e)))?;
        index.add_path(Path::new(path))
            .map_err(|e| JagError::Internal(format!("Failed to add file: {}", e)))?;
        index.write()
            .map_err(|e| JagError::Internal(format!("Failed to write index: {}", e)))?;
        Ok(())
    }

    /// Stage all changed files.
    pub fn add_all(&self) -> Result<()> {
        let mut index = self.repo.index()
            .map_err(|e| JagError::Internal(format!("Failed to get index: {}", e)))?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| JagError::Internal(format!("Failed to add all: {}", e)))?;
        index.write()
            .map_err(|e| JagError::Internal(format!("Failed to write index: {}", e)))?;
        Ok(())
    }

    /// Create a commit with the current index.
    pub fn commit(&self, message: &str) -> Result<String> {
        let sig = Signature::now("Jag IDE", "jag@localhost")
            .map_err(|e| JagError::Internal(format!("Failed to create signature: {}", e)))?;

        let mut index = self.repo.index()
            .map_err(|e| JagError::Internal(format!("Failed to get index: {}", e)))?;
        let tree_oid = index.write_tree()
            .map_err(|e| JagError::Internal(format!("Failed to write tree: {}", e)))?;
        let tree = self.repo.find_tree(tree_oid)
            .map_err(|e| JagError::Internal(format!("Failed to find tree: {}", e)))?;

        // Get parent commit (if any)
        let parent = self.repo.head().ok()
            .and_then(|h| h.peel_to_commit().ok());

        let parents = match &parent {
            Some(p) => vec![p],
            None => vec![],
        };

        let oid = self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parents,
        ).map_err(|e| JagError::Internal(format!("Failed to commit: {}", e)))?;

        let commit_id = oid.to_string();
        info!(commit_id = %commit_id, message = message, "Created git commit");
        Ok(commit_id)
    }

    /// Get the diff of the working directory vs HEAD.
    pub fn diff(&self) -> Result<String> {
        let head_tree = self.repo.head().ok()
            .and_then(|h| h.peel_to_tree().ok());

        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_workdir(
            head_tree.as_ref(),
            Some(&mut diff_opts),
        ).map_err(|e| JagError::Internal(format!("Failed to create diff: {}", e)))?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                _ => "",
            };
            diff_text.push_str(&format!("{}{}", prefix, content));
            true
        }).map_err(|e| JagError::Internal(format!("Failed to format diff: {}", e)))?;

        Ok(diff_text)
    }

    /// Get commit log (most recent N commits).
    pub fn log(&self, max_count: usize) -> Result<Vec<GitLogEntry>> {
        let mut revwalk = self.repo.revwalk()
            .map_err(|e| JagError::Internal(format!("Failed to create revwalk: {}", e)))?;
        revwalk.push_head()
            .map_err(|e| JagError::Internal(format!("Failed to push HEAD: {}", e)))?;
        revwalk.set_sorting(git2::Sort::TIME)
            .map_err(|e| JagError::Internal(format!("Failed to set sorting: {}", e)))?;

        let mut entries = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= max_count {
                break;
            }
            let oid = oid.map_err(|e| JagError::Internal(format!("Failed to get OID: {}", e)))?;
            let commit = self.repo.find_commit(oid)
                .map_err(|e| JagError::Internal(format!("Failed to find commit: {}", e)))?;

            entries.push(GitLogEntry {
                id: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
                timestamp: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default(),
            });
        }

        Ok(entries)
    }

    /// List all branches.
    pub fn branch_list(&self) -> Result<Vec<String>> {
        let branches = self.repo.branches(None)
            .map_err(|e| JagError::Internal(format!("Failed to list branches: {}", e)))?;

        let mut names = Vec::new();
        for branch in branches {
            let (branch, _) = branch
                .map_err(|e| JagError::Internal(format!("Failed to get branch: {}", e)))?;
            if let Some(name) = branch.name()
                .map_err(|e| JagError::Internal(format!("Failed to get branch name: {}", e)))? {
                names.push(name.to_string());
            }
        }

        Ok(names)
    }

    /// Get the current branch name.
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()
            .map_err(|e| JagError::Internal(format!("Failed to get HEAD: {}", e)))?;
        let name = head.shorthand().unwrap_or("HEAD").to_string();
        Ok(name)
    }

    /// Get a reference to the underlying git2::Repository.
    pub fn inner(&self) -> &Repository {
        &self.repo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_repo() -> (GitRepository, std::path::PathBuf) {
        let dir = env::temp_dir().join(format!("jag-git-test-{}", uuid::Uuid::new_v4()));
        let repo = GitRepository::init(&dir).unwrap();
        (repo, dir)
    }

    #[test]
    fn test_init_and_status() {
        let (repo, dir) = temp_repo();
        let status = repo.status().unwrap();
        assert!(status.is_empty()); // Fresh repo

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_add_and_commit() {
        let (repo, dir) = temp_repo();

        // Create a file
        std::fs::write(dir.join("test.txt"), "hello").unwrap();

        // Stage and commit
        repo.add("test.txt").unwrap();
        let commit_id = repo.commit("Initial commit").unwrap();
        assert!(!commit_id.is_empty());

        // Check log
        let log = repo.log(10).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].message, "Initial commit");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_branch_operations() {
        let (repo, dir) = temp_repo();

        // Need at least one commit to have branches
        std::fs::write(dir.join("init.txt"), "init").unwrap();
        repo.add("init.txt").unwrap();
        repo.commit("init").unwrap();

        let branches = repo.branch_list().unwrap();
        assert!(!branches.is_empty());

        let current = repo.current_branch().unwrap();
        assert!(!current.is_empty());

        let _ = std::fs::remove_dir_all(dir);
    }
}
