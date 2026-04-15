use crate::types::{
    AgentId, ArtifactId, ModelPreference, TaskId, WorkspaceId,
};

/// Central error type for all Jag IDE operations.
/// Each variant corresponds to a distinct failure domain.
#[derive(Debug, thiserror::Error)]
pub enum JagError {
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),

    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(ArtifactId),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(WorkspaceId),

    #[error("Approval required for this operation")]
    ApprovalRequired,

    #[error("Command denied by security policy: {0}")]
    CommandDenied(String),

    #[error("Circular dependency detected in workflow")]
    CircularDependency,

    #[error("Context window exceeded: {used}/{max} tokens")]
    ContextWindowExceeded { used: usize, max: usize },

    #[error("No model available for preference: {0:?}")]
    NoModelAvailable(ModelPreference),

    #[error("Path traversal attempted: {0}")]
    PathTraversal(String),

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Agent communication error: {0}")]
    CommunicationError(String),

    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Catch-all for errors that don't fit a specific variant.
    /// Note: anyhow::Error cannot use #[from] alongside other From impls
    /// due to blanket impl overlap — wrap manually via `.map_err(JagError::Internal)`.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience alias so callers write `jag_core::errors::Result<T>`.
pub type Result<T> = std::result::Result<T, JagError>;

// Manual From<anyhow::Error> since thiserror's #[from] conflicts with blanket impls
impl From<anyhow::Error> for JagError {
    fn from(err: anyhow::Error) -> Self {
        JagError::Internal(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // --- From conversions ---

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let jag_err = JagError::from(io_err);
        assert!(matches!(jag_err, JagError::Io(_)));
        assert!(jag_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let jag_err = JagError::from(json_err);
        assert!(matches!(jag_err, JagError::Serialization(_)));
        assert!(jag_err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something went wrong deeply");
        let jag_err = JagError::from(anyhow_err);
        assert!(matches!(jag_err, JagError::Internal(_)));
        assert!(jag_err.to_string().contains("Internal error"));
        assert!(jag_err.to_string().contains("something went wrong deeply"));
    }

    // --- Display output ---

    #[test]
    fn test_display_approval_required() {
        let err = JagError::ApprovalRequired;
        assert_eq!(err.to_string(), "Approval required for this operation");
    }

    #[test]
    fn test_display_command_denied() {
        let err = JagError::CommandDenied("rm -rf /".to_string());
        assert_eq!(
            err.to_string(),
            "Command denied by security policy: rm -rf /"
        );
    }

    #[test]
    fn test_display_circular_dependency() {
        let err = JagError::CircularDependency;
        assert_eq!(err.to_string(), "Circular dependency detected in workflow");
    }

    #[test]
    fn test_display_context_window_exceeded() {
        let err = JagError::ContextWindowExceeded {
            used: 8500,
            max: 8192,
        };
        assert_eq!(err.to_string(), "Context window exceeded: 8500/8192 tokens");
    }

    #[test]
    fn test_display_path_traversal() {
        let err = JagError::PathTraversal("../../etc/passwd".to_string());
        assert_eq!(err.to_string(), "Path traversal attempted: ../../etc/passwd");
    }

    #[test]
    fn test_display_timeout() {
        let err = JagError::Timeout(std::time::Duration::from_secs(30));
        assert!(err.to_string().contains("Timeout after"));
        assert!(err.to_string().contains("30s"));
    }

    #[test]
    fn test_display_resource_limit() {
        let err = JagError::ResourceLimitExceeded("memory: 512MB".to_string());
        assert_eq!(
            err.to_string(),
            "Resource limit exceeded: memory: 512MB"
        );
    }

    #[test]
    fn test_display_encryption_decryption() {
        assert_eq!(JagError::EncryptionFailed.to_string(), "Encryption failed");
        assert_eq!(JagError::DecryptionFailed.to_string(), "Decryption failed");
    }

    #[test]
    fn test_display_communication_error() {
        let err = JagError::CommunicationError("channel closed".to_string());
        assert_eq!(
            err.to_string(),
            "Agent communication error: channel closed"
        );
    }

    // --- std::error::Error implementation ---

    #[test]
    fn test_implements_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(JagError::ApprovalRequired);
        assert!(err.to_string().len() > 0);
    }
}
