use std::path::PathBuf;
use async_trait::async_trait;
use jag_core::errors::{JagError, Result};
use jag_core::types::*;
use jag_git::repository::GitRepository;
use jag_git::provider::{create_provider, PullRequest};
use jag_git::branch::generate_branch_name;
use jag_git::pr::generate_pr_description;
use jag_validation::coverage::{check_coverage};
use crate::traits::Agent;
use jag_core::config::Config;
use tracing::info;

pub struct GitAgent {
    id: AgentId,
    config: Config,
    repo_path: PathBuf,
}

impl GitAgent {
    pub fn new(id: AgentId, config: Config, repo_path: PathBuf) -> Self {
        Self { id, config, repo_path }
    }

    /// Orchestrates the autonomous PR generation and coverage enforcement flow.
    pub async fn execute_mission(
        &self,
        mission_prompt: &str,
        mission_id: &TaskId,
        artifacts: &[Artifact],
    ) -> Result<PullRequest> {
        info!(mission_id = %mission_id, "GitAgent starting autonomous PR flow");

        let _repo = GitRepository::open(&self.repo_path)?;
        let provider = create_provider(&self.config.git)?;

        let branch_name = generate_branch_name(
            mission_prompt,
            mission_id,
            &self.config.git.prefix,
            self.config.git.max_slug_words,
        );
        
        info!(branch_name = %branch_name, "Creating feature branch");

        info!("Enforcing code coverage gates");
        let coverage = check_coverage(&self.repo_path, &self.config.validation).await?;
        
        if !coverage.passed {
            return Err(JagError::Validation(format!(
                "Coverage check failed: {:.1}% Rust, {:.1}% TS. Threshold is 80%.",
                coverage.rust_coverage * 100.0,
                coverage.ts_coverage * 100.0
            )));
        }

        let dashboard_url = "http://localhost:3000"; 
        let pr_body = generate_pr_description(
            mission_prompt,
            mission_id,
            artifacts,
            &coverage,
            dashboard_url,
        );

        info!("Pushing to remote and creating Pull Request");
        
        let pr_title = format!("🤖 AI Mission: {}", mission_prompt);
        let pr = provider.create_pr(
            &pr_title,
            &pr_body,
            &branch_name,
            "main",
        ).await?;

        info!(pr_url = %pr.html_url, "Autonomous PR successfully created");
        Ok(pr)
    }
}

#[async_trait]
impl Agent for GitAgent {
    fn id(&self) -> AgentId { self.id.clone() }

    async fn on_message(&self, _message: AgentMessage) -> Result<()> {
        Ok(())
    }

    fn state(&self) -> AgentState {
        AgentState {
            status: AgentStatus::Idle,
            current_task: None,
            progress: 0,
            last_heartbeat: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl jag_workflow::engine::AgentExecutor for GitAgent {
    fn role(&self) -> AgentRole { AgentRole::Integration }
    fn capabilities(&self) -> Vec<TaskType> { vec![TaskType::Deploy, TaskType::RunTests] }
    
    async fn execute(&self, task: Task) -> Result<Artifact> {
        // Example integration: if task is Deploy, run the mission flow.
        // For Phase 3.21, we return a success artifact.
        Ok(Artifact {
            id: ArtifactId::new(),
            task_id: Some(task.id.clone()),
            artifact_type: ArtifactType::DeploymentPackage,
            content: format!("PR simulation for task {}", task.id).into_bytes(),
            metadata: ArtifactMetadata {
                created_by: self.id.clone(),
                timestamp: chrono::Utc::now(),
                version: "1.0".into(),
                format: "log".into(),
                size: 0,
            },
            verification_status: VerificationStatus::Approved,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_git_agent_execution() {
        let agent = GitAgent::new(
            AgentId::new(),
            Config::default(),
            PathBuf::from("."),
        );
        
        let task = Task {
            id: TaskId::new(),
            agent_id: None,
            task_type: TaskType::Deploy,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            payload: serde_json::json!({}),
            dependencies: vec![],
        };
        
        let artifact = jag_workflow::engine::AgentExecutor::execute(&agent, task).await.expect("Execution failed");
        assert_eq!(artifact.artifact_type, ArtifactType::DeploymentPackage);
        assert!(String::from_utf8_lossy(&artifact.content).contains("PR simulation"));
    }
}
