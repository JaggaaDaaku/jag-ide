use std::sync::Arc;
use jag_core::types::{ModelPreference, ModelResponse};
use jag_core::errors::Result;
use crate::ollama_client::{OllamaClient, OllamaGenerateRequest, OllamaOptions};
use tracing::info;

#[derive(Debug, Clone)]
pub enum ModelInput {
    Text(String),
    Vision {
        prompt: String,
        image_base64: String, // raw base64 or with prefix
        mime_type: String,    // "image/png", etc.
    },
    Comparison {
        prompt: String,
        images: Vec<ComparisonImage>,
        strategy: ComparisonStrategy,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComparisonImage {
    pub base64: String,
    pub mime_type: String,
    pub label: String,
    pub role: ImageRole,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ImageRole {
    Reference,
    Candidate,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComparisonStrategy {
    /// For vision-language models: send all images in a single prompt
    SinglePrompt { max_images: usize },
    /// For models with context limits: compare pairwise against reference
    Pairwise { reference_index: usize },
    /// Score each individually, then aggregate (text-only or context constrained)
    Sequential,
}

#[async_trait::async_trait]
pub trait ModelRouting: Send + Sync {
    async fn generate(&self, input: ModelInput, preference: ModelPreference) -> Result<ModelResponse>;
    fn clone_box(&self) -> Arc<dyn ModelRouting>;
}

/// Routes LLM requests to the optimal model based on task preference.
#[derive(Clone)]
pub struct ModelRouter {
    client: OllamaClient,
    reasoning_model: String,
    code_model: String,
    fast_model: String,
}

#[async_trait::async_trait]
impl ModelRouting for ModelRouter {
    async fn generate(&self, input: ModelInput, preference: ModelPreference) -> Result<ModelResponse> {
        self.generate_internal(input, preference).await
    }

    fn clone_box(&self) -> Arc<dyn ModelRouting> {
        Arc::new(self.clone())
    }
}

impl ModelRouter {
    pub fn new(client: OllamaClient) -> Self {
        Self {
            client,
            reasoning_model: "qwen2.5:14b".into(),
            code_model: "qwen2.5-coder:14b".into(),
            fast_model: "qwen2.5:7b".into(),
        }
    }

    /// Create a router with custom model names.
    pub fn with_models(
        client: OllamaClient,
        reasoning_model: String,
        code_model: String,
        fast_model: String,
    ) -> Self {
        Self {
            client,
            reasoning_model,
            code_model,
            fast_model,
        }
    }

    /// Select the model name based on the preference tier.
    pub fn select_model(&self, preference: &ModelPreference) -> &str {
        match preference {
            ModelPreference::Reasoning => &self.reasoning_model,
            ModelPreference::CodeGeneration => &self.code_model,
            ModelPreference::Fast => &self.fast_model,
        }
    }

    /// Generate a response using the optimal model for the given preference.
    pub async fn generate_internal(&self, input: ModelInput, preference: ModelPreference) -> Result<ModelResponse> {
        match input {
            ModelInput::Text(prompt) => {
                let model = self.select_model(&preference).to_string();
                self.generate_with_specific_model(&prompt, &model, Some(preference)).await
            }
            ModelInput::Vision { prompt, image_base64, .. } => {
                self.generate_vision(&prompt, &image_base64, preference).await
            }
            ModelInput::Comparison { prompt, images, strategy } => {
                let model = self.select_model(&preference).to_string();
                self.generate_comparison(prompt, images, strategy, &model).await
            }
        }
    }

    /// Generate a response using a specific model name.
    pub async fn generate_with_specific_model(
        &self,
        prompt: &str,
        model_name: &str,
        preference: Option<ModelPreference>,
    ) -> Result<ModelResponse> {
        info!(model = %model_name, preference = ?preference, "Routing LLM request to specific model");

        let request = OllamaGenerateRequest {
            model: model_name.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(match preference {
                    Some(ModelPreference::Reasoning) => 0.3,
                    Some(ModelPreference::CodeGeneration) => 0.1,
                    Some(ModelPreference::Fast) | None => 0.5,
                }),
                num_ctx: Some(match preference {
                    Some(ModelPreference::Reasoning) | Some(ModelPreference::CodeGeneration) => 8192,
                    Some(ModelPreference::Fast) | None => 4096,
                }),
                ..Default::default()
            }),
            system: None,
            images: None,
        };

        let response = self.client.generate(request).await?;
        Ok(ModelResponse {
            text: response.response,
            model_name: response.model,
            prompt_tokens: response.prompt_eval_count,
            completion_tokens: response.eval_count,
            total_duration_ms: response.total_duration.map(|d| d / 1_000_000), // convert ns to ms
        })
    }

    /// Specialized multimodal generation logic.
    pub async fn generate_vision(&self, prompt: &str, image_base64: &str, preference: ModelPreference) -> Result<ModelResponse> {
        // In local Ollama, we usually route to the reasoning model or a specific vision model if available.
        // For now, we assume the reasoning_model or specialized vision model is capable.
        let model = self.select_model(&preference).to_string();
        info!(model = %model, preference = ?preference, "Routing Vision LLM request");

        // Strip base64 prefix if present (Ollama expects raw base64)
        let clean_base64 = if let Some(pos) = image_base64.find(',') {
            &image_base64[(pos + 1)..]
        } else {
            image_base64
        };

        let request = OllamaGenerateRequest {
            model: model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.2),
                num_ctx: Some(8192),
                ..Default::default()
            }),
            system: None,
            images: Some(vec![clean_base64.to_string()]),
        };

        let response = self.client.generate(request).await?;
        Ok(ModelResponse {
            text: response.response,
            model_name: response.model,
            prompt_tokens: response.prompt_eval_count,
            completion_tokens: response.eval_count,
            total_duration_ms: response.total_duration.map(|d| d / 1_000_000),
        })
    }

    /// Comparison generation logic for judging.
    pub async fn generate_comparison(
        &self,
        prompt: String,
        images: Vec<ComparisonImage>,
        strategy: ComparisonStrategy,
        model_name: &str,
    ) -> Result<ModelResponse> {
        info!(model = %model_name, strategy = ?strategy, num_images = images.len(), "Routing Comparison LLM request");

        match strategy {
            ComparisonStrategy::SinglePrompt { .. } => {
                let mut base64_images = Vec::new();
                for img in images {
                    let clean = if let Some(pos) = img.base64.find(',') {
                        &img.base64[(pos + 1)..]
                    } else {
                        &img.base64
                    };
                    base64_images.push(clean.to_string());
                }

                let request = OllamaGenerateRequest {
                    model: model_name.to_string(),
                    prompt,
                    stream: false,
                    options: Some(OllamaOptions {
                        temperature: Some(0.1),
                        num_ctx: Some(16384),
                        ..Default::default()
                    }),
                    system: None,
                    images: Some(base64_images),
                };

                let response = self.client.generate(request).await?;
                Ok(ModelResponse {
                    text: response.response,
                    model_name: response.model,
                    prompt_tokens: response.prompt_eval_count,
                    completion_tokens: response.eval_count,
                    total_duration_ms: response.total_duration.map(|d| d / 1_000_000),
                })
            }
            ComparisonStrategy::Sequential => {
                self.generate_with_specific_model(&prompt, model_name, Some(ModelPreference::Reasoning)).await
            }
            ComparisonStrategy::Pairwise { .. } => {
                self.generate_with_specific_model(&prompt, model_name, Some(ModelPreference::Reasoning)).await
            }
        }
    }

    /// Generate with a custom system prompt.
    pub async fn generate_with_system(
        &self,
        prompt: &str,
        system: &str,
        preference: ModelPreference,
    ) -> Result<ModelResponse> {
        let model = self.select_model(&preference).to_string();
        info!(model = %model, preference = ?preference, "Routing LLM request with system prompt");

        let request = OllamaGenerateRequest {
            model,
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.2),
                num_ctx: Some(8192),
                ..Default::default()
            }),
            system: Some(system.to_string()),
            images: None,
        };

        let response = self.client.generate(request).await?;
        Ok(ModelResponse {
            text: response.response,
            model_name: response.model,
            prompt_tokens: response.prompt_eval_count,
            completion_tokens: response.eval_count,
            total_duration_ms: response.total_duration.map(|d| d / 1_000_000),
        })
    }

    /// Check which models are actually available on the Ollama instance
    /// and auto-configure to the best available.
    pub async fn auto_configure(&mut self) -> Result<()> {
        let models = self.client.list_models().await?;
        let available: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

        info!(available = ?available, "Auto-configuring model router");

        // Find the largest available model for reasoning
        if let Some(model) = Self::find_best_match(&available, &["qwen2.5:14b", "qwen2.5:7b", "llama3:8b", "gemma2:9b"]) {
            self.reasoning_model = model;
        }

        // Find a code model
        if let Some(model) = Self::find_best_match(&available, &["qwen2.5-coder:14b", "qwen2.5-coder:7b", "codellama:7b"]) {
            self.code_model = model;
        }

        // Find a fast model
        if let Some(model) = Self::find_best_match(&available, &["qwen2.5:7b", "qwen2.5:3b", "gemma2:2b", "phi3:mini"]) {
            self.fast_model = model;
        }

        info!(
            reasoning = %self.reasoning_model,
            code = %self.code_model,
            fast = %self.fast_model,
            "Model router configured"
        );

        Ok(())
    }

    /// Find the first available model from a preference-ordered list.
    fn find_best_match(available: &[String], preferences: &[&str]) -> Option<String> {
        for pref in preferences {
            if available.iter().any(|a| a == pref) {
                return Some(pref.to_string());
            }
        }
        // Fallback: use the first available model
        available.first().cloned()
    }

    /// Get a reference to the underlying client.
    pub fn client(&self) -> &OllamaClient {
        &self.client
    }

    /// Get the names of the currently configured models.
    pub fn available_models(&self) -> Vec<String> {
        vec![
            self.reasoning_model.clone(),
            self.code_model.clone(),
            self.fast_model.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection() {
        let client = OllamaClient::new("http://localhost:11434");
        let router = ModelRouter::new(client);

        assert_eq!(router.select_model(&ModelPreference::Reasoning), "qwen2.5:14b");
        assert_eq!(router.select_model(&ModelPreference::CodeGeneration), "qwen2.5-coder:14b");
        assert_eq!(router.select_model(&ModelPreference::Fast), "qwen2.5:7b");
    }

    #[test]
    fn test_custom_models() {
        let client = OllamaClient::new("http://localhost:11434");
        let router = ModelRouter::with_models(
            client,
            "llama3:70b".into(),
            "codestral:22b".into(),
            "phi3:mini".into(),
        );

        assert_eq!(router.select_model(&ModelPreference::Reasoning), "llama3:70b");
        assert_eq!(router.select_model(&ModelPreference::CodeGeneration), "codestral:22b");
        assert_eq!(router.select_model(&ModelPreference::Fast), "phi3:mini");
    }

    #[test]
    fn test_find_best_match() {
        let available = vec!["qwen2.5:7b".into(), "gemma2:2b".into()];

        // Should find qwen2.5:7b from the preference list
        let result = ModelRouter::find_best_match(&available, &["qwen2.5:14b", "qwen2.5:7b"]);
        assert_eq!(result, Some("qwen2.5:7b".into()));

        // None match the preferences, falls back to first available
        let result = ModelRouter::find_best_match(&available, &["llama3:70b"]);
        assert_eq!(result, Some("qwen2.5:7b".into()));

        // Empty available list
        let result = ModelRouter::find_best_match(&[], &["qwen2.5:14b"]);
        assert_eq!(result, None);
    }
}
