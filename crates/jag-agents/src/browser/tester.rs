use super::agent::BrowserAgent;
use super::vision::{VisionAnalyzer, VisionAnalysis};
use super::reference_store::ReferenceStore;
use super::ensemble_evaluator::{EnsembleVisualEvaluator, EnsembleVisualRequest, EnsembleVisualResult};
use jag_models::router::ModelRouter;
use anyhow::Result;
use std::time::Duration;
use tracing::{info, warn};
use std::path::PathBuf;
use std::sync::Arc;

use jag_db::Database;
use jag_core::config::CostConfig;

pub struct BrowserTester {
    pub agent: BrowserAgent,
    pub vision_analyzer: Option<Arc<VisionAnalyzer>>,
    pub reference_store: ReferenceStore,
    pub model_router: Arc<ModelRouter>,
    pub db: Arc<Database>,
    pub cost_config: CostConfig,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UiSmokeTestResult {
    pub screenshot_path: PathBuf,
    pub vision_analysis: Option<VisionAnalysis>,
}

impl BrowserTester {
    pub fn new(
        agent: BrowserAgent,
        vision_analyzer: Option<Arc<VisionAnalyzer>>,
        reference_store: ReferenceStore,
        model_router: Arc<ModelRouter>,
        db: Arc<Database>,
        cost_config: CostConfig,
    ) -> Self {
        Self { agent, vision_analyzer, reference_store, model_router, db, cost_config }
    }

    /// Robust execution wrapper with retry logic for flaky UI tests.
    pub async fn run_with_retry<F, Fut, T>(
        &self,
        max_attempts: u32,
        delay: Duration,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("UI Test attempt {}/{} failed: {}", attempt, max_attempts, e);
                    last_error = Some(e);
                    if attempt < max_attempts {
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown test failure")))
    }

    /// Performs a high-level smoke test on a generated UI.
    pub async fn run_ui_smoke_test(&self, artifact_id: &str, url: &str) -> Result<UiSmokeTestResult> {
        info!("Starting Browser Smoke Test for artifact: {} at {}", artifact_id, url);

        self.run_with_retry(3, Duration::from_secs(2), || async {
            let screenshot = self.agent.capture_screenshot(artifact_id, url, None).await?;
            info!("Screenshot captured at: {:?}", screenshot);

            let mut vision_analysis = None;
            if let Some(analyzer) = &self.vision_analyzer {
                info!("Starting visual analysis for artifact: {}", artifact_id);
                vision_analysis = Some(analyzer.analyze_with_fallback(&screenshot, None).await?);
            }
            
            info!("Smoke test successful for: {}", artifact_id);
            Ok(UiSmokeTestResult {
                screenshot_path: screenshot,
                vision_analysis,
            })
        }).await
    }

    /// Run a multi-model ensemble evaluation to find the best visual match for a prompt.
    pub async fn run_ensemble_visual_test(
        &self,
        request: EnsembleVisualRequest,
    ) -> Result<EnsembleVisualResult> {
        info!("Starting Ensemble Visual Test for artifact: {}", request.artifact_id);

        let analyzer = self.vision_analyzer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VisionAnalyzer not initialized"))?;

        let evaluator = EnsembleVisualEvaluator::new(
            self.model_router.clone(),
            self.agent.clone(),
            self.reference_store.clone(),
            (**analyzer).clone(),
            self.db.clone(),
            self.cost_config.clone(),
        );

        evaluator.evaluate(request).await.map_err(|e| anyhow::anyhow!(e))
    }
}
