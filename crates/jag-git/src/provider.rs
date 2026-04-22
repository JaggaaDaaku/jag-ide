use async_trait::async_trait;
use jag_core::errors::{JagError, Result};
use jag_core::config::GitConfig;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub html_url: String,
    pub state: String,
}

#[async_trait]
pub trait GitProvider: Send + Sync {
    async fn create_pr(
        &self,
        title: &str,
        body: &str,
        head_branch: &str,
        base_branch: &str,
    ) -> Result<PullRequest>;

    async fn get_pr_status(&self, pr_number: u64) -> Result<PullRequest>;
}

pub struct GitHubProvider {
    client: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubProvider {
    pub fn new(config: &GitConfig) -> Result<Self> {
        let token = config.github_token.as_ref().ok_or_else(|| {
            JagError::ConfigurationError("GitHub token missing".into())
        })?;

        let client = Octocrab::builder()
            .personal_token(token.clone())
            .build()
            .map_err(|e| JagError::Internal(format!("Failed to build Octocrab: {}", e)))?;

        Ok(Self {
            client,
            owner: config.owner.clone(),
            repo: config.repo.clone(),
        })
    }
}

#[async_trait]
impl GitProvider for GitHubProvider {
    async fn create_pr(
        &self,
        title: &str,
        body: &str,
        head_branch: &str,
        base_branch: &str,
    ) -> Result<PullRequest> {
        let pr = self.client
            .pulls(&self.owner, &self.repo)
            .create(title, head_branch, base_branch)
            .body(body)
            .send()
            .await
            .map_err(|e| JagError::Internal(format!("Failed to create PR: {}", e)))?;

        Ok(PullRequest {
            number: pr.number,
            html_url: pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
            state: pr.state.map(|s| format!("{:?}", s)).unwrap_or_else(|| "unknown".into()),
        })
    }

    async fn get_pr_status(&self, pr_number: u64) -> Result<PullRequest> {
        let pr = self.client
            .pulls(&self.owner, &self.repo)
            .get(pr_number)
            .await
            .map_err(|e| JagError::Internal(format!("Failed to get PR: {}", e)))?;

        Ok(PullRequest {
            number: pr.number,
            html_url: pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
            state: pr.state.map(|s| format!("{:?}", s)).unwrap_or_else(|| "unknown".into()),
        })
    }
}

pub fn create_provider(config: &GitConfig) -> Result<Box<dyn GitProvider>> {
    match config.provider.as_str() {
        "github" => Ok(Box::new(GitHubProvider::new(config)?)),
        _ => Err(JagError::ConfigurationError(format!("Unsupported git provider: {}", config.provider))),
    }
}
