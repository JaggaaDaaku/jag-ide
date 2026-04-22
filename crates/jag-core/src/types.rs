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

        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
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
define_id!(UserId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ModelId(pub String);

impl Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    Admin,
    Developer,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: UserId,
    pub roles: Vec<UserRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageRecord {
    pub model_name: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub cost_estimated: f64,
    pub timestamp: DateTime<Utc>,
    pub is_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub id: TaskId,
    pub model_name: String,
    pub task_type: String,
    pub latency_ms: u64,
    pub tokens_per_second: f64,
    pub total_tokens: u32,
    pub cost_usd: f64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Denied(String),
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub workspace_id: Option<WorkspaceId>,
    pub user_id: Option<UserId>,
    pub agent_id: Option<AgentId>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub signature: String,
}

/// Rich response from a model containing text and token usage metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub model_name: String,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseAnalytics {
    pub total_tokens_consumed: i64,
    pub total_cost_estimated: f64,
    pub model_distribution: serde_json::Value,
    pub audit_summary: serde_json::Value,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalDecision {
    /// Score ≥ 90: Fully auto-approve, no human review needed
    AutoApprove { confidence: f32, reasoning: String },
    
    /// Score 80-89: Approve but flag for optional review (async notification)
    ApproveWithNotice { 
        confidence: f32, 
        reasoning: String,
        review_deadline: Option<DateTime<Utc>>,
    },
    
    /// Score < 80: Require explicit human approval before merge/deploy
    RequiresApproval { 
        confidence: f32, 
        reasoning: String,
        suggested_fixes: Vec<String>,
    },
}

impl ApprovalDecision {
    pub fn from_score(score: f32, reasoning: String, suggested_fixes: Vec<String>) -> Self {
        if score >= 90.0 {
            ApprovalDecision::AutoApprove { confidence: score, reasoning }
        } else if score >= 80.0 {
            ApprovalDecision::ApproveWithNotice {
                confidence: score,
                reasoning,
                review_deadline: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            }
        } else {
            ApprovalDecision::RequiresApproval {
                confidence: score,
                reasoning,
                suggested_fixes,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
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
    pub task_id: Option<TaskId>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceRole {
    Admin,
    Developer,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub workspace_id: WorkspaceId,
    pub user_id: UserId,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: ArtifactId,
    pub task_id: Option<TaskId>,
    pub artifact_type: String,
    pub file_path: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BrowserEngine {
    Playwright,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub engine: BrowserEngine,
    pub navigation_timeout_ms: u64,
    pub test_timeout_ms: u64,
    pub headless: bool,
    pub viewport: ViewportSpec,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            engine: BrowserEngine::Playwright,
            navigation_timeout_ms: 30000,
            test_timeout_ms: 60000,
            headless: true,
            viewport: ViewportSpec::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSpec {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
}

impl Default for ViewportSpec {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            device_scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSpec {
    pub description: String,
    pub image_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub current_task: Option<TaskId>,
    pub progress: u8,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub rust_coverage: f32, // 0.0 to 1.0
    pub ts_coverage: f32,
    pub passed: bool,
    pub details: String,
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
