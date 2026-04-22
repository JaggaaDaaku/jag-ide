use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use jag_core::types::*;
use jag_core::errors::{JagError, Result};
use jag_models::router::{ModelRouting, ModelInput};
use jag_artifacts::generator::ArtifactGenerator;
use jag_workflow::engine::AgentExecutor;
use crate::traits::Agent;
use tracing::info;

/// Agent 2: Backend Developer
///
/// Consumes PRD/API spec artifacts and generates backend code.
pub struct BackendAgent {
    id: AgentId,
    model_router: Arc<dyn ModelRouting>,
    artifact_gen: ArtifactGenerator,
    state: Mutex<AgentState>,
}

impl BackendAgent {
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

    async fn implement_api(&self, spec: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"You are a senior backend engineer. Given this API specification, generate production-ready code.

API Specification:
{}

Generate:
1. Route handlers for each endpoint
2. Request/response types with validation
3. Database query functions
4. Error handling middleware

Output production-ready code."#,
            &spec.chars().take(4000).collect::<String>()
        );

        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;

        Ok(self.artifact_gen.create(&response.text, ArtifactType::DatabaseSchema, self.id.clone(), task_id))
    }

    async fn generate_models(&self, schema: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"Generate database models and schema from this data model description:

{}

Generate:
1. Database migration SQL
2. ORM model structs
3. CRUD operations

Output production-ready code."#,
            &schema.chars().take(3000).collect::<String>()
        );

        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;

        Ok(self.artifact_gen.create(&response.text, ArtifactType::DatabaseSchema, self.id.clone(), task_id))
    }
}

#[async_trait]
impl AgentExecutor for BackendAgent {
    fn role(&self) -> AgentRole { AgentRole::Backend }

    fn capabilities(&self) -> Vec<TaskType> {
        vec![TaskType::ImplementAPI, TaskType::GenerateModels, TaskType::ImplementAuth]
    }

    async fn execute(&self, task: Task) -> Result<Artifact> {
        self.update_state(AgentStatus::Working, Some(task.id.clone()), 0);
        info!(task_id = %task.id, task_type = ?task.task_type, "BackendAgent executing");

        let result = match task.task_type {
            TaskType::ImplementAPI => {
                let spec = task.payload.get("api_spec")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'api_spec'".into()))?;
                self.implement_api(spec, Some(task.id.clone())).await
            }
            TaskType::GenerateModels | TaskType::ImplementAuth => {
                let schema = task.payload.get("schema")
                    .or_else(|| task.payload.get("prd_content"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'schema' or 'prd_content'".into()))?;
                self.generate_models(schema, Some(task.id.clone())).await
            }
            _ => Err(JagError::InvalidInput(format!("BackendAgent cannot handle {:?}", task.task_type))),
        };

        match &result {
            Ok(_) => self.update_state(AgentStatus::Completed, None, 100),
            Err(_) => self.update_state(AgentStatus::Error, None, 0),
        }

        result
    }
}

#[async_trait]
impl Agent for BackendAgent {
    fn id(&self) -> AgentId { self.id.clone() }
    async fn on_message(&self, _message: AgentMessage) -> Result<()> { Ok(()) }
    fn state(&self) -> AgentState {
        self.state.lock().map(|s| s.clone()).unwrap_or(AgentState {
            status: AgentStatus::Error, current_task: None, progress: 0, last_heartbeat: chrono::Utc::now(),
        })
    }
}
