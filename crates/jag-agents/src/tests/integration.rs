use std::sync::Arc;
use jag_core::types::*;
use jag_workflow::engine::AgentExecutor;
use jag_models::mock::MockModelRouter;
use crate::planner::PlannerAgent;
use crate::backend_agent::BackendAgent;

#[tokio::test]
async fn test_planner_to_backend_workflow() {
    let mock_router = MockModelRouter::new()
        .with_response("Senior Product Architect", "# Test PRD").await
        .with_response("database models", "pub fn generated_code() {}").await;
    
    let router_arc = Arc::new(mock_router);
    let planner = PlannerAgent::new(router_arc.clone());
    let backend = BackendAgent::new(router_arc.clone());
    
    // 1. Planner generates PRD
    let planner_task = Task {
        id: TaskId::new(),
        agent_id: None,
        task_type: TaskType::GeneratePRD,
        status: TaskStatus::Pending,
        priority: Priority::High,
        payload: serde_json::json!({ "description": "My App" }),
        dependencies: vec![],
    };
    
    let prd_artifact = AgentExecutor::execute(&planner, planner_task.clone()).await.expect("Planner failed");
    assert_eq!(prd_artifact.artifact_type, ArtifactType::PRD);
    let prd_content = String::from_utf8_lossy(&prd_artifact.content).to_string();
    
    // 2. Backend consumes PRD
    let backend_task = Task {
        id: TaskId::new(),
        agent_id: None,
        task_type: TaskType::GenerateModels,
        status: TaskStatus::Pending,
        priority: Priority::High,
        payload: serde_json::json!({ "prd_content": prd_content }),
        dependencies: vec![planner_task.id.clone()],
    };
    
    let code_artifact = AgentExecutor::execute(&backend, backend_task).await.expect("Backend failed");
    assert_eq!(code_artifact.artifact_type, ArtifactType::DatabaseSchema);
    assert!(String::from_utf8_lossy(&code_artifact.content).contains("generated_code"));
}
