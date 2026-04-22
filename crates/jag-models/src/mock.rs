use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use jag_core::types::{ModelPreference, ModelResponse};
use jag_core::errors::{JagError, Result};
use crate::router::{ModelRouting, ModelInput};

/// Mock ModelRouter for testing — returns deterministic responses
/// without making real Ollama calls.
#[derive(Clone, Default)]
pub struct MockModelRouter {
    responses: Arc<Mutex<HashMap<String, String>>>,
}

impl MockModelRouter {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn with_response(self, prompt_contains: &str, response: &str) -> Self {
        self.responses.lock().await.insert(prompt_contains.to_string(), response.to_string());
        self
    }
}

#[async_trait::async_trait]
impl ModelRouting for MockModelRouter {
    async fn generate(&self, input: ModelInput, _preference: ModelPreference) -> Result<ModelResponse> {
        let prompt = match input {
            ModelInput::Text(p) => p,
            ModelInput::Vision { prompt, .. } => prompt,
            ModelInput::Comparison { prompt, .. } => prompt,
        };

        let responses = self.responses.lock().await;
        for (key, value) in responses.iter() {
            if prompt.contains(key) {
                return Ok(ModelResponse {
                    text: value.clone(),
                    model_name: "mock-model".into(),
                    prompt_tokens: Some(10),
                    completion_tokens: Some(50),
                    total_duration_ms: Some(100),
                });
            }
        }

        Err(JagError::Internal(format!("No mock response found for prompt containing any of: {:?}", responses.keys().collect::<Vec<_>>())))
    }

    fn clone_box(&self) -> Arc<dyn ModelRouting> {
        Arc::new(self.clone())
    }
}
