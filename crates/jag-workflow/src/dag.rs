use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::is_cyclic_directed;
use jag_core::types::{Task, TaskId, TaskStatus, DependencyType};
use jag_core::errors::{JagError, Result};

/// Directed Acyclic Graph representing a workflow of tasks.
///
/// Each node is a `Task`, each edge represents a dependency between tasks.
/// Tasks can only execute once all their incoming dependencies are resolved.
pub struct WorkflowDag {
    graph: DiGraph<Task, DependencyType>,
    /// Maps TaskId → NodeIndex for O(1) lookups
    index_map: HashMap<TaskId, NodeIndex>,
}

impl WorkflowDag {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index_map: HashMap::new(),
        }
    }

    /// Add a task to the DAG. Returns the internal node index.
    pub fn add_task(&mut self, task: Task) -> NodeIndex {
        let task_id = task.id.clone();
        let idx = self.graph.add_node(task);
        self.index_map.insert(task_id, idx);
        idx
    }

    /// Add a dependency edge: `dependent` depends on `dependency`.
    /// `dependent` cannot start until `dependency` is completed.
    pub fn add_dependency(
        &mut self,
        dependency: &TaskId,
        dependent: &TaskId,
        dep_type: DependencyType,
    ) -> Result<()> {
        let from = *self.index_map.get(dependency)
            .ok_or_else(|| JagError::TaskNotFound(dependency.clone()))?;
        let to = *self.index_map.get(dependent)
            .ok_or_else(|| JagError::TaskNotFound(dependent.clone()))?;

        self.graph.add_edge(from, to, dep_type);
        Ok(())
    }

    /// Validate the DAG has no cycles.
    pub fn validate(&self) -> Result<()> {
        if is_cyclic_directed(&self.graph) {
            return Err(JagError::CircularDependency);
        }
        Ok(())
    }

    /// Get all tasks whose dependencies are fully resolved (completed).
    /// These tasks are ready to execute.
    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        let mut ready = Vec::new();

        for node_idx in self.graph.node_indices() {
            let task = &self.graph[node_idx];

            // Skip tasks that are already running, completed, failed, or cancelled
            if task.status != TaskStatus::Pending {
                continue;
            }

            // Check if all incoming edges (dependencies) have completed tasks
            let all_deps_done = self.graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .all(|dep_idx| {
                    self.graph[dep_idx].status == TaskStatus::Completed
                });

            if all_deps_done {
                ready.push(task);
            }
        }

        ready
    }

    /// Update a task's status in the DAG.
    pub fn update_task_status(&mut self, task_id: &TaskId, status: TaskStatus) -> Result<()> {
        let idx = *self.index_map.get(task_id)
            .ok_or_else(|| JagError::TaskNotFound(task_id.clone()))?;
        self.graph[idx].status = status;
        Ok(())
    }

    /// Get a task by its ID.
    pub fn get_task(&self, task_id: &TaskId) -> Option<&Task> {
        self.index_map.get(task_id).map(|idx| &self.graph[*idx])
    }

    /// Get all tasks in the DAG.
    pub fn all_tasks(&self) -> Vec<&Task> {
        self.graph.node_weights().collect()
    }

    /// Check if the entire workflow is complete.
    pub fn is_complete(&self) -> bool {
        self.graph.node_weights().all(|t| {
            t.status == TaskStatus::Completed || t.status == TaskStatus::Cancelled
        })
    }

    /// Check if any task has failed.
    pub fn has_failures(&self) -> bool {
        self.graph.node_weights().any(|t| t.status == TaskStatus::Failed)
    }

    /// Get a count of tasks by status.
    pub fn status_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for task in self.graph.node_weights() {
            let key = format!("{:?}", task.status);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for WorkflowDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jag_core::types::*;

    fn make_task(task_type: TaskType) -> Task {
        Task {
            id: TaskId::new(),
            agent_id: None,
            task_type,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            payload: serde_json::json!({}),
            dependencies: vec![],
        }
    }

    #[test]
    fn test_add_tasks_and_dependencies() {
        let mut dag = WorkflowDag::new();

        let t1 = make_task(TaskType::GeneratePRD);
        let t2 = make_task(TaskType::DesignArchitecture);
        let t1_id = t1.id.clone();
        let t2_id = t2.id.clone();

        dag.add_task(t1);
        dag.add_task(t2);
        dag.add_dependency(&t1_id, &t2_id, DependencyType::Hard).unwrap();

        assert!(dag.validate().is_ok());
    }

    #[test]
    fn test_circular_dependency_detected() {
        let mut dag = WorkflowDag::new();

        let t1 = make_task(TaskType::GeneratePRD);
        let t2 = make_task(TaskType::DesignArchitecture);
        let t1_id = t1.id.clone();
        let t2_id = t2.id.clone();

        dag.add_task(t1);
        dag.add_task(t2);
        dag.add_dependency(&t1_id, &t2_id, DependencyType::Hard).unwrap();
        dag.add_dependency(&t2_id, &t1_id, DependencyType::Hard).unwrap();

        assert!(dag.validate().is_err());
    }

    #[test]
    fn test_get_ready_tasks() {
        let mut dag = WorkflowDag::new();

        let t1 = make_task(TaskType::GeneratePRD);
        let t2 = make_task(TaskType::DesignArchitecture);
        let t3 = make_task(TaskType::SpecifyAPIs);
        let t1_id = t1.id.clone();
        let t2_id = t2.id.clone();
        let t3_id = t3.id.clone();

        dag.add_task(t1);
        dag.add_task(t2);
        dag.add_task(t3);

        // t2 depends on t1, t3 depends on t1
        dag.add_dependency(&t1_id, &t2_id, DependencyType::Hard).unwrap();
        dag.add_dependency(&t1_id, &t3_id, DependencyType::Hard).unwrap();

        // Initially, only t1 is ready (no deps)
        let ready = dag.get_ready_tasks();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, t1_id);

        // Complete t1 → t2, t3 become ready
        dag.update_task_status(&t1_id, TaskStatus::Completed).unwrap();
        let ready = dag.get_ready_tasks();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_workflow_completion() {
        let mut dag = WorkflowDag::new();
        let t1 = make_task(TaskType::GeneratePRD);
        let t1_id = t1.id.clone();
        dag.add_task(t1);

        assert!(!dag.is_complete());
        dag.update_task_status(&t1_id, TaskStatus::Completed).unwrap();
        assert!(dag.is_complete());
    }

    #[test]
    fn test_status_counts() {
        let mut dag = WorkflowDag::new();
        let t1 = make_task(TaskType::GeneratePRD);
        let t2 = make_task(TaskType::DesignArchitecture);
        let t1_id = t1.id.clone();

        dag.add_task(t1);
        dag.add_task(t2);

        dag.update_task_status(&t1_id, TaskStatus::Completed).unwrap();

        let counts = dag.status_counts();
        assert_eq!(*counts.get("Completed").unwrap_or(&0), 1);
        assert_eq!(*counts.get("Pending").unwrap_or(&0), 1);
    }
}
