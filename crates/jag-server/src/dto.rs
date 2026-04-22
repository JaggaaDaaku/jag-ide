use serde::{Deserialize, Serialize};
use jag_core::types::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub agents: Vec<AgentStateDto>,
    pub workflow: WorkflowStatusDto,
    pub recent_artifacts: Vec<ArtifactRecord>,
    pub available_models: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnterpriseAnalytics {
    pub total_tokens_consumed: i64,
    pub total_cost_estimated: f64,
    pub model_distribution: serde_json::Value,
    pub audit_summary: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStateDto {
    pub role: AgentRole,
    pub status: AgentStatus,
    pub progress: u8,
    pub current_task: Option<TaskId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatusDto {
    pub is_complete: bool,
    pub has_failures: bool,
    pub status_counts: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactMetadataDto {
    pub id: ArtifactId,
    pub artifact_type: ArtifactType,
    pub created_by: AgentId,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub format: String,
    pub size: usize,
    pub verification_status: VerificationStatus,
}

#[derive(Debug, Deserialize)]
pub struct StartWorkflowRequest {
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTaskRequest {
    pub task_type: TaskType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalDto {
    pub id: ArtifactId,
    pub workflow_id: TaskId,
    pub task_id: TaskId,
    pub decision: ApprovalDecision,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalDecisionRequest {
    pub status: VerificationStatus, // Approved or Rejected
    pub comments: Option<String>,
}
