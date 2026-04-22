use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use jag_core::types::*;
use jag_core::errors::{JagError, Result};
use jag_models::router::{ModelRouting, ModelInput};
use jag_artifacts::generator::ArtifactGenerator;
use jag_workflow::engine::AgentExecutor;
use crate::traits::Agent;
use tracing::info;

/// Agent 3: Frontend Developer
///
/// Consumes architecture/PRD artifacts and generates UI code.
pub struct FrontendAgent {
    id: AgentId,
    model_router: Arc<dyn ModelRouting>,
    artifact_gen: ArtifactGenerator,
    state: Mutex<AgentState>,
}

impl FrontendAgent {
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

    async fn build_ui(&self, architecture: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"You are a senior frontend engineer. Given this architecture, generate a complete React/TypeScript page layout.

Architecture:
{}

Generate:
1. Page component with proper layout (CSS Grid or Flexbox)
2. TypeScript interfaces for all props and state
3. Responsive design breakpoints

Output clean React/TypeScript code."#,
            &architecture.chars().take(4000).collect::<String>()
        );

        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;

        Ok(self.artifact_gen.create(&response.text, ArtifactType::FrontendCode, self.id.clone(), task_id))
    }

    async fn generate_components(&self, spec: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"Generate reusable React/TypeScript UI components from this specification:

{}

Output production-ready TypeScript React code."#,
            &spec.chars().take(3000).collect::<String>()
        );

        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;

        Ok(self.artifact_gen.create(&response.text, ArtifactType::FrontendCode, self.id.clone(), task_id))
    }

    async fn generate_styles(&self, spec: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"Generate a CSS design system from this specification:

{}

Output valid CSS code."#,
            &spec.chars().take(2000).collect::<String>()
        );

        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;

        Ok(self.artifact_gen.create(&response.text, ArtifactType::FrontendCode, self.id.clone(), task_id))
    }
}

#[async_trait]
impl AgentExecutor for FrontendAgent {
    fn role(&self) -> AgentRole { AgentRole::Frontend }

    fn capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::BuildUI, TaskType::GenerateComponents, TaskType::GenerateStyles]
    }

    async fn execute(&self, task: Task) -> Result<Artifact> {
        self.update_state(AgentStatus::Working, Some(task.id.clone()), 0);
        info!(task_id = %task.id, task_type = ?task.task_type, "FrontendAgent executing");

        let result = match task.task_type {
            TaskType::BuildUI => {
                let arch = task.payload.get("architecture")
                    .or_else(|| task.payload.get("prd_content"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'architecture' or 'prd_content'".into()))?;
                self.build_ui(arch, Some(task.id.clone())).await
            }
            TaskType::GenerateComponents => {
                let spec = task.payload.get("component_spec")
                    .or_else(|| task.payload.get("prd_content"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'component_spec' or 'prd_content'".into()))?;
                self.generate_components(spec, Some(task.id.clone())).await
            }
            TaskType::GenerateStyles => {
                let spec = task.payload.get("style_spec")
                    .or_else(|| task.payload.get("prd_content"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'style_spec' or 'prd_content'".into()))?;
                self.generate_styles(spec, Some(task.id.clone())).await
            }
            _ => Err(JagError::InvalidInput(format!("FrontendAgent cannot handle {:?}", task.task_type))),
        };

        match &result {
            Ok(_) => self.update_state(AgentStatus::Completed, None, 100),
            Err(_) => self.update_state(AgentStatus::Error, None, 0),
        }

        result
    }
}

#[async_trait]
impl Agent for FrontendAgent {
    fn id(&self) -> AgentId { self.id.clone() }
    async fn on_message(&self, _message: AgentMessage) -> Result<()> { Ok(()) }
    fn state(&self) -> AgentState {
        self.state.lock().map(|s| s.clone()).unwrap_or(AgentState {
            status: AgentStatus::Error, current_task: None, progress: 0, last_heartbeat: chrono::Utc::now(),
        })
    }
}
