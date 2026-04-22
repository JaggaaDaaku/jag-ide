use jag_core::types::{Artifact, ArtifactId, ArtifactMetadata, ArtifactType, AgentId, VerificationStatus, TaskId};
use chrono::Utc;

/// Creates `Artifact` structs from raw content.
///
/// This is a factory that stamps the correct metadata (timestamp, size, format)
/// based on the artifact type. It does NOT persist — use `ArtifactStore` for that.
pub struct ArtifactGenerator;

impl ArtifactGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Create a new artifact from raw string content.
    pub fn create(
        &self,
        content: &str,
        artifact_type: ArtifactType,
        agent_id: AgentId,
        task_id: Option<TaskId>,
    ) -> Artifact {
        let format = Self::format_for_type(&artifact_type);
        let bytes = content.as_bytes().to_vec();
        let size = bytes.len();

        Artifact {
            id: ArtifactId::new(),
            task_id,
            artifact_type,
            content: bytes,
            metadata: ArtifactMetadata {
                created_by: agent_id,
                timestamp: Utc::now(),
                version: "1.0".into(),
                format,
                size,
            },
            verification_status: VerificationStatus::Pending,
        }
    }

    /// Determine the file format string based on artifact type.
    fn format_for_type(artifact_type: &ArtifactType) -> String {
        match artifact_type {
            ArtifactType::PRD => "markdown".into(),
            ArtifactType::ArchitectureDiagram => "mermaid+markdown".into(),
            ArtifactType::APISpecification => "yaml".into(),
            ArtifactType::DatabaseSchema => "sql".into(),
            ArtifactType::BackendCode => "rust".into(),
            ArtifactType::FrontendCode => "typescript".into(),
            ArtifactType::TestReport => "markdown".into(),
            ArtifactType::DeploymentPackage => "tar.gz".into(),
            ArtifactType::CodeDiff => "diff".into(),
        }
    }

    /// Determine the file extension based on artifact type.
    pub fn extension_for_type(artifact_type: &ArtifactType) -> &'static str {
        match artifact_type {
            ArtifactType::PRD => "md",
            ArtifactType::ArchitectureDiagram => "md",
            ArtifactType::APISpecification => "yaml",
            ArtifactType::DatabaseSchema => "sql",
            ArtifactType::BackendCode => "rs",
            ArtifactType::FrontendCode => "tsx",
            ArtifactType::TestReport => "md",
            ArtifactType::DeploymentPackage => "tar.gz",
            ArtifactType::CodeDiff => "diff",
        }
    }
}

impl Default for ArtifactGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_prd_artifact() {
        let generator = ArtifactGenerator::new();
        let agent = AgentId::new();
        let art = generator.create("# My PRD\n\nSome content", ArtifactType::PRD, agent.clone(), None);

        assert_eq!(art.artifact_type, ArtifactType::PRD);
        assert_eq!(art.metadata.format, "markdown");
        assert_eq!(art.metadata.created_by, agent);
        assert_eq!(art.verification_status, VerificationStatus::Pending);
        assert_eq!(String::from_utf8(art.content).unwrap(), "# My PRD\n\nSome content");
    }

    #[test]
    fn test_create_api_spec_artifact() {
        let generator = ArtifactGenerator::new();
        let art = generator.create("openapi: 3.0.0", ArtifactType::APISpecification, AgentId::new(), None);

        assert_eq!(art.metadata.format, "yaml");
        assert_eq!(art.metadata.size, 14);
    }

    #[test]
    fn test_extension_mapping() {
        assert_eq!(ArtifactGenerator::extension_for_type(&ArtifactType::PRD), "md");
        assert_eq!(ArtifactGenerator::extension_for_type(&ArtifactType::APISpecification), "yaml");
        assert_eq!(ArtifactGenerator::extension_for_type(&ArtifactType::BackendCode), "rs");
        assert_eq!(ArtifactGenerator::extension_for_type(&ArtifactType::CodeDiff), "diff");
    }
}
