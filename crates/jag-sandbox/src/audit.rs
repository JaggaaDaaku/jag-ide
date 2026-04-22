use jag_core::types::{AgentId, UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use jag_db::Database;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Represents a single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<UserId>,
    pub agent_id: Option<AgentId>,
    pub action: String,
    pub resource: String,
    pub details: String,
    pub result: AuditResult,
}

/// Outcome of an audited action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Denied(String),
    Failed(String),
}

/// Persistent audit logger with cryptographic signing.
pub struct AuditLogger {
    db: Arc<Database>,
    hmac_secret: String,
}

impl AuditLogger {
    pub fn new(db: Arc<Database>, hmac_secret: String) -> Self {
        Self { db, hmac_secret }
    }

    /// Log a signed action.
    #[allow(clippy::too_many_arguments)]
    pub async fn log_signed(
        &self,
        workspace_id: Option<WorkspaceId>,
        user_id: Option<UserId>,
        agent_id: Option<AgentId>,
        action: &str,
        resource: &str,
        details: &str,
        result: AuditResult,
    ) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            user_id: user_id.clone(),
            agent_id: agent_id.clone(),
            action: action.to_string(),
            resource: resource.to_string(),
            details: details.to_string(),
            result: result.clone(),
        };

        let signature = self.sign_entry(&entry);
        
        let details_json = match result {
            AuditResult::Success => serde_json::json!({ "details": details }),
            AuditResult::Denied(ref r) => serde_json::json!({ "reason": r }),
            AuditResult::Failed(ref e) => serde_json::json!({ "error": e }),
        };

        let _ = self.db.log_signed_action(
            workspace_id,
            user_id,
            agent_id,
            action,
            Some("resource"), // Simplified mapping
            Some(resource),
            details_json,
            &signature,
        ).await;
    }

    /// Sign an entry using HMAC-SHA256.
    fn sign_entry(&self, entry: &AuditEntry) -> String {
        let payload = format!(
            "{:?}|{:?}|{:?}|{}|{}|{}|{:?}",
            entry.timestamp, entry.user_id, entry.agent_id, entry.action, entry.resource, entry.details, entry.result
        );
        
        let mut mac = HmacSha256::new_from_slice(self.hmac_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();

        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }

    /// Verify a signature against an entry.
    pub fn verify_signature(&self, entry: &AuditEntry, signature: &str) -> bool {
        let expected = self.sign_entry(entry);
        expected == signature
    }
}

#[cfg(test)]
mod tests {
    // Tests are currently disabled due to Database dependency.
    // In a production environment, we would use an in-memory SQLite database for testing.
}
