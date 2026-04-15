use serde::Deserialize;
use dotenvy::dotenv;
use crate::errors::{JagError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub database: DatabaseConfig,
    pub redis: Option<RedisConfig>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig { pub url: String }

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        config::Config::builder()
            .add_source(config::Environment::with_prefix("JAG").separator("_"))
            .build()
            .map_err(|e| JagError::ConfigurationError(e.to_string()))?
            .try_deserialize()
            .map_err(|e| JagError::ConfigurationError(e.to_string()))
    }

    pub fn validate_llm_config(&self) -> Result<()> {
        let has_cloud = self.llm.anthropic_api_key.is_some()
            || self.llm.openai_api_key.is_some()
            || self.llm.google_api_key.is_some()
            || self.llm.mistral_api_key.is_some();
        
        let has_local = reqwest::blocking::Client::new()
            .get(&self.llm.ollama_base_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
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
