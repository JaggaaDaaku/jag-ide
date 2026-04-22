use jag_core::types::*;
use jag_core::errors::{JagError, Result};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| JagError::Database(e.into()))?;
        Ok(())
    }

    // Workspaces
    pub async fn create_workspace(&self, ws: &Workspace) -> Result<()> {
        let id_str = ws.id.to_string();
        let settings = serde_json::to_string(&serde_json::json!({})).unwrap_or_else(|_| "{}".to_string());
        
        sqlx::query(
            "INSERT INTO workspaces (id, name, root_path, created_at, modified_at, settings_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&ws.name)
        .bind(&ws.root_path)
        .bind(ws.created_at.to_rfc3339())
        .bind(ws.modified_at.to_rfc3339())
        .bind(settings)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_workspace(&self, id: &WorkspaceId) -> Result<Option<Workspace>> {
        let id_str = id.to_string();
        let row = sqlx::query("SELECT * FROM workspaces WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let id_text: String = row.try_get("id")?;
            let created_at_txt: String = row.try_get("created_at")?;
            let modified_at_txt: String = row.try_get("modified_at")?;

            let parsed_id = uuid::Uuid::parse_str(&id_text).map_err(|_| JagError::Internal("Invalid UUID".to_string()))?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let modified_at = chrono::DateTime::parse_from_rfc3339(&modified_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(Some(Workspace {
                id: WorkspaceId(parsed_id),
                name: row.try_get("name")?,
                root_path: row.try_get("root_path")?,
                created_at,
                modified_at,
                agents: vec![],
                artifacts: vec![],
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let rows = sqlx::query("SELECT * FROM workspaces")
            .fetch_all(&self.pool)
            .await?;

        let mut workspaces = Vec::new();
        for row in rows {
            let id_text: String = row.try_get("id")?;
            let created_at_txt: String = row.try_get("created_at")?;
            let modified_at_txt: String = row.try_get("modified_at")?;

            let parsed_id = uuid::Uuid::parse_str(&id_text).map_err(|_| JagError::Internal("Invalid UUID".to_string()))?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let modified_at = chrono::DateTime::parse_from_rfc3339(&modified_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            workspaces.push(Workspace {
                id: WorkspaceId(parsed_id),
                name: row.try_get("name")?,
                root_path: row.try_get("root_path")?,
                created_at,
                modified_at,
                agents: vec![],
                artifacts: vec![],
            });
        }
        Ok(workspaces)
    }

    pub async fn add_workspace_member(&self, workspace_id: &WorkspaceId, user_id: &UserId, role: WorkspaceRole, invited_by: Option<&UserId>) -> Result<()> {
        let role_str = match role {
            WorkspaceRole::Admin => "Admin",
            WorkspaceRole::Developer => "Developer",
            WorkspaceRole::Viewer => "Viewer",
        };

        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role, joined_at, invited_by) VALUES (?, ?, ?, datetime('now'), ?)"
        )
        .bind(workspace_id.to_string())
        .bind(user_id.to_string())
        .bind(role_str)
        .bind(invited_by.map(|u| u.to_string()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_workspace_members(&self, workspace_id: &WorkspaceId) -> Result<Vec<WorkspaceMember>> {
        let rows = sqlx::query("SELECT workspace_id, user_id, role, joined_at, invited_by FROM workspace_members WHERE workspace_id = ?")
            .bind(workspace_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut members = Vec::new();
        for row in rows {
            let role_text: String = row.try_get("role")?;
            let role = match role_text.as_str() {
                "Admin" => WorkspaceRole::Admin,
                "Developer" => WorkspaceRole::Developer,
                "Viewer" => WorkspaceRole::Viewer,
                _ => WorkspaceRole::Viewer,
            };

            let joined_at_txt: String = row.try_get("joined_at")?;
            let joined_at = chrono::DateTime::parse_from_rfc3339(&joined_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            members.push(WorkspaceMember {
                workspace_id: WorkspaceId(uuid::Uuid::parse_str(&row.try_get::<String, _>("workspace_id")?).unwrap_or_default()),
                user_id: UserId(uuid::Uuid::parse_str(&row.try_get::<String, _>("user_id")?).unwrap_or_default()),
                role,
                joined_at,
                invited_by: row.get::<Option<String>, _>("invited_by").map(|id| UserId(uuid::Uuid::parse_str(&id).unwrap_or_default())),
            });
        }
        Ok(members)
    }

    pub async fn remove_workspace_member(&self, workspace_id: &WorkspaceId, user_id: &UserId) -> Result<()> {
        sqlx::query("DELETE FROM workspace_members WHERE workspace_id = ? AND user_id = ?")
            .bind(workspace_id.to_string())
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Workflows
    pub async fn create_workflow(&self, id: &TaskId, workspace_id: &WorkspaceId, name: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO workflows (id, workspace_id, name, status, created_at, updated_at) VALUES (?, ?, ?, 'Active', datetime('now'), datetime('now'))"
        )
        .bind(id.to_string())
        .bind(workspace_id.to_string())
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_workflow_tasks(&self, workflow_id: &TaskId) -> Result<Vec<Task>> {
        let rows = sqlx::query("SELECT * FROM tasks WHERE workflow_id = ?")
            .bind(workflow_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut tasks = Vec::new();
        for row in rows {
            // Reusing existing task parsing logic (simplified)
            let id_text: String = row.try_get("id")?;
            let status_str: String = row.try_get("status")?;
            let priority_str: String = row.try_get("priority")?;
            let payload_str: String = row.try_get("payload_json")?;

            let task_type_str: String = row.try_get("task_type")?;

            tasks.push(Task {
                id: TaskId(uuid::Uuid::parse_str(&id_text).unwrap_or_default()),
                agent_id: None,
                task_type: serde_json::from_str(&format!("\"{}\"", task_type_str)).unwrap_or(TaskType::GeneratePRD),
                status: serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or(TaskStatus::Pending),
                priority: serde_json::from_str(&format!("\"{}\"", priority_str)).unwrap_or(Priority::Normal),
                payload: serde_json::from_str(&payload_str).unwrap_or_default(),
                dependencies: vec![],
            });
        }
        Ok(tasks)
    }

    // Agents
    pub async fn create_agent(&self, id: &AgentId, workspace_id: &WorkspaceId, role: &AgentRole, model_id: Option<&ModelId>, tier: &SecurityTier) -> Result<()> {
        let role_str = match role {
            AgentRole::Planner => "Planner",
            AgentRole::Backend => "Backend",
            AgentRole::Frontend => "Frontend",
            AgentRole::Integration => "Integration",
        };
        
        let tier_str = match tier {
            SecurityTier::Off => "Off",
            SecurityTier::Auto => "Auto",
            SecurityTier::Turbo => "Turbo",
        };

        sqlx::query(
            "INSERT INTO agents (id, workspace_id, role, status, model_id, security_tier, created_at, modified_at) VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
        )
        .bind(id.to_string())
        .bind(workspace_id.to_string())
        .bind(role_str)
        .bind("Idle")
        .bind(model_id.map(|m| m.to_string()))
        .bind(tier_str)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_agent(&self, id: &AgentId) -> Result<Option<AgentState>> {
        let row = sqlx::query("SELECT status, modified_at FROM agents WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let status_text: String = row.try_get("status")?;
            let modified_text: String = row.try_get("modified_at")?;

            let status = match status_text.as_str() {
                "Idle" => AgentStatus::Idle,
                "Working" => AgentStatus::Working,
                "Completed" => AgentStatus::Completed,
                "Error" => AgentStatus::Error,
                _ => AgentStatus::Idle,
            };
            
            let last_heartbeat = chrono::DateTime::parse_from_rfc3339(&modified_text)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(Some(AgentState {
                status,
                current_task: None, // In real scenario, retrieve from active task query
                progress: 0,
                last_heartbeat,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_agent_status(&self, id: &AgentId, status: &AgentStatus) -> Result<()> {
        let status_str = match status {
            AgentStatus::Idle => "Idle",
            AgentStatus::Working => "Working",
            AgentStatus::Completed => "Completed",
            AgentStatus::Error => "Error",
        };

        sqlx::query("UPDATE agents SET status = ?, modified_at = datetime('now') WHERE id = ?")
            .bind(status_str)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Tasks
    pub async fn create_task(&self, task: &Task) -> Result<()> {
        let type_str = format!("{:?}", task.task_type);
        let status_str = match task.status {
            TaskStatus::Pending => "Pending",
            TaskStatus::Running => "Running",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Cancelled => "Cancelled",
        };
        
        let priority_str = match task.priority {
            Priority::Low => "Low",
            Priority::Normal => "Normal",
            Priority::High => "High",
            Priority::Critical => "Critical",
        };

        let payload_json = serde_json::to_string(&task.payload).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            "INSERT INTO tasks (id, agent_id, workflow_id, task_type, status, priority, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(task.id.to_string())
        .bind(task.agent_id.as_ref().map(|a| a.to_string()))
        .bind(None::<String>) // workflow_id initially null unless specified
        .bind(type_str)
        .bind(status_str)
        .bind(priority_str)
        .bind(payload_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_task(&self, id: &TaskId) -> Result<Option<Task>> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let payload_str: String = row.try_get("payload_json")?;
            let payload = serde_json::from_str(&payload_str).unwrap_or(serde_json::json!({}));
            
            let status_str: String = row.try_get("status")?;
            let status = match status_str.as_str() {
                "Pending" => TaskStatus::Pending,
                "Running" => TaskStatus::Running,
                "Completed" => TaskStatus::Completed,
                "Failed" => TaskStatus::Failed,
                "Cancelled" => TaskStatus::Cancelled,
                _ => TaskStatus::Pending,
            };
            
            let priority_str: String = row.try_get("priority")?;
            let priority = match priority_str.as_str() {
                "Low" => Priority::Low,
                "Normal" => Priority::Normal,
                "High" => Priority::High,
                "Critical" => Priority::Critical,
                _ => Priority::Normal,
            };

            let id_text: String = row.try_get("id")?;
            let parsed_id = uuid::Uuid::parse_str(&id_text)
                .map_err(|_| JagError::Internal("Invalid UUID".into()))?;

            let task_type_str: String = row.try_get("task_type")?;
            let task_type = serde_json::from_str(&format!("\"{}\"", task_type_str)).unwrap_or(TaskType::GeneratePRD);

            Ok(Some(Task {
                id: TaskId(parsed_id),
                agent_id: None,
                task_type,
                status,
                priority,
                payload,
                dependencies: vec![],
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_task_status(&self, id: &TaskId, status: &TaskStatus) -> Result<()> {
        let status_str = match status {
            TaskStatus::Pending => "Pending",
            TaskStatus::Running => "Running",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Cancelled => "Cancelled",
        };

        sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Artifacts
    pub async fn create_artifact(&self, art: &Artifact) -> Result<()> {
        let type_str = format!("{:?}", art.artifact_type);
        
        sqlx::query(
            "INSERT INTO artifacts (id, workspace_id, agent_id, task_id, artifact_type, file_size, format, verification_status, metadata_json, created_at, modified_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
        )
        .bind(art.id.to_string())
        .bind(art.metadata.created_by.to_string())
        .bind(art.metadata.created_by.to_string())
        .bind(art.task_id.as_ref().map(|t| t.to_string()))
        .bind(type_str)
        .bind(art.metadata.size as i64)
        .bind(&art.metadata.format)
        .bind(format!("{:?}", art.verification_status))
        .bind("{}")
        .execute(&self.pool)
        .await?;
 
        Ok(())
    }

    pub async fn update_artifact_status(&self, id: &ArtifactId, status: &VerificationStatus) -> Result<()> {
        let status_str = match status {
            VerificationStatus::Pending => "Pending",
            VerificationStatus::Approved => "Approved",
            VerificationStatus::Rejected => "Rejected",
        };

        sqlx::query("UPDATE artifacts SET verification_status = ?, modified_at = datetime('now') WHERE id = ?")
            .bind(status_str)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_artifact_content(&self, id: &ArtifactId, _new_content: &str, current_version: i32) -> Result<bool> {
        // In real impl, we'd update file contents. Here we just bump the version in DB.
        let result = sqlx::query("UPDATE artifacts SET version = version + 1, modified_at = datetime('now') WHERE id = ? AND version = ?")
            .bind(id.to_string())
            .bind(current_version)
            .execute(&self.pool)
            .await?;
            
        Ok(result.rows_affected() > 0)
    }

    // Approvals
    pub async fn create_approval(
        &self,
        id: &ArtifactId,
        workflow_id: &TaskId,
        task_id: &TaskId,
        decision: &ApprovalDecision,
    ) -> Result<()> {
        let (decision_str, confidence, reasoning, fixes) = match decision {
            ApprovalDecision::AutoApprove { confidence, reasoning } => {
                ("AutoApprove", *confidence, reasoning.clone(), "[]".to_string())
            }
            ApprovalDecision::ApproveWithNotice { confidence, reasoning, .. } => {
                ("ApproveWithNotice", *confidence, reasoning.clone(), "[]".to_string())
            }
            ApprovalDecision::RequiresApproval { confidence, reasoning, suggested_fixes } => {
                ("RequiresApproval", *confidence, reasoning.clone(), serde_json::to_string(suggested_fixes).unwrap_or_default())
            }
        };

        sqlx::query(
            "INSERT INTO workflow_approvals (id, workflow_id, task_id, decision, confidence, reasoning, suggested_fixes_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(id.to_string())
        .bind(workflow_id.to_string())
        .bind(task_id.to_string())
        .bind(decision_str)
        .bind(confidence)
        .bind(reasoning)
        .bind(fixes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_artifacts_by_agent(&self, _agent_id: &AgentId) -> Result<Vec<Artifact>> {
        // Simple return to satisfy signatures
        Ok(vec![])
    }

    // Audit
    pub async fn log_action(&self, workspace_id: &WorkspaceId, agent_id: Option<&AgentId>, action: &str, details: serde_json::Value) -> Result<()> {
        let details_str = serde_json::to_string(&details).unwrap_or_else(|_| "{}".to_string());
        sqlx::query(
            "INSERT INTO audit_log (workspace_id, agent_id, action_type, details_json, created_at) VALUES (?, ?, ?, ?, datetime('now'))"
        )
        .bind(workspace_id.to_string())
        .bind(agent_id.map(|a| a.to_string()))
        .bind(action)
        .bind(details_str)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn log_signed_action(
        &self,
        workspace_id: Option<WorkspaceId>,
        user_id: Option<UserId>,
        agent_id: Option<AgentId>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
        details: serde_json::Value,
        signature: &str,
    ) -> Result<()> {
        let details_str = serde_json::to_string(&details).unwrap_or_else(|_| "{}".to_string());
        sqlx::query(
            "INSERT INTO audit_log (workspace_id, user_id, agent_id, action_type, resource_type, resource_id, details_json, signature, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(workspace_id.map(|w| w.to_string()))
        .bind(user_id.map(|u| u.to_string()))
        .bind(agent_id.map(|a| a.to_string()))
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(details_str)
        .bind(signature)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Model Usage
    pub async fn log_model_usage(&self, record: ModelUsageRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO model_usage (model_name, prompt_tokens, completion_tokens, total_tokens, cost_estimated, timestamp, is_local)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&record.model_name)
        .bind(record.prompt_tokens as i64)
        .bind(record.completion_tokens as i64)
        .bind(record.total_tokens as i64)
        .bind(record.cost_estimated)
        .bind(record.timestamp.to_rfc3339())
        .bind(if record.is_local { 1 } else { 0 })
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_usage_summary(&self) -> Result<serde_json::Value> {
        let row = sqlx::query("SELECT COUNT(*) as calls, SUM(total_tokens) as tokens, SUM(cost_estimated) as cost FROM model_usage")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(serde_json::json!({
            "total_calls": row.get::<i64, _>("calls"),
            "total_tokens": row.get::<Option<i64>, _>("tokens").unwrap_or(0),
            "total_cost": row.get::<Option<f64>, _>("cost").unwrap_or(0.0),
        }))
    }

    pub async fn get_recent_artifacts(&self, limit: usize) -> Result<Vec<ArtifactRecord>> {
        let rows = sqlx::query("SELECT * FROM artifacts ORDER BY created_at DESC LIMIT ?")
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut artifacts = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let task_id = row.get::<Option<String>, _>("task_id")
                .and_then(|s| if s.is_empty() || s == "null" { None } else { Some(TaskId(uuid::Uuid::parse_str(&s).unwrap_or_default())) });
            let created_at_str: String = row.get("created_at");

            artifacts.push(ArtifactRecord {
                id: ArtifactId(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                task_id,
                artifact_type: row.try_get("artifact_type").unwrap_or_else(|_| "Unknown".into()),
                file_path: row.get::<Option<String>, _>("content_path").unwrap_or_default(),
                metadata: serde_json::from_str(&row.try_get::<String, _>("metadata_json").unwrap_or_default()).unwrap_or_default(),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_else(|_| chrono::Utc::now().into()).with_timezone(&chrono::Utc),
                version: row.try_get("version").unwrap_or(1),
            });
        }
        Ok(artifacts)
    }

    // Settings
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT value_json FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.and_then(|r| r.try_get("value_json").ok()))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO settings (key, value_json, updated_at) VALUES (?, ?, datetime('now')) ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Authentication & Users
    pub async fn get_user_by_id(&self, id: &UserId) -> Result<Option<User>> {
        let id_str = id.to_string();
        let row = sqlx::query("SELECT id, email, role, created_at FROM users WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let id_text: String = row.try_get("id")?;
            let role_text: String = row.try_get("role")?;
            let created_at_txt: String = row.try_get("created_at")?;

            let id = UserId(uuid::Uuid::parse_str(&id_text).map_err(|_| JagError::Internal("Invalid UUID".to_string()))?);
            let role = match role_text.as_str() {
                "Admin" => UserRole::Admin,
                "Developer" => UserRole::Developer,
                "Viewer" => UserRole::Viewer,
                _ => UserRole::Developer,
            };
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(Some(User { id, email: row.try_get("email")?, role, created_at }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT id, email, role, created_at FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let id_text: String = row.try_get("id")?;
            let role_text: String = row.try_get("role")?;
            let created_at_txt: String = row.try_get("created_at")?;

            let id = UserId(uuid::Uuid::parse_str(&id_text).map_err(|_| JagError::Internal("Invalid UUID".to_string()))?);
            let role = match role_text.as_str() {
                "Admin" => UserRole::Admin,
                "Developer" => UserRole::Developer,
                "Viewer" => UserRole::Viewer,
                _ => UserRole::Developer,
            };
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(Some(User { id, email: row.try_get("email")?, role, created_at }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_user(&self, email: &str, role: &UserRole) -> Result<User> {
        let id = UserId::new();
        let role_str = match role {
            UserRole::Admin => "Admin",
            UserRole::Developer => "Developer",
            UserRole::Viewer => "Viewer",
        };

        sqlx::query("INSERT INTO users (id, email, role, created_at) VALUES (?, ?, ?, datetime('now'))")
            .bind(id.to_string())
            .bind(email)
            .bind(role_str)
            .execute(&self.pool)
            .await?;

        Ok(User {
            id,
            email: email.to_string(),
            role: role.clone(),
            created_at: chrono::Utc::now(),
        })
    }

    // Sessions
    pub async fn create_session(
        &self,
        user_id: &UserId,
        token_hash: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
        ip: Option<&str>,
    ) -> Result<UserSession> {
        let id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, refresh_token_hash, expires_at, created_at, ip_address) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(token_hash)
        .bind(expires_at.to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(ip)
        .execute(&self.pool)
        .await?;

        Ok(UserSession {
            id,
            user_id: user_id.clone(),
            refresh_token_hash: token_hash.to_string(),
            expires_at,
            created_at: chrono::Utc::now(),
            ip_address: ip.map(|s| s.to_string()),
        })
    }

    pub async fn validate_session(&self, token_hash: &str) -> Result<Option<UserSession>> {
        let row = sqlx::query("SELECT * FROM user_sessions WHERE refresh_token_hash = ? AND expires_at > datetime('now')")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let id_str: String = row.try_get("id")?;
            let user_id_str: String = row.try_get("user_id")?;
            let expires_at_txt: String = row.try_get("expires_at")?;
            let created_at_txt: String = row.try_get("created_at")?;

            Ok(Some(UserSession {
                id: uuid::Uuid::parse_str(&id_str).unwrap_or_default(),
                user_id: UserId(uuid::Uuid::parse_str(&user_id_str).unwrap_or_default()),
                refresh_token_hash: row.try_get("refresh_token_hash")?,
                expires_at: chrono::DateTime::parse_from_rfc3339(&expires_at_txt).unwrap().with_timezone(&chrono::Utc),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_txt).unwrap().with_timezone(&chrono::Utc),
                ip_address: row.try_get("ip_address")?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_session(&self, token_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_sessions WHERE refresh_token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_all_user_sessions(&self, user_id: &UserId) -> Result<()> {
        sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Audit Logging
    pub async fn get_audit_logs_paginated(&self, offset: u32, limit: u32) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query("SELECT * FROM audit_log ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut entries = Vec::new();
        for row in rows {
            let created_at_txt: String = row.try_get("created_at")?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            let details_json: String = row.try_get("details_json")?;

            entries.push(AuditEntry {
                id: row.try_get("id")?,
                timestamp: created_at,
                workspace_id: row.try_get::<Option<String>, _>("workspace_id")?.map(|s| WorkspaceId(uuid::Uuid::parse_str(&s).unwrap_or_default())),
                user_id: row.try_get::<Option<String>, _>("user_id")?.map(|s| UserId(uuid::Uuid::parse_str(&s).unwrap_or_default())),
                agent_id: row.try_get::<Option<String>, _>("agent_id")?.map(|s| AgentId(uuid::Uuid::parse_str(&s).unwrap_or_default())),
                action: row.try_get("action_type")?,
                resource_type: row.try_get("resource_type")?,
                resource_id: row.try_get("resource_id")?,
                details: serde_json::from_str(&details_json).unwrap_or_default(),
                result: AuditResult::Success, // Default for historical logs
                ip_address: row.try_get("ip_address")?,
                signature: row.try_get("signature")?,
            });
        }
        Ok(entries)
    }

    pub async fn get_all_audit_logs(&self) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query("SELECT * FROM audit_log ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut entries = Vec::new();
        for row in rows {
            let created_at_txt: String = row.try_get("created_at")?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_txt)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            let details_json: String = row.try_get("details_json")?;

            entries.push(AuditEntry {
                id: row.try_get("id")?,
                timestamp: created_at,
                workspace_id: row.try_get::<Option<String>, _>("workspace_id")?.map(|s| WorkspaceId(uuid::Uuid::parse_str(&s).unwrap_or_default())),
                user_id: row.try_get::<Option<String>, _>("user_id")?.map(|s| UserId(uuid::Uuid::parse_str(&s).unwrap_or_default())),
                agent_id: row.try_get::<Option<String>, _>("agent_id")?.map(|s| AgentId(uuid::Uuid::parse_str(&s).unwrap_or_default())),
                action: row.try_get("action_type")?,
                resource_type: row.try_get("resource_type")?,
                resource_id: row.try_get("resource_id")?,
                details: serde_json::from_str(&details_json).unwrap_or_default(),
                result: AuditResult::Success,
                ip_address: row.try_get("ip_address")?,
                signature: row.try_get("signature")?,
            });
        }
        Ok(entries)
    }

    pub async fn get_daily_usage_stats(&self, days: u32) -> Result<serde_json::Value> {
        let rows = sqlx::query(
            "SELECT 
                date(timestamp) as usage_date,
                COUNT(*) as calls,
                SUM(prompt_tokens) as prompt_tokens,
                SUM(completion_tokens) as completion_tokens,
                SUM(total_tokens) as total_tokens,
                SUM(cost_estimated) as cost
             FROM model_usage 
             WHERE timestamp >= date('now', ?)
             GROUP BY usage_date
             ORDER BY usage_date ASC"
        )
        .bind(format!("-{} days", days))
        .fetch_all(&self.pool)
        .await?;

        let stats: Vec<serde_json::Value> = rows.into_iter().map(|row| {
            serde_json::json!({
                "date": row.get::<String, _>("usage_date"),
                "calls": row.get::<i64, _>("calls"),
                "prompt_tokens": row.get::<Option<i64>, _>("prompt_tokens").unwrap_or(0),
                "completion_tokens": row.get::<Option<i64>, _>("completion_tokens").unwrap_or(0),
                "total_tokens": row.get::<Option<i64>, _>("total_tokens").unwrap_or(0),
                "cost": row.get::<Option<f64>, _>("cost").unwrap_or(0.0),
            })
        }).collect();

        Ok(serde_json::json!(stats))
    }

    // Benchmarks
    pub async fn create_benchmark(&self, bench: &BenchmarkResult) -> Result<()> {
        sqlx::query(
            "INSERT INTO model_benchmarks (id, model_name, task_type, latency_ms, tokens_per_second, total_tokens, cost_usd, timestamp, success)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(bench.id.to_string())
        .bind(&bench.model_name)
        .bind(&bench.task_type)
        .bind(bench.latency_ms as i64)
        .bind(bench.tokens_per_second)
        .bind(bench.total_tokens as i64)
        .bind(bench.cost_usd)
        .bind(bench.timestamp.to_rfc3339())
        .bind(bench.success)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_model_benchmarks(&self, model_name: &str) -> Result<Vec<BenchmarkResult>> {
        let rows = sqlx::query("SELECT * FROM model_benchmarks WHERE model_name = ? ORDER BY timestamp DESC LIMIT 50")
            .bind(model_name)
            .fetch_all(&self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id")?;
            let timestamp_str: String = row.try_get("timestamp")?;
            
            results.push(BenchmarkResult {
                id: TaskId(uuid::Uuid::parse_str(&id_str).unwrap_or_default()),
                model_name: row.try_get("model_name")?,
                task_type: row.try_get("task_type")?,
                latency_ms: row.get::<i64, _>("latency_ms") as u64,
                tokens_per_second: row.try_get("tokens_per_second")?,
                total_tokens: row.get::<i64, _>("total_tokens") as u32,
                cost_usd: row.try_get("cost_usd")?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str).unwrap_or_else(|_| chrono::Utc::now().into()).with_timezone(&chrono::Utc),
                success: row.try_get("success")?,
            });
        }
        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    async fn setup_db() -> Database {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.run_migrations().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_workspace_crud() {
        let db = setup_db().await;
        
        let ws = Workspace {
            id: WorkspaceId::new(),
            name: "Test WS".to_string(),
            root_path: "/tmp/ws".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            agents: vec![],
            artifacts: vec![],
        };

        db.create_workspace(&ws).await.unwrap();
        let fetched = db.get_workspace(&ws.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Test WS");
        
        let list = db.list_workspaces().await.unwrap();
        assert_eq!(list.len(), 1);
    }
    
    #[tokio::test]
    async fn test_settings_crud() {
        let db = setup_db().await;
        db.set_setting("test_key", "\"test_value\"").await.unwrap();
        let val = db.get_setting("test_key").await.unwrap().unwrap();
        assert_eq!(val, "\"test_value\"");
        
        // Default insert from migration
        let def = db.get_setting("security.default_tier").await.unwrap().unwrap();
        assert_eq!(def, "\"Auto\"");
    }
}
