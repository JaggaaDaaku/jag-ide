use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use jag_core::errors::{JagError, Result};
use crate::security::SecurityPolicy;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Output from a sandboxed command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u64,
}

/// Executes shell commands within sandbox constraints.
///
/// All commands are validated against the security policy before execution.
/// Enforces timeouts and captures stdout/stderr.
pub struct CommandExecutor {
    policy: SecurityPolicy,
    default_timeout: Duration,
}

impl CommandExecutor {
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy,
            default_timeout: Duration::from_secs(120),
        }
    }

    pub fn with_timeout(policy: SecurityPolicy, timeout: Duration) -> Self {
        Self {
            policy,
            default_timeout: timeout,
        }
    }

    /// Execute a command within the sandbox.
    ///
    /// The command string is validated against the security policy's denylist.
    /// The working directory must be within the workspace.
    pub async fn execute(
        &self,
        command: &str,
        cwd: &Path,
        timeout: Option<Duration>,
    ) -> Result<CommandOutput> {
        // Validate command against denylist
        self.policy.validate_command(command)?;

        // Validate working directory
        self.policy.validate_path(cwd)?;

        let timeout = timeout.unwrap_or(self.default_timeout);
        // Build the command
        let mut child = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", command])
                .current_dir(cwd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| JagError::Internal(format!("Failed to spawn command: {}", e)))?
        } else {
            Command::new("sh")
                .args(["-c", command])
                .current_dir(cwd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| JagError::Internal(format!("Failed to spawn command: {}", e)))?
        };

        let start = std::time::Instant::now();

        // Capture output streams
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();

        // Wait with timeout
        match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => {
                let duration = start.elapsed();
                
                // Read buffers
                use tokio::io::AsyncReadExt;
                let mut out_buf = Vec::new();
                let mut err_buf = Vec::new();
                let _ = stdout.read_to_end(&mut out_buf).await;
                let _ = stderr.read_to_end(&mut err_buf).await;

                let result = CommandOutput {
                    stdout: String::from_utf8_lossy(&out_buf).to_string(),
                    stderr: String::from_utf8_lossy(&err_buf).to_string(),
                    exit_code: status.code(),
                    timed_out: false,
                    duration_ms: duration.as_millis() as u64,
                };

                if !status.success() {
                    warn!(
                        command = command,
                        exit_code = ?status.code(),
                        stderr = %result.stderr.chars().take(200).collect::<String>(),
                        "Command exited with non-zero status"
                    );
                }

                Ok(result)
            }
            Ok(Err(e)) => {
                Err(JagError::Internal(format!("Command I/O error: {}", e)))
            }
            Err(_) => {
                // Timeout — kill the process (child is still accessible because wait() took &mut)
                let _ = child.kill().await;
                warn!(command = command, timeout_secs = timeout.as_secs(), "Command timed out");
                Ok(CommandOutput {
                    stdout: String::new(),
                    stderr: "Command timed out".to_string(),
                    exit_code: None,
                    timed_out: true,
                    duration_ms: timeout.as_millis() as u64,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jag_core::types::SecurityTier;

    fn test_policy() -> SecurityPolicy {
        let cwd = std::env::current_dir().unwrap();
        SecurityPolicy::new(SecurityTier::Auto, cwd)
    }

    #[tokio::test]
    async fn test_simple_command() {
        let executor = CommandExecutor::new(test_policy());
        let cwd = std::env::current_dir().unwrap();

        let cmd = if cfg!(windows) { "echo hello" } else { "echo hello" };
        let result = executor.execute(cmd, &cwd, None).await.unwrap();

        assert!(!result.timed_out);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_denied_command() {
        let executor = CommandExecutor::new(test_policy());
        let cwd = std::env::current_dir().unwrap();

        let result = executor.execute("rm -rf /", &cwd, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout() {
        let executor = CommandExecutor::with_timeout(
            test_policy(),
            Duration::from_millis(100),
        );
        let cwd = std::env::current_dir().unwrap();

        let cmd = if cfg!(windows) { "ping -n 10 127.0.0.1" } else { "sleep 10" };
        let result = executor.execute(cmd, &cwd, Some(Duration::from_millis(100))).await.unwrap();

        assert!(result.timed_out);
    }
}
