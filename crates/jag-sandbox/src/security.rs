use std::path::{Path, PathBuf};
use jag_core::types::SecurityTier;
use jag_core::errors::{JagError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    ReadFile,
    SearchIndex,
    FormatLint,
    RunTests,
    InstallDeps,
    ModifyFile,
    GitOperation,
    NetworkAccess,
    BrowserAccess,
}

/// Returns whether an action is auto-approved for the given tier
pub fn is_auto_approved(tier: &SecurityTier, action: &ActionType) -> bool {
    match tier {
        SecurityTier::Off => false,
        SecurityTier::Auto => matches!(
            action,
            ActionType::ReadFile | ActionType::SearchIndex | ActionType::FormatLint | ActionType::RunTests
        ),
        SecurityTier::Turbo => true,
    }
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_seconds: u64,
    pub max_process_count: u32,
    pub max_file_size_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_seconds: 60,
            max_process_count: 10,
            max_file_size_mb: 100,
        }
    }
}

pub struct SecurityPolicy {
    tier: SecurityTier,
    workspace_root: PathBuf,
    pub allowed_paths: Vec<PathBuf>,
    pub denied_commands: Vec<String>,
    pub resource_limits: ResourceLimits,
}

impl SecurityPolicy {
    pub fn new(tier: SecurityTier, workspace_root: PathBuf) -> Self {
        Self {
            tier,
            workspace_root,
            allowed_paths: vec![],
            denied_commands: vec![
                "rm -rf".into(),
                "dd ".into(),
                "mkfs".into(),
                "shutdown".into(),
                "reboot".into(),
                "format ".into(),
                "chmod 777".into(),
                "chown ".into(),
                "kill -9".into(),
                "pkill".into(),
                "curl ".into(),
                "wget ".into(),
                "nc ".into(),
                "ncat ".into(),
                "netcat".into(),
                "/dev/tcp".into(),
                "/dev/udp".into(),
            ],
            resource_limits: ResourceLimits::default(),
        }
    }

    pub fn validate_action(&self, action: &ActionType) -> Result<bool> {
        Ok(is_auto_approved(&self.tier, action))
    }

    pub fn validate_path(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        
        // 1. Basic path traversal check (syntactic)
        if path_str.contains("..") {
            return Err(JagError::PathTraversal(path_str.into_owned()));
        }
 
        // 2. Resolve root
        let root = if self.workspace_root.is_absolute() {
            self.workspace_root.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(&self.workspace_root)
        };
        
        let canon_root = root.canonicalize().unwrap_or(root);
        
        // 3. Resolve target
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            canon_root.join(path)
        };
 
        // Resolve symlinks if path exists
        let canon_target = target.canonicalize().unwrap_or(target);
 
        // 4. Prefix check
        #[cfg(windows)]
        {
            let s_target = canon_target.to_string_lossy().to_lowercase();
            let s_root = canon_root.to_string_lossy().to_lowercase();
            if !s_target.starts_with(&s_root) {
                return Err(JagError::PathTraversal(path_str.into_owned()));
            }
        }
        #[cfg(not(windows))]
        {
            if !canon_target.starts_with(&canon_root) {
                return Err(JagError::PathTraversal(path_str.into_owned()));
            }
        }
 
        Ok(())
    }

    pub fn validate_command(&self, command: &str) -> Result<()> {
        for denied in &self.denied_commands {
            if command.contains(denied) {
                return Err(JagError::CommandDenied(command.to_string()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_tier_off() {
        let policy = SecurityPolicy::new(SecurityTier::Off, PathBuf::from("/tmp"));
        assert!(!policy.validate_action(&ActionType::ReadFile).unwrap());
        assert!(!policy.validate_action(&ActionType::ModifyFile).unwrap());
    }

    #[test]
    fn test_tier_auto() {
        let policy = SecurityPolicy::new(SecurityTier::Auto, PathBuf::from("/tmp"));
        
        // Allowed
        assert!(policy.validate_action(&ActionType::ReadFile).unwrap());
        assert!(policy.validate_action(&ActionType::SearchIndex).unwrap());
        assert!(policy.validate_action(&ActionType::FormatLint).unwrap());
        assert!(policy.validate_action(&ActionType::RunTests).unwrap());

        // Denied (Require prompt / false)
        assert!(!policy.validate_action(&ActionType::ModifyFile).unwrap());
        assert!(!policy.validate_action(&ActionType::GitOperation).unwrap());
        assert!(!policy.validate_action(&ActionType::NetworkAccess).unwrap());
    }

    #[test]
    fn test_tier_turbo() {
        let policy = SecurityPolicy::new(SecurityTier::Turbo, PathBuf::from("/tmp"));
        assert!(policy.validate_action(&ActionType::ModifyFile).unwrap());
        assert!(policy.validate_action(&ActionType::ReadFile).unwrap());
        assert!(policy.validate_action(&ActionType::NetworkAccess).unwrap());
    }

    #[test]
    fn test_path_traversal_dot_dot() {
        let policy = SecurityPolicy::new(SecurityTier::Auto, PathBuf::from("/workspace"));
        let res = policy.validate_path(Path::new("../../etc/passwd"));
        match res {
            Err(JagError::PathTraversal(p)) => assert!(p.contains("../../etc/passwd")),
            _ => panic!("Expected PathTraversal error"),
        }
    }

    #[test]
    fn test_path_traversal_absolute_outside() {
        let current_dir = env::current_dir().unwrap();
        // create a workspace inside current_dir so starts_with checks natively map over Windows pathing seamlessly
        let ws = current_dir.join("workspace");
        let policy = SecurityPolicy::new(SecurityTier::Auto, ws);
        
        // An absolute path outside
        // Adjust for OS
        #[cfg(windows)]
        let outside_path = Path::new("C:\\Windows\\System32\\cmd.exe");
        #[cfg(not(windows))]
        let outside_path = Path::new("/absolute/outside");

        let res = policy.validate_path(outside_path);
        match res {
            Err(JagError::PathTraversal(p)) => {
                #[cfg(windows)]
                assert!(p.contains("cmd.exe"));
                #[cfg(not(windows))]
                assert!(p.contains("/absolute/outside"));
            },
            _ => panic!("Expected PathTraversal error, got {:?}", res),
        }
    }

    #[test]
    fn test_path_valid_inside() {
        let current_dir = env::current_dir().unwrap();
        let ws = current_dir.join("workspace");
        let policy = SecurityPolicy::new(SecurityTier::Auto, ws);
        
        let valid = Path::new("src/main.rs");
        let res = policy.validate_path(valid);
        assert!(res.is_ok(), "Valid internal path should be Ok()");
    }

    #[test]
    fn test_denied_command() {
        let policy = SecurityPolicy::new(SecurityTier::Auto, PathBuf::from("/tmp"));
        let res = policy.validate_command("sudo rm -rf / hidden");
        
        match res {
            Err(JagError::CommandDenied(cmd)) => assert!(cmd.contains("rm -rf /")),
            _ => panic!("Expected CommandDenied error"),
        }
    }
}
