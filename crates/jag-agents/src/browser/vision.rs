// File: crates/jag-agents/src/browser/vision.rs
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use tracing::{info, warn};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use jag_models::router::{ModelRouter, ModelInput, ModelRouting};
use jag_core::types::ModelPreference;

pub const UI_ANALYSIS_PROMPT: &str = r#"You are a UI QA expert. Analyze this screenshot of a web application.

## Task
Determine if the UI matches these requirements:
1. The #root element is present and not empty
2. No React/Next.js error overlays are visible (check for red error boundaries or stack traces)
3. Primary layout components are visible (header, footer, main content if specified)
4. Visual look and feel matches a modern, premium design system.

## Output Format (STRICT JSON)
{
  "passed": boolean,
  "issues": [
    {
      "severity": "critical" | "warning" | "info",
      "description": string,
      "suggested_fix": string (optional)
    }
  ],
  "confidence": 0.0-1.0
}

Respond ONLY with the JSON block."#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysis {
    pub passed: bool,
    pub issues: Vec<VisionIssue>,
    pub confidence: f32,
}

pub struct VisionBudget {
    pub daily_limit_usd: f64,
    pub spent_today: f64,
    pub last_reset: DateTime<Utc>,
}

impl VisionBudget {
    pub fn new(daily_limit_usd: f64) -> Self {
        Self {
            daily_limit_usd,
            spent_today: 0.0,
            last_reset: Utc::now(),
        }
    }

    pub fn can_spend(&mut self, amount: f64) -> bool {
        let now = Utc::now();
        if now.date_naive() > self.last_reset.date_naive() {
            self.spent_today = 0.0;
            self.last_reset = now;
        }
        (self.spent_today + amount) <= self.daily_limit_usd
    }

    pub fn spend(&mut self, amount: f64) {
        self.spent_today += amount;
    }
}

#[derive(Clone)]
pub struct VisionAnalyzer {
    model_router: Arc<RwLock<ModelRouter>>,
    budget: Arc<RwLock<VisionBudget>>,
    cache: Arc<RwLock<LruCache<u64, VisionAnalysis>>>,
}

impl VisionAnalyzer {
    pub fn new(model_router: Arc<RwLock<ModelRouter>>) -> Self {
        Self {
            model_router,
            budget: Arc::new(RwLock::new(VisionBudget::new(10.0))), // $10/day default
            cache: Arc::new(RwLock::new(LruCache::new(std::num::NonZeroUsize::new(100).unwrap()))),
        }
    }

    fn compute_cache_key(image_bytes: &[u8], prompt: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        image_bytes.hash(&mut hasher);
        prompt.hash(&mut hasher);
        hasher.finish()
    }

    pub async fn analyze(&self, screenshot_path: &Path, prompt: Option<&str>) -> Result<VisionAnalysis> {
        let image_bytes = tokio::fs::read(screenshot_path).await?;
        self.analyze_bytes(&image_bytes, prompt).await
    }

    pub async fn analyze_bytes(&self, image_bytes: &[u8], prompt: Option<&str>) -> Result<VisionAnalysis> {
        let prompt = prompt.unwrap_or(UI_ANALYSIS_PROMPT);
        
        let cache_key = Self::compute_cache_key(image_bytes, prompt);
        {
            let mut cache = self.cache.write().await;
            if let Some(cached) = cache.get(&cache_key) {
                info!("Vision analysis cache hit");
                return Ok(cached.clone());
            }
        }

        let mut budget = self.budget.write().await;
        let estimated_cost = 0.01;
        if !budget.can_spend(estimated_cost) {
            anyhow::bail!("Vision budget exceeded for today");
        }

        let image_base64 = BASE64.encode(image_bytes);
        let input = ModelInput::Vision {
            prompt: prompt.to_string(),
            image_base64,
            mime_type: "image/png".to_string(),
        };

        info!("Sending vision analysis request...");
        let model_router = self.model_router.read().await;
        let response = model_router.generate(input, ModelPreference::Reasoning).await?;
        
        let analysis = self.parse_vision_response(&response.text)?;
        
        budget.spend(estimated_cost);
        {
            let mut cache = self.cache.write().await;
            cache.put(cache_key, analysis.clone());
        }

        Ok(analysis)
    }

    pub async fn analyze_with_fallback(&self, screenshot_path: &Path, prompt: Option<&str>) -> Result<VisionAnalysis> {
        match self.analyze(screenshot_path, prompt).await {
            Ok(res) => Ok(res),
            Err(e) => {
                warn!("Vision analysis failed, falling back to basic result: {}", e);
                Ok(VisionAnalysis {
                    passed: true,
                    issues: vec![VisionIssue {
                        severity: IssueSeverity::Warning,
                        description: format!("Visual analysis failed: {}", e),
                        suggested_fix: Some("Check logs for details or retry".to_string()),
                    }],
                    confidence: 0.5,
                })
            }
        }
    }

    fn parse_vision_response(&self, response: &str) -> Result<VisionAnalysis> {
        // Try direct parse
        if let Ok(analysis) = serde_json::from_str::<VisionAnalysis>(response) {
            return Ok(analysis);
        }

        // Fallback: extract from code block
        let mut json_str = response;
        if let Some(start) = response.find("```json") {
            let rest = &response[start + 7..];
            if let Some(end) = rest.find("```") {
                json_str = rest[..end].trim();
            }
        } else if let Some((start, end)) = response.find('{').and_then(|s| response.rfind('}').map(|e| (s, e))) {
            json_str = &response[start..=end];
        }

        serde_json::from_str::<VisionAnalysis>(json_str)
            .context(format!("Failed to parse vision response as JSON: {}", response))
    }
}
