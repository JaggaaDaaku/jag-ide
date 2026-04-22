use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use jag_core::types::*;
use jag_core::errors::{JagError, Result};
use jag_models::router::{ModelRouting, ModelInput};
use jag_artifacts::generator::ArtifactGenerator;
use jag_workflow::engine::AgentExecutor;
use crate::traits::Agent;
use tracing::info;

/// Agent 1: Product Architect & System Planner
///
/// Responsibilities:
/// - Analyze project requirements → generate structured PRD
/// - Design system architecture → Mermaid diagrams
/// - Define data models → database schemas
/// - Specify APIs → OpenAPI specifications
/// - Output: Artifacts ready for Backend/Frontend Agent consumption
pub struct PlannerAgent {
    id: AgentId,
    model_router: Arc<dyn ModelRouting>,
    artifact_gen: ArtifactGenerator,
    state: Mutex<AgentState>,
}

impl PlannerAgent {
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

    /// Generate a Product Requirements Document from project description.
    async fn generate_prd(
        &self,
        project_desc: &str,
        tech_prefs: Option<&str>,
        task_id: Option<TaskId>,
    ) -> Result<Artifact> {
        let prompt = format!(
            r#"You are a Senior Product Architect. Your goal is to design a clear, actionable Product Requirements Document (PRD) for a new software project.

Project Description:
{}

Preferred Tech Stack (if any):
{}

Generate a structured PRD with these sections (in Markdown):
# Product Requirements Document

## 1. Overview
- Problem statement, target users, success metrics

## 2. Core Features
- Features with priority (Must-have / Should-have / Could-have)
- User stories (As a [user], I want [goal], so that [benefit])

## 3. Functional Requirements
- Detailed behavior, edge cases, error handling

## 4. Non-Functional Requirements
- Performance, security, scalability

## 5. Data Models
- Entity descriptions (name, attributes, relationships)

## 6. API Endpoints
- REST endpoints with method, path, request/response schema

## 7. Project Structure
- Recommended folder layout

## 8. Development Roadmap
- Phase 1 (MVP), Phase 2, Phase 3

Format: Valid Markdown. Use code blocks for schemas. Be specific."#,
            project_desc,
            tech_prefs.unwrap_or("Auto-detect optimal stack")
        );

        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::Reasoning)
            .await?;

        Ok(self.artifact_gen.create(&response.text, ArtifactType::PRD, self.id.clone(), task_id))
    }

    /// Generate system architecture diagram in Mermaid syntax.
    async fn generate_architecture(&self, prd_content: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"Based on this PRD, generate a system architecture diagram in Mermaid syntax.
 
PRD Content (excerpt):
{}
 
Generate a Mermaid graph showing:
- Major components (Frontend, Backend, Database, External APIs)
- Data flow between components
- Authentication/authorization boundaries
 
Output a valid Mermaid diagram in a code block, followed by a brief description.
 
```mermaid
graph TD
  A[Frontend] --> B[API Gateway]
  B --> C[Backend Service]
  C --> D[(Database)]
```
 
Description: ..."#,
            &prd_content.chars().take(2000).collect::<String>()
        );
 
        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::Reasoning)
            .await?;
 
        Ok(self.artifact_gen.create(&response.text, ArtifactType::ArchitectureDiagram, self.id.clone(), task_id))
    }

    /// Generate OpenAPI specification from PRD.
    async fn generate_api_spec(&self, prd_content: &str, task_id: Option<TaskId>) -> Result<Artifact> {
        let prompt = format!(
            r#"Extract the API design from this PRD and generate a valid OpenAPI 3.0 YAML specification.
 
PRD Content:
{}
 
Generate OpenAPI 3.0 YAML with:
- info: title, version, description
- servers: localhost for dev
- paths: each endpoint with method, parameters, request/response schema
- components: schemas for all data models
 
Output: Raw YAML only."#,
            &prd_content.chars().take(3000).collect::<String>()
        );
 
        let response = self.model_router
            .generate(ModelInput::Text(prompt), ModelPreference::CodeGeneration)
            .await?;
 
        Ok(self.artifact_gen.create(&response.text, ArtifactType::APISpecification, self.id.clone(), task_id))
    }
}

#[async_trait]
impl AgentExecutor for PlannerAgent {
    fn role(&self) -> AgentRole { AgentRole::Planner }

    fn capabilities(&self) -> Vec<TaskType> {
        vec![
            TaskType::GeneratePRD,
            TaskType::DesignArchitecture,
            TaskType::DefineDataModels,
            TaskType::SpecifyAPIs,
        ]
    }

    async fn execute(&self, task: Task) -> Result<Artifact> {
        self.update_state(AgentStatus::Working, Some(task.id.clone()), 0);
        info!(task_id = %task.id, task_type = ?task.task_type, "PlannerAgent executing");

        let result = match task.task_type {
            TaskType::GeneratePRD => {
                let desc = task.payload.get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'description' in payload".into()))?;
                let tech = task.payload.get("tech_stack").and_then(|v| v.as_str());
                self.update_state(AgentStatus::Working, Some(task.id.clone()), 50);
                self.generate_prd(desc, tech, Some(task.id.clone())).await
            }
            TaskType::DesignArchitecture => {
                let prd = task.payload.get("prd_content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'prd_content' in payload".into()))?;
                self.generate_architecture(prd, Some(task.id.clone())).await
            }
            TaskType::SpecifyAPIs | TaskType::DefineDataModels => {
                let prd = task.payload.get("prd_content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JagError::InvalidInput("Missing 'prd_content' in payload".into()))?;
                self.generate_api_spec(prd, Some(task.id.clone())).await
            }
            _ => Err(JagError::InvalidInput(format!("PlannerAgent cannot handle {:?}", task.task_type))),
        };

        match &result {
            Ok(_) => self.update_state(AgentStatus::Completed, None, 100),
            Err(_) => self.update_state(AgentStatus::Error, None, 0),
        }

        result
    }
}

#[async_trait]
impl Agent for PlannerAgent {
    fn id(&self) -> AgentId { self.id.clone() }

    async fn on_message(&self, _message: AgentMessage) -> Result<()> {
        // Planner listens for iteration requests (deferred to Phase 3)
        Ok(())
    }

    fn state(&self) -> AgentState {
        self.state.lock()
            .map(|s| s.clone())
            .unwrap_or(AgentState {
                status: AgentStatus::Error,
                current_task: None,
                progress: 0,
                last_heartbeat: chrono::Utc::now(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jag_models::ollama_client::OllamaClient;
    use jag_models::router::ModelRouter;

    #[test]
    fn test_planner_capabilities() {
        let client = OllamaClient::new("http://localhost:11434");
        let router = Arc::new(ModelRouter::new(client));
        let planner = PlannerAgent::new(router);

        let caps = AgentExecutor::capabilities(&planner);
        assert_eq!(caps.len(), 4);
        assert!(caps.contains(&TaskType::GeneratePRD));
        assert!(caps.contains(&TaskType::DesignArchitecture));
    }

    #[test]
    fn test_planner_initial_state() {
        let client = OllamaClient::new("http://localhost:11434");
        let router = Arc::new(ModelRouter::new(client));
        let planner = PlannerAgent::new(router);

        let state = Agent::state(&planner);
        assert_eq!(state.status, AgentStatus::Idle);
        assert_eq!(state.progress, 0);
    }

    #[tokio::test]
    async fn test_planner_prd_generation() {
        use jag_models::mock::MockModelRouter;
        
        let mock_router = MockModelRouter::new()
            .with_response("Senior Product Architect", "# Mock PRD\n\n## Overview\nThis is a test.").await;
        
        let planner = PlannerAgent::new(Arc::new(mock_router));
        
        let task = Task {
            id: TaskId::new(),
            agent_id: None,
            task_type: TaskType::GeneratePRD,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            payload: serde_json::json!({
                "description": "Build a test app",
                "tech_stack": "Rust"
            }),
            dependencies: vec![],
        };
        
        let artifact = AgentExecutor::execute(&planner, task).await.expect("Execution failed");
        assert_eq!(artifact.artifact_type, ArtifactType::PRD);
        assert!(String::from_utf8_lossy(&artifact.content).contains("# Mock PRD"));
    }
}
