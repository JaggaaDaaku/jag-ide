use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use jag_core::types::*;
use jag_core::errors::{JagError, Result};
use jag_models::router::{ModelRouting, ModelInput};
use jag_artifacts::generator::ArtifactGenerator;
use jag_workflow::engine::AgentExecutor;
use crate::traits::Agent;
use tracing::info;

/// Agent 4: Integration Engineer
///
/// Orchestrates cross-agent integration: wires frontend ↔ backend,
/// generates test suites, deployment configs, and project documentation.
pub struct IntegrationAgent {
    id: AgentId,
    model_router: Arc<dyn ModelRouting>,
    artifact_gen: ArtifactGenerator,
    state: Mutex<AgentState>,
}

impl IntegrationAgent {
    pub fn new(model_router: Arc<dyn ModelRouting>) -> Self {
        Self {
            id: AgentId::new(),
            model_router,
            artifact_gen: ArtifactGenerator::new(),
            state: Mutex::new(AgentState {
                status: AgentStatus::Idle,
                current_task: None,
                progress: 0,
                last_heartbeat: chrono::Utc::now(),
            }),
        }
    }

    fn update_state(&self, status: AgentStatus, task: Option<TaskId>, progress: u8) {
        if let Ok(mut state) = self.state.lock() {
            state.status = status;
            state.current_task = task;
            state.progress = progress;
            state.last_heartbeat = chrono::Utc::now();
        }
    }

    async fn integrate(&self, backend: &str, frontend: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!("Wire together backend and frontend:\n\nBackend:\n{}\n\nFrontend:\n{}", backend, frontend);
        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::Reasoning)
            .await?;
        Ok(self.artifact_gen.create(&response.text, ArtifactType::CodeDiff, self.id.clone(), task_id))
    }

    async fn generate_tests(&self, code_context: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!("Generate tests for:\n\n{}", code_context);
        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;
        Ok(self.artifact_gen.create(&response.text, ArtifactType::TestReport, self.id.clone(), task_id))
    }

    async fn generate_readme(&self, project_desc: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!("Generate a professional README.md for:\n\n{}", project_desc);
        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::Fast)
            .await?;
        Ok(self.artifact_gen.create(&response.text, ArtifactType::PRD, self.id.clone(), task_id))
    }

    async fn generate_deploy_config(&self, project_desc: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!("Generate deployment configuration for:\n\n{}", project_desc);
        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::Fast)
            .await?;
        Ok(self.artifact_gen.create(&response.text, ArtifactType::DeploymentPackage, self.id.clone(), task_id))
    }
}

#[async_trait]
impl AgentExecutor for IntegrationAgent {
    fn role(&self) -> AgentRole { AgentRole::Integration }

    fn capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::Integrate, TaskType::RunTests, TaskType::GenerateReadme, TaskType::Deploy]
    }

    async fn execute(&self, task: Task) -> Result<Artifact> {
        self.update_state(AgentStatus::Working, Some(task.id.clone()), 0);
        info!(task_id = %task.id, task_type = ?task.task_type, "IntegrationAgent executing");

        let result = match task.task_type {
            TaskType::Integrate => {
                let backend = task.payload.get("backend_code").and_then(|v| v.as_str()).unwrap_or("");
                let frontend = task.payload.get("frontend_code").and_then(|v| v.as_str()).unwrap_or("");
                self.integrate(backend, frontend, Some(task.id.clone())).await
            }
            TaskType::RunTests => {
                let code = task.payload.get("code_context").and_then(|v| v.as_str()).unwrap_or("");
                self.generate_tests(code, Some(task.id.clone())).await
            }
            TaskType::GenerateReadme => {
                let desc = task.payload.get("prd_content")
                    .or_else(|| task.payload.get("description"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'description' or 'prd_content'".into()))?;
                self.generate_readme(desc, Some(task.id.clone())).await
            }
            TaskType::Deploy => {
                let desc = task.payload.get("prd_content")
                    .or_else(|| task.payload.get("description"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'description' or 'prd_content'".into()))?;
                self.generate_deploy_config(desc, Some(task.id.clone())).await
            }
            _ => Err(JagError::InvalidInput(format!("IntegrationAgent cannot handle {:?}", task.task_type))),
        };

        match &result {
            Ok(_) => self.update_state(AgentStatus::Completed, None, 100),
            Err(_) => self.update_state(AgentStatus::Error, None, 0),
        }

        result
    }
}

#[async_trait]
impl Agent for IntegrationAgent {
    fn id(&self) -> AgentId { self.id.clone() }
    async fn on_message(&self, _message: AgentMessage) -> Result<()> { Ok(()) }
    fn state(&self) -> AgentState {
        self.state.lock().map(|s| s.clone()).unwrap_or(AgentState {
            status: AgentStatus::Error, current_task: None, progress: 0, last_heartbeat: chrono::Utc::now(),
        })
    }
}
