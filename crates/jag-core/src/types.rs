use std::collections::HashMap;
use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// -----------------------------------------------------------------------------
// ID Types
// -----------------------------------------------------------------------------

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(AgentId);
define_id!(TeamId);
define_id!(TaskId);
define_id!(ArtifactId);
define_id!(WorkspaceId);
define_id!(ProjectId);
define_id!(MessageId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ModelId(pub String);

impl Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ProviderId(pub String);

impl Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -----------------------------------------------------------------------------
// Enums
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRole {
    Planner,
    Backend,
    Frontend,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Working,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    GeneratePRD,
    DesignArchitecture,
    DefineDataModels,
    SpecifyAPIs,
    ImplementAPI,
    GenerateModels,
    ImplementAuth,
    BuildUI,
    GenerateComponents,
    GenerateStyles,
    IntegrateAPI,
    Integrate,
    RunTests,
    Deploy,
    GenerateReadme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactType {
    PRD,
    ArchitectureDiagram,
    APISpecification,
    DatabaseSchema,
    BackendCode,
    FrontendCode,
    TestReport,
    DeploymentPackage,
    CodeDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityTier {
    Off,
    Auto,
    Turbo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    Hard,
    Soft,
    Parallel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelPreference {
    Reasoning,
    CodeGeneration,
    Fast,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelProvider {
    Local,
    Cloud(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    ArtifactReady(ArtifactType),
    TaskRequest(TaskType),
    StatusUpdate(AgentStatus),
    DependencyResolved(TaskId),
    ErrorReport(String),
}

// -----------------------------------------------------------------------------
// Structs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: MessageId,
    pub from: AgentId,
    pub to: Option<AgentId>,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub priority: Priority,
    pub correlation_id: Option<MessageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub artifact_id: Option<ArtifactId>,
    pub task_id: Option<TaskId>,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub agent_id: Option<AgentId>,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: Priority,
    pub payload: serde_json::Value,
    pub dependencies: Vec<TaskId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub artifact_type: ArtifactType,
    pub content: Vec<u8>,
    pub metadata: ArtifactMetadata,
    pub verification_status: VerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub created_by: AgentId,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub format: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub root_path: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub agents: Vec<AgentId>,
    pub artifacts: Vec<ArtifactId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub current_task: Option<TaskId>,
    pub progress: u8,
    pub last_heartbeat: DateTime<Utc>,
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_serialization() {
        let id = AgentId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: AgentId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_task_serialization() {
        let task = Task {
            id: TaskId::new(),
            agent_id: Some(AgentId::new()),
            task_type: TaskType::GeneratePRD,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            payload: serde_json::json!({"key": "value"}),
            dependencies: vec![TaskId::new()],
        };

        let serialized = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.task_type, deserialized.task_type);
    }
}
