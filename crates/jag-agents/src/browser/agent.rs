// File: crates/jag-agents/src/browser/agent.rs
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use serde::{Serialize, Deserialize};
use serde_json::json;
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use chrono::Utc;
use jag_core::types::BrowserConfig;
use image::GenericImageView;

#[derive(Debug, Clone, Default)]
pub enum BrowserMode {
    #[default]
    Headless,
    Headful {
        devtools: bool,
    },
}

#[derive(Debug, Clone)]
pub struct BrowserAgent {
    pub workspace_root: PathBuf,
    pub config: BrowserConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserCommand {
    pub action: String,
    pub url: Option<String>,
    pub selector: Option<String>,
    pub path: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserResponse {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl BrowserAgent {
    pub fn new(workspace_root: PathBuf, config: BrowserConfig) -> Self {
        Self { workspace_root, config }
    }

    /// Explicitly check for Node.js v18+ in the current environment.
    pub async fn has_compatible_node() -> bool {
        let output = Command::new("node")
            .arg("--version")
            .output()
            .await;
        
        match output {
            Ok(out) => {
                let version = String::from_utf8_lossy(&out.stdout);
                info!("Detected Node.js version: {}", version.trim());
                // Simple v18/v20/v22 check
                version.contains("v18") || version.contains("v20") || version.contains("v22")
            }
            Err(_) => false,
        }
    }

    /// Ensure playwright and browsers are installed via npx.
    pub async fn ensure_playwright_installed() -> Result<()> {
        let status = Command::new("npx")
            .arg("playwright")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;
        
        if !status.success() {
            anyhow::bail!("Playwright not found. Please run 'npx playwright install' in your project root.");
        }
        Ok(())
    }

    pub async fn run_command(&self, cmd: BrowserCommand) -> Result<BrowserResponse> {
        let runner_path = self.workspace_root.join("crates/jag-agents/src/browser/playwright_runner.js");
        
        let cmd_with_config = json!({
            "action": cmd.action,
            "url": cmd.url,
            "selector": cmd.selector,
            "path": cmd.path,
            "timeout_ms": cmd.timeout_ms.unwrap_or(self.config.test_timeout_ms),
            "headless": self.config.headless,
            "viewport": {
                "width": self.config.viewport.width,
                "height": self.config.viewport.height,
            }
        });

        let cmd_json = serde_json::to_string(&cmd_with_config)?;
        
        let output = Command::new("node")
            .arg(runner_path)
            .arg(cmd_json)
            .output()
            .await
            .context("Failed to execute playwright_runner.js")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Playwright runner failed: {}", stderr);
        }

        let resp: BrowserResponse = serde_json::from_slice(&output.stdout)
            .context("Failed to parse response from playwright_runner.js")?;
        
        Ok(resp)
    }

    pub async fn capture_screenshot(&self, artifact_id: &str, url: &str, selector: Option<&str>) -> Result<PathBuf> {
        let timestamp = Utc::now().timestamp();
        let filename = match selector {
            Some(sel) => format!("screenshot_{}_{}.png", sel.replace(['/', '#', '.'], "_"), timestamp),
            None => format!("screenshot_viewport_{}.png", timestamp),
        };
        
        let screenshot_dir = self.workspace_root
            .join(".jag")
            .join("artifacts")
            .join(artifact_id)
            .join("screenshots");
        
        tokio::fs::create_dir_all(&screenshot_dir).await?;
        let screenshot_path = screenshot_dir.join(filename);

        let cmd = BrowserCommand {
            action: "screenshot".to_string(),
            url: Some(url.to_string()),
            selector: selector.map(|s| s.to_string()),
            path: Some(screenshot_path.to_string_lossy().to_string()),
            timeout_ms: Some(self.config.navigation_timeout_ms),
        };

        let resp = self.run_command(cmd).await?;
        if !resp.success {
            anyhow::bail!("Playwright capture failed: {:?}", resp.error);
        }

        Ok(screenshot_path)
    }

    /// Compares two screenshots and returns a similarity percentage (0.0 to 100.0).
    pub fn compare_screenshots(base: &Path, current: &Path) -> Result<f32> {
        let img1 = image::open(base).context("Failed to open base image")?;
        let img2 = image::open(current).context("Failed to open current image")?;
        
        if img1.dimensions() != img2.dimensions() {
            warn!("Screenshot dimensions mismatch: {:?} vs {:?}", img1.dimensions(), img2.dimensions());
            return Ok(0.0);
        }
        
        let mut diff_count = 0;
        let (width, height) = img1.dimensions();
        let total_pixels = width * height;
        
        for (x, y, p1) in img1.pixels() {
            let p2 = img2.get_pixel(x, y);
            if p1 != p2 {
                diff_count += 1;
            }
        }
        
        let match_percentage = (1.0 - (diff_count as f32 / total_pixels as f32)) * 100.0;
        info!("Visual match: {:.2}%", match_percentage);
        Ok(match_percentage)
    }
}
