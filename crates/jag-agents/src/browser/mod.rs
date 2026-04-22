// File: crates/jag-agents/src/browser/mod.rs
pub mod agent;
pub mod vision;
pub mod tester;
pub mod reference_store;
pub mod ensemble_evaluator;

pub use agent::{BrowserAgent, BrowserMode};
pub use tester::BrowserTester;
pub use vision::{VisionAnalyzer, VisionAnalysis, VisionIssue, IssueSeverity};
pub use reference_store::{ReferenceStore, ReferenceMetadata};
pub use ensemble_evaluator::{EnsembleVisualEvaluator, EnsembleVisualRequest, EnsembleVisualResult, CandidateResult};
