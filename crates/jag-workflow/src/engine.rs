use std::collections::HashMap;
use async_trait::async_trait;
use jag_core::types::*;
use jag_core::errors::Result;
use crate::dag::WorkflowDag;
use tracing::{info, warn, error};
use std::sync::Arc;
use jag_db::Database;

/// Trait for agent execution — agents must implement this to be usable by the engine.
/// Defined here to avoid circular dependency (jag-agents depends on jag-workflow).
use prometheus::{IntGauge, opts, register_int_gauge};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ACTIVE_MISSIONS: IntGauge = register_int_gauge!(
        opts!("jag_active_missions_total", "Number of active missions")
    ).unwrap();
}

#[async_trait]
pub trait AgentExecutor: Send + Sync {
    fn role(&self) -> AgentRole;
    fn capabilities(&self) -> Vec<TaskType>;
    async fn execute(&self, task: Task) -> Result<Artifact>;
}

/// Executes workflows by dispatching ready tasks to registered agents.
pub struct WorkflowEngine {
    dag: WorkflowDag,
    agents: HashMap<AgentRole, Box<dyn AgentExecutor>>,
    #[allow(dead_code)]
    db: Option<Arc<Database>>,
}

impl WorkflowEngine {
    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        task: &Task,
        config: RetryConfig,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut delay = std::time::Duration::from_millis(config.initial_delay_ms);
        let mut last_error = None;

        for attempt in 1..=config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!(task_id = %task.id, attempt, max = config.max_attempts, error = %e, "Task attempt failed");
                    last_error = Some(e);

                    if attempt < config.max_attempts {
                        tokio::time::sleep(delay).await;
                        delay = std::time::Duration::from_millis(
                            (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                        )
                        .min(std::time::Duration::from_millis(config.max_delay_ms));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            jag_core::errors::JagError::Internal("Retry loop exited without result".into())
        }))
    }

    pub fn new(dag: WorkflowDag) -> Self {
        Self {
            dag,
            agents: HashMap::new(),
            db: None,
        }
    }

    pub fn with_db(dag: WorkflowDag, db: Arc<Database>) -> Self {
        Self {
            dag,
            agents: HashMap::new(),
            db: Some(db),
        }
    }

    /// Hydrate an engine from a persistent workflow in the database.
    pub async fn from_db(workflow_id: TaskId, db: Arc<Database>) -> Result<Self> {
        let tasks = db.get_workflow_tasks(&workflow_id).await?;
        let mut dag = WorkflowDag::new();
        
        for task in tasks {
            dag.add_task(task);
        }
        
        // Note: Task dependencies would also need to be hydrated here.
        // For now, this establishes the basic pattern.
        
        Ok(Self::with_db(dag, db))
    }

    /// Register an agent for a specific role.
    pub fn register_agent(&mut self, role: AgentRole, agent: Box<dyn AgentExecutor>) {
        self.agents.insert(role, agent);
    }

    /// Run one tick of the workflow engine.
    ///
    /// Finds ready tasks, dispatches them to appropriate agents,
    /// and updates their status based on results.
    ///
    /// Returns the list of artifacts produced in this tick.
    pub async fn tick(&mut self) -> Result<Vec<Artifact>> {
        let ready_tasks: Vec<Task> = self.dag.get_ready_tasks()
            .iter()
            .map(|t| (*t).clone())
            .collect();

        if ready_tasks.is_empty() {
            return Ok(vec![]);
        }

        info!(count = ready_tasks.len(), "Dispatching ready tasks");

        let mut artifacts = Vec::new();

        for task in ready_tasks {
            let agent_role = Self::role_for_task_type(&task.task_type);

            let agent: &dyn AgentExecutor = match self.agents.get(&agent_role) {
                Some(a) => a.as_ref(),
                None => {
                    warn!(role = ?agent_role, task_id = %task.id, "No agent registered for role");
                    self.dag.update_task_status(&task.id, TaskStatus::Failed)?;
                    continue;
                }
            };

            // Mark as running
            self.dag.update_task_status(&task.id, TaskStatus::Running)?;
            info!(task_id = %task.id, task_type = ?task.task_type, role = ?agent_role, "Executing task");

            match agent.execute(task.clone()).await {
                Ok(artifact) => {
                    info!(task_id = %task.id, artifact_id = %artifact.id, "Task completed successfully");
                    self.dag.update_task_status(&task.id, TaskStatus::Completed)?;
                    artifacts.push(artifact);
                }
                Err(e) => {
                    error!(task_id = %task.id, error = %e, "Task execution failed");
                    self.dag.update_task_status(&task.id, TaskStatus::Failed)?;
                }
            }
        }

        Ok(artifacts)
    }

    /// Run the workflow to completion (or until stuck).
    ///
    /// Returns all artifacts produced during the workflow.
    pub async fn run_to_completion(&mut self) -> Result<Vec<Artifact>> {
        let mut all_artifacts = Vec::new();

        loop {
            if self.dag.is_complete() {
                info!("Workflow complete");
                break;
            }

            if self.dag.has_failures() {
                warn!("Workflow has failures — checking if remaining tasks can proceed");
            }

            let artifacts: Vec<Artifact> = self.tick().await?;
            if artifacts.is_empty() {
                // No progress possible — either complete or stuck
                if !self.dag.is_complete() {
                    warn!(counts = ?self.dag.status_counts(), "Workflow stuck — no ready tasks");
                }
                break;
            }
            all_artifacts.extend(artifacts);
        }

        Ok(all_artifacts)
    }

    /// Determine which agent role handles a given task type.
    pub fn role_for_task_type(task_type: &TaskType) -> AgentRole {
        match task_type {
            TaskType::GeneratePRD
            | TaskType::DesignArchitecture
            | TaskType::DefineDataModels
            | TaskType::SpecifyAPIs => AgentRole::Planner,

            TaskType::ImplementAPI
            | TaskType::GenerateModels
            | TaskType::ImplementAuth => AgentRole::Backend,

            TaskType::BuildUI
            | TaskType::GenerateComponents
            | TaskType::GenerateStyles => AgentRole::Frontend,

            TaskType::IntegrateAPI
            | TaskType::Integrate
            | TaskType::RunTests
            | TaskType::Deploy
            | TaskType::GenerateReadme => AgentRole::Integration,
        }
    }

    /// Get the current DAG state.
    pub fn dag(&self) -> &WorkflowDag {
        &self.dag
    }

    /// Get mutable access to the DAG.
    pub fn dag_mut(&mut self) -> &mut WorkflowDag {
        &mut self.dag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_mapping() {
        assert_eq!(WorkflowEngine::role_for_task_type(&TaskType::GeneratePRD), AgentRole::Planner);
        assert_eq!(WorkflowEngine::role_for_task_type(&TaskType::ImplementAPI), AgentRole::Backend);
        assert_eq!(WorkflowEngine::role_for_task_type(&TaskType::BuildUI), AgentRole::Frontend);
        assert_eq!(WorkflowEngine::role_for_task_type(&TaskType::RunTests), AgentRole::Integration);
    }

    struct MockAgent {
        mock_role: AgentRole,
    }

    #[async_trait]
    impl AgentExecutor for MockAgent {
        fn role(&self) -> AgentRole { self.mock_role.clone() }
        fn capabilities(&self) -> Vec<TaskType> { vec![TaskType::GeneratePRD] }
        async fn execute(&self, task: Task) -> Result<Artifact> {
            Ok(Artifact {
                id: ArtifactId::new(),
                task_id: Some(task.id.clone()),
                artifact_type: ArtifactType::PRD,
                content: b"# Mock PRD".to_vec(),
                metadata: ArtifactMetadata {
                    created_by: AgentId::new(),
                    timestamp: chrono::Utc::now(),
                    version: "1.0".into(),
                    format: "markdown".into(),
                    size: 10,
                },
                verification_status: VerificationStatus::Pending,
            })
        }
    }

    #[tokio::test]
    async fn test_engine_dispatches_to_correct_agent() {
        let mut dag = WorkflowDag::new();
        let task = Task {
            id: TaskId::new(),
            agent_id: None,
            task_type: TaskType::GeneratePRD,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            payload: serde_json::json!({"description": "test"}),
            dependencies: vec![],
        };
        dag.add_task(task);

        let mut engine = WorkflowEngine::new(dag);
        engine.register_agent(AgentRole::Planner, Box::new(MockAgent { mock_role: AgentRole::Planner }));

        let artifacts = engine.tick().await.unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].artifact_type, ArtifactType::PRD);
    }

    #[tokio::test]
    async fn test_engine_run_to_completion() {
        let mut dag = WorkflowDag::new();
        let t1 = Task {
            id: TaskId::new(),
            agent_id: None,
            task_type: TaskType::GeneratePRD,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            payload: serde_json::json!({}),
            dependencies: vec![],
        };
        dag.add_task(t1);

        let mut engine = WorkflowEngine::new(dag);
        engine.register_agent(AgentRole::Planner, Box::new(MockAgent { mock_role: AgentRole::Planner }));

        let artifacts = engine.run_to_completion().await.unwrap();
        assert_eq!(artifacts.len(), 1);
        assert!(engine.dag().is_complete());
    }
}
