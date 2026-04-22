use serde::{Deserialize, Serialize};
use chrono::Utc;
use jag_core::types::{ModelPreference, BenchmarkResult, TaskId};
use jag_core::errors::Result;
use jag_models::router::{ModelRouting, ModelInput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetric {
    pub model_name: String,
    pub avg_latency_ms: u64,
    pub avg_tps: f64,
    pub reliability_score: f32, // 0.0 - 1.0
}

#[async_trait::async_trait]
pub trait Benchmarking {
    async fn run_benchmark(&self, model_name: &str, input: ModelInput) -> Result<BenchmarkResult>;
}

pub struct BenchmarkRunner {
    router: std::sync::Arc<dyn ModelRouting>,
}

impl BenchmarkRunner {
    pub fn new(router: std::sync::Arc<dyn ModelRouting>) -> Self {
        Self { router }
    }

    pub async fn benchmark_prompt(&self, model_name: &str, prompt: &str) -> Result<BenchmarkResult> {
        let start = std::time::Instant::now();
        let input = ModelInput::Text(prompt.to_string());
        
        let response = self.router.generate(input, ModelPreference::Reasoning).await?;
        let duration = start.elapsed();
        
        let total_tokens = response.completion_tokens.unwrap_or(0) + response.prompt_tokens.unwrap_or(0);
        let tps = if duration.as_secs_f64() > 0.0 {
            response.completion_tokens.unwrap_or(0) as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Ok(BenchmarkResult {
            id: TaskId::new(),
            model_name: model_name.to_string(),
            task_type: "text_generation".to_string(),
            latency_ms: duration.as_millis() as u64,
            tokens_per_second: tps,
            total_tokens,
            cost_usd: 0.0, // TBD: Price registry integration
            timestamp: Utc::now(),
            success: true,
        })
    }
}
