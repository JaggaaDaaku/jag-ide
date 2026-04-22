use std::sync::Arc;
use serde::{Serialize, Deserialize};
use jag_core::types::{ViewportSpec, DesignSpec, Priority, ArtifactId};
use jag_core::errors::{JagError, Result};
use jag_db::Database;
use jag_models::router::ModelRouter;
use crate::browser::{BrowserAgent, VisionAnalyzer, ReferenceStore};
use tracing::{info, warn, error};
use base64::Engine;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnsembleVisualResult {
    pub winning_candidate: CandidateResult,
    pub all_candidates: Vec<CandidateResult>,
    pub selection_reason: String,
    pub confidence: f32,
    pub judge_feedback: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CandidateResult {
    pub model_name: String,
    pub generated_code: String,
    pub screenshot_base64: String,
    pub vision_score: Option<f32>,
    pub pixel_similarity: Option<f32>,
    pub code_quality_score: f32,
    pub overall_score: f32,
    pub judge_score: Option<f32>,
    pub judge_comment: Option<String>,
}

pub struct EnsembleVisualRequest {
    pub prompt: String,
    pub artifact_id: ArtifactId,
    pub design_reference: Option<DesignSpec>,
    pub candidate_models: Vec<String>,
    pub viewport: ViewportSpec,
    pub priority: Priority,
    pub enable_judge: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JudgeDecision {
    pub winner_index: usize,
    pub scores: Vec<JudgeScore>,
    pub reasoning: String,
    pub confidence: f32,
}

#[derive(Debug, Deserialize)]
pub struct JudgeScore {
    pub candidate_index: usize,
    pub overall: f32,
    pub reasoning: String,
}

pub struct JudgeConfig {
    pub primary_judge: String,
    pub fallback_judge: String,
    pub vision_model: String,
    pub enabled_for_priorities: Vec<Priority>,
    pub trigger_on_close_scores: bool,
    pub close_score_threshold: f32,
    pub timeout_secs: u64,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            primary_judge: "qwen3.5:27b".into(),
            fallback_judge: "deepseek-r1:8b".into(),
            vision_model: "qwen3-vl:30b".into(),
            enabled_for_priorities: vec![Priority::High, Priority::Critical],
            trigger_on_close_scores: true,
            close_score_threshold: 0.05,
            timeout_secs: 45,
        }
    }
}

pub const JUDGE_COMPARISON_PROMPT: &str = r#"You are an expert UI/UX designer and frontend engineer evaluating multiple implementations of the same design.

## Task
Compare these {num_candidates} candidate UIs and select the best one based on:

## Evaluation Criteria (Weighted)
1. **Visual Fidelity** (40%): Layout, spacing, colors, typography match to reference
2. **Functionality** (30%): Interactive elements present and properly styled
3. **Code Quality** (20%): Clean, modular, best practices
4. **Accessibility** (10%): ARIA labels, contrast, keyboard navigation

## Output Format (STRICT JSON)
{{
  "winner_index": 0,
  "scores": [
    {{ "candidate_index": 0, "overall": 0.87, "reasoning": "..." }},
    ...
  ],
  "reasoning": "Detailed explanation...",
  "confidence": 0.92
}}

Design specification: {design_spec_text}

Analyze the provided candidate screenshots and respond in JSON only."#;

pub struct EnsembleVisualEvaluator {
    model_router: Arc<ModelRouter>,
    browser_agent: BrowserAgent,
    reference_store: ReferenceStore,
    vision_analyzer: VisionAnalyzer,
    db: Arc<Database>,
    cost_config: jag_core::config::CostConfig,
    config: JudgeConfig,
}

impl EnsembleVisualEvaluator {
    pub fn new(
        model_router: Arc<ModelRouter>,
        browser_agent: BrowserAgent,
        reference_store: ReferenceStore,
        vision_analyzer: VisionAnalyzer,
        db: Arc<Database>,
        cost_config: jag_core::config::CostConfig,
    ) -> Self {
        Self {
            model_router,
            browser_agent,
            reference_store,
            vision_analyzer,
            db,
            cost_config,
            config: JudgeConfig::default(),
        }
    }

    pub async fn evaluate(&self, request: EnsembleVisualRequest) -> Result<EnsembleVisualResult> {
        info!(
            models = ?request.candidate_models,
            "Starting ensemble visual evaluation"
        );

        // 1. Generate code candidates in parallel
        let mut handles = Vec::new();
        for model in &request.candidate_models {
            let router = self.model_router.clone();
            let prompt = request.prompt.clone();
            let model_name = model.clone();
            
            handles.push(tokio::spawn(async move {
                router.generate_with_specific_model(&prompt, &model_name, None).await
            }));
        }

        let mut candidates_code = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            let model_name = request.candidate_models[i].clone();
            match handle.await {
                Ok(Ok(resp)) => {
                    let code = resp.text.clone();
                    // Record usage
                    let record = self.calculate_usage(&resp);
                    let _ = self.db.log_model_usage(record).await;
                    
                    candidates_code.push((model_name, code));
                }
                Ok(Err(e)) => warn!(model = %model_name, error = %e, "Candidate generation failed"),
                Err(e) => error!(model = %model_name, error = %e, "Candidate task join failed"),
            }
        }

        if candidates_code.is_empty() {
            return Err(JagError::Internal("All candidate generations failed".into()));
        }

        // 2. Evaluate each candidate
        let mut evaluated = Vec::new();
        for (model_name, code) in candidates_code {
            let code_base64 = base64::engine::general_purpose::STANDARD.encode(code.as_bytes());
            let data_url = format!("data:text/html;base64,{}", code_base64);
            
            let screenshot_path = self.browser_agent
                .capture_screenshot(&request.artifact_id.to_string(), &data_url, None)
                .await
                .map_err(|e| JagError::Internal(format!("Failed to capture screenshot: {}", e)))?;
            
            let screenshot = tokio::fs::read(&screenshot_path).await
                .map_err(JagError::Io)?;
            
            let vision_score = if let Some(design) = &request.design_reference {
                let vision_prompt = format!(
                    "Evaluate this UI according to the following design spec: {}. \
                     Rate from 0.0 to 1.0 based on layout, color, and component matching.",
                    design.description
                );
                
                self.vision_analyzer
                    .analyze_bytes(&screenshot, Some(&vision_prompt))
                    .await
                    .ok()
                    .map(|ana| ana.confidence)
            } else {
                None
            };

            let pixel_similarity = self.reference_store
                .load_reference(&request.artifact_id.to_string(), &request.viewport)
                .await?
                .and_then(|(_, meta)| {
                    let img = image::load_from_memory(&screenshot).ok()?;
                    let current_phash = self.reference_store.compute_phash(&img);
                    ReferenceStore::compute_similarity(&meta.phash, &current_phash).ok()
                });

            let code_quality = self.score_code_quality(&code);

            let overall_score = self.compute_overall_score(
                vision_score,
                pixel_similarity,
                code_quality,
                &request.priority,
            );

            evaluated.push(CandidateResult {
                model_name,
                generated_code: code,
                screenshot_base64: base64::engine::general_purpose::STANDARD.encode(&screenshot),
                vision_score,
                pixel_similarity,
                code_quality_score: code_quality,
                overall_score,
                judge_score: None,
                judge_comment: None,
            });
        }

        // 3. LLM-as-Judge Decision
        let judge_decision = self.maybe_run_judge(&mut evaluated, &request).await.ok().flatten();

        // 4. Final Selection
        let (winner, selection_reason, judge_feedback) = if let Some(decision) = judge_decision {
            let win_idx = decision.winner_index.min(evaluated.len() - 1);
            let winner = evaluated[win_idx].clone();
            (winner, "LLM Judge selection".to_string(), Some(decision.reasoning))
        } else {
            let winner = evaluated.iter()
                .max_by(|a, b| a.overall_score.partial_cmp(&b.overall_score).unwrap())
                .cloned()
                .unwrap();
            
            let reason = format!("Heuristic winner based on {} priority", match request.priority {
                Priority::High | Priority::Critical => "high",
                _ => "standard",
            });
            (winner, reason, None)
        };

        Ok(EnsembleVisualResult {
            winning_candidate: winner,
            all_candidates: evaluated,
            selection_reason,
            confidence: 0.90,
            judge_feedback,
        })
    }

    pub async fn judge_candidates(
        &self,
        candidates: &mut [CandidateResult],
        request: &EnsembleVisualRequest,
    ) -> Result<JudgeDecision> {
        use jag_models::router::{ComparisonImage, ImageRole, ComparisonStrategy};

        info!("Running LLM Judge for best candidate selection");

        let mut images = Vec::new();
        // Add candidates to comparison
        for (i, candidate) in candidates.iter().enumerate() {
            images.push(ComparisonImage {
                base64: candidate.screenshot_base64.clone(),
                mime_type: "image/png".into(),
                label: format!("candidate_{}", i),
                role: ImageRole::Candidate,
            });
        }

        let design_spec = request.design_reference.as_ref()
            .map(|d| d.description.as_str())
            .unwrap_or("No specific design spec provided.");

        let prompt = JUDGE_COMPARISON_PROMPT
            .replace("{num_candidates}", &candidates.len().to_string())
            .replace("{design_spec_text}", design_spec);

        let response = self.model_router
            .generate_comparison(
                prompt,
                images,
                ComparisonStrategy::SinglePrompt { max_images: 4 },
                &self.config.vision_model,
            )
            .await?;

        // Record usage
        let record = self.calculate_usage(&response);
        let _ = self.db.log_model_usage(record).await;

        let response_text = response.text;

        // Extract JSON from response (naive parsing)
        let json_start = response_text.find('{').ok_or_else(|| JagError::Internal("No JSON in judge response".into()))?;
        let json_end = response_text.rfind('}').ok_or_else(|| JagError::Internal("No JSON in judge response".into()))?;
        let json_str = &response_text[json_start..=json_end];

        let decision: JudgeDecision = serde_json::from_str(json_str)
            .map_err(|e| JagError::Internal(format!("Failed to parse judge JSON: {}", e)))?;

        // Backfill scores to candidates
        for score in &decision.scores {
            if let Some(cand) = candidates.get_mut(score.candidate_index) {
                cand.judge_score = Some(score.overall);
                cand.judge_comment = Some(score.reasoning.clone());
            }
        }

        Ok(decision)
    }

    async fn maybe_run_judge(
        &self,
        candidates: &mut [CandidateResult],
        request: &EnsembleVisualRequest,
    ) -> Result<Option<JudgeDecision>> {
        // Condition 1: Explicitly enabled/disabled
        if let Some(enabled) = request.enable_judge {
            if !enabled { return Ok(None); }
        } else if !self.config.enabled_for_priorities.contains(&request.priority) {
            // Condition 2: Priority check
            // Check margin only if NOT high priority
            if self.config.trigger_on_close_scores {
                let mut scores: Vec<f32> = candidates.iter().map(|c| c.overall_score).collect();
                scores.sort_by(|a, b| b.partial_cmp(a).unwrap());
                
                if scores.len() >= 2 {
                    let margin = (scores[0] - scores[1]).abs();
                    if margin > self.config.close_score_threshold {
                        info!(margin = %margin, "Heuristic margin exceeds threshold; skipping judge");
                        return Ok(None);
                    }
                }
            } else {
                return Ok(None);
            }
        }

        match tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout_secs),
            self.judge_candidates(candidates, request)
        ).await {
            Ok(Ok(decision)) => Ok(Some(decision)),
            Ok(Err(e)) => {
                warn!(error = %e, "Judge failed; falling back to heuristics");
                Ok(None)
            }
            Err(_) => {
                warn!("Judge timed out; falling back to heuristics");
                Ok(None)
            }
        }
    }

    fn compute_overall_score(
        &self,
        vision: Option<f32>,
        pixel: Option<f32>,
        code_quality: f32,
        priority: &Priority,
    ) -> f32 {
        let (vision_weight, pixel_weight, code_weight) = match priority {
            Priority::Low => (0.1, 0.1, 0.8),
            Priority::Normal => (0.3, 0.3, 0.4),
            Priority::High => (0.4, 0.4, 0.2),
            Priority::Critical => (0.5, 0.4, 0.1),
        };

        let vision_val = vision.unwrap_or(0.5);
        let pixel_val = pixel.unwrap_or(0.5);
        
        (vision_val * vision_weight) + (pixel_val * pixel_weight) + (code_quality * code_weight)
    }

    fn score_code_quality(&self, code: &str) -> f32 {
        let mut score: f32 = 0.5;
        if code.contains("<!DOCTYPE html>") || code.contains("<html") { score += 0.1; }
        if code.contains("style>") || code.contains("class=") { score += 0.1; }
        if code.contains("script>") { score += 0.1; }
        
        // Penalize empty or extremely short code
        if code.len() < 100 { score -= 0.3; }
        
        score.clamp(0.0, 1.0)
    }

    fn calculate_usage(&self, resp: &jag_core::types::ModelResponse) -> jag_core::types::ModelUsageRecord {
        let prompt_tokens = resp.prompt_tokens.unwrap_or(0) as usize;
        let completion_tokens = resp.completion_tokens.unwrap_or(0) as usize;
        let total_tokens = prompt_tokens + completion_tokens;
        
        // Match user's hybrid cost recommendation
        let is_local = resp.model_name.contains("ollama") || !self.cost_config.cloud_rates.contains_key(&resp.model_name);
        
        let cost = if is_local {
            match &self.cost_config.local_model_strategy {
                jag_core::config::LocalCostStrategy::PerToken { rate_per_1k } => {
                    (total_tokens as f64 / 1000.0) * rate_per_1k
                },
                jag_core::config::LocalCostStrategy::PerCall { rate_per_call } => *rate_per_call,
            }
        } else {
            let rate = self.cost_config.cloud_rates.get(&resp.model_name).unwrap_or(&0.01);
            (total_tokens as f64 / 1000.0) * rate
        };

        jag_core::types::ModelUsageRecord {
            model_name: resp.model_name.clone(),
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cost_estimated: cost,
            timestamp: chrono::Utc::now(),
            is_local,
        }
    }
}
