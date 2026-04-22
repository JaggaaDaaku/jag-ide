use serde::Deserialize;
use dotenvy::dotenv;
use crate::errors::{JagError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub cost: CostConfig,
    pub audit: AuditConfig,
    pub git: GitConfig,
    pub validation: ValidationConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitConfig {
    pub provider: String,
    pub github_token: Option<String>,
    pub owner: String,
    pub repo: String,
    pub prefix: String,
    pub max_slug_words: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidationConfig {
    pub rust_coverage_threshold: f32,
    pub ts_coverage_threshold: f32,
    pub mock_mode: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub oidc_enabled: bool,
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub oidc_redirect_url: String,
    pub oidc_scopes: Vec<String>,
    pub jwt_secret: String,
    pub hmac_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostConfig {
    pub cloud_rates: std::collections::HashMap<String, f64>,
    pub local_model_strategy: LocalCostStrategy,
}

#[derive(Debug, Clone, Deserialize)]
pub enum LocalCostStrategy {
    PerToken { rate_per_1k: f64 },
    PerCall { rate_per_call: f64 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditConfig {
    pub hmac_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    pub ollama_base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig { pub url: String }


impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        config::Config::builder()
            .add_source(config::File::with_name("settings").required(false))
            .add_source(config::Environment::with_prefix("JAG").separator("__"))
            .build()
            .map_err(|e| JagError::ConfigurationError(e.to_string()))?
            .try_deserialize()
            .map_err(|e| JagError::ConfigurationError(e.to_string()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                anthropic_api_key: None,
                openai_api_key: None,
                google_api_key: None,
                mistral_api_key: None,
                ollama_base_url: "http://localhost:11434".into(),
            },
            database: DatabaseConfig { url: "sqlite::memory:".into() },
            auth: AuthConfig {
                oidc_enabled: false,
                oidc_issuer_url: "".into(),
                oidc_client_id: "".into(),
                oidc_client_secret: "".into(),
                oidc_redirect_url: "".into(),
                oidc_scopes: vec![],
                jwt_secret: "test_secret".into(),
                hmac_secret: "test_secret".into(),
            },
            cost: CostConfig {
                cloud_rates: std::collections::HashMap::new(),
                local_model_strategy: LocalCostStrategy::PerCall { rate_per_call: 0.0 },
            },
            audit: AuditConfig {
                hmac_secret: "test_secret".into(),
            },
            git: GitConfig {
                provider: "github".into(),
                github_token: None,
                owner: "test".into(),
                repo: "test".into(),
                prefix: "jag/".into(),
                max_slug_words: 5,
            },
            validation: ValidationConfig {
                rust_coverage_threshold: 80.0,
                ts_coverage_threshold: 80.0,
                mock_mode: true,
            },
            telemetry: TelemetryConfig {
                enabled: false,
            },
        }
    }
}

impl Config {
    pub async fn validate_llm_config(&self) -> Result<()> {
        let has_cloud = self.llm.anthropic_api_key.is_some()
            || self.llm.openai_api_key.is_some()
            || self.llm.google_api_key.is_some()
            || self.llm.mistral_api_key.is_some();
        
        let has_local = reqwest::Client::new()
            .get(&self.llm.ollama_base_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        
        if !has_cloud && !has_local {
            return Err(JagError::ConfigurationError(
                "At least one LLM provider must be configured (cloud API key or local Ollama)".into()
            ));
        }
        Ok(())
    }
}
