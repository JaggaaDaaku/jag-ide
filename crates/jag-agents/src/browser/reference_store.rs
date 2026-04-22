use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use jag_core::errors::{JagError, Result};
use jag_core::types::ViewportSpec;
use image::DynamicImage;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferenceMetadata {
    pub artifact_id: String,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub phash: String,
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ReferenceStore {
    base_dir: PathBuf,
}

impl ReferenceStore {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Save a "golden" reference screenshot and its perceptual hash.
    pub async fn save_reference(
        &self,
        artifact_id: &str,
        viewport: &ViewportSpec,
        image_data: &[u8],
    ) -> Result<()> {
        let dir = self.base_dir.join(artifact_id);
        fs::create_dir_all(&dir).map_err(JagError::Io)?;

        let viewport_name = format!("{}x{}", viewport.width, viewport.height);
        let image_path = dir.join(format!("{}.png", viewport_name));
        let meta_path = dir.join(format!("{}.json", viewport_name));

        // Compute pHash
        let img = image::load_from_memory(image_data)
            .map_err(|e| JagError::InvalidInput(format!("Failed to decode image for phash: {}", e)))?;
        let phash = self.compute_phash(&img);

        let metadata = ReferenceMetadata {
            artifact_id: artifact_id.to_string(),
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            phash: phash.to_string(),
            captured_at: chrono::Utc::now(),
        };

        // Save files
        fs::write(&image_path, image_data).map_err(JagError::Io)?;
        let meta_json = serde_json::to_string_pretty(&metadata)
            .map_err(JagError::Serialization)?;
        fs::write(&meta_path, meta_json).map_err(JagError::Io)?;

        info!(path = ?image_path, "Reference screenshot saved");
        Ok(())
    }

    /// Load a reference screenshot and its metadata.
    pub async fn load_reference(
        &self,
        artifact_id: &str,
        viewport: &ViewportSpec,
    ) -> Result<Option<(Vec<u8>, ReferenceMetadata)>> {
        let viewport_name = format!("{}x{}", viewport.width, viewport.height);
        let dir = self.base_dir.join(artifact_id);
        let image_path = dir.join(format!("{}.png", viewport_name));
        let meta_path = dir.join(format!("{}.json", viewport_name));

        if !image_path.exists() || !meta_path.exists() {
            return Ok(None);
        }

        let image_data = fs::read(&image_path).map_err(JagError::Io)?;
        let meta_json = fs::read_to_string(&meta_path).map_err(JagError::Io)?;
        let metadata: ReferenceMetadata = serde_json::from_str(&meta_json)
            .map_err(JagError::Serialization)?;

        Ok(Some((image_data, metadata)))
    }

    /// Compute Hamming distance similarity (0.0 - 1.0).
    pub fn compute_similarity(hash1_str: &str, hash2_str: &str) -> Result<f32> {
        let hash1 = hex::decode(hash1_str).map_err(|_| JagError::InvalidInput("Invalid hash1 hex".into()))?;
        let hash2 = hex::decode(hash2_str).map_err(|_| JagError::InvalidInput("Invalid hash2 hex".into()))?;

        if hash1.len() != hash2.len() {
            return Err(JagError::InvalidInput("Hash length mismatch".into()));
        }

        let mut diff_bits = 0;
        for (b1, b2) in hash1.iter().zip(hash2.iter()) {
            diff_bits += (b1 ^ b2).count_ones();
        }

        let total_bits = (hash1.len() * 8) as f32;
        let distance = diff_bits as f32 / total_bits;
        
        // Similarity is 1.0 - distance
        Ok(1.0 - distance)
    }

    /// Internal helper to compute perceptual hash.
    /// Uses a custom Mean Hash implementation to avoid version conflicts with the img_hash crate.
    pub(crate) fn compute_phash(&self, img: &DynamicImage) -> String {
        
        
        // 1. Resize to 8x8, grayscale
        let small = img.resize_exact(8, 8, image::imageops::FilterType::Nearest).to_luma8();
        
        // 2. Calculate mean
        let pixels = small.as_raw();
        let sum: u64 = pixels.iter().map(|&p| p as u64).sum();
        let mean = (sum / 64) as u8;
        
        // 3. Generate bits: 1 if pixel >= mean, else 0
        let mut hash_bytes = [0u8; 8];
        for i in 0..64 {
            if pixels[i] >= mean {
                hash_bytes[i / 8] |= 1 << (7 - (i % 8));
            }
        }
        
        hex::encode(hash_bytes)
    }
}
