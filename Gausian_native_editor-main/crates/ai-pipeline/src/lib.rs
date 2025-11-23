/// AI Pipeline for LORA Training
/// Phase 4: Automatic LORA Creator
///
/// Provides frame extraction, captioning, and LoRA training integration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod dataset;
pub mod lora_config;
// TODO: Implement these modules
// pub mod training;
pub mod captioning;
// pub mod backends;
// pub mod models;

pub use dataset::{Dataset, DatasetBuilder, TrainingImage};
pub use lora_config::{LoraConfig, LoraRank};
// pub use training::{TrainingJob, TrainingProgress, TrainingBackend};
pub use captioning::{CaptionProvider, Caption};

/// Main LoRA creator interface (simplified - backend implementation pending)
pub struct LoraCreator {
    /// Base model for LoRA training
    pub base_model: String,

    /// Training configuration
    pub config: LoraConfig,

    /// Dataset of images and captions
    pub dataset: Option<Dataset>,
}

impl LoraCreator {
    /// Create new LoRA creator with default config
    pub fn new(base_model: String) -> Self {
        Self {
            base_model,
            config: LoraConfig::default(),
            dataset: None,
        }
    }

    /// Set training configuration
    pub fn with_config(mut self, config: LoraConfig) -> Self {
        self.config = config;
        self
    }

    /// Load dataset from directory
    pub fn load_dataset(&mut self, path: &Path) -> Result<()> {
        self.dataset = Some(Dataset::from_directory(path)?);
        Ok(())
    }

    /// Build dataset from images
    pub fn build_dataset(&mut self, images: Vec<TrainingImage>) -> Result<()> {
        self.dataset = Some(Dataset::new(images));
        Ok(())
    }

    // TODO: Implement training methods once backend is ready
    // pub async fn train(&self) -> Result<TrainingJob>
    // pub async fn monitor_progress(&self, job: &TrainingJob) -> Result<TrainingProgress>
    // pub async fn download_weights(&self, job: &TrainingJob, output_path: &Path) -> Result<PathBuf>
}

/// LoRA weights metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraWeights {
    /// Path to weights file (.safetensors)
    pub path: PathBuf,

    /// Base model used for training
    pub base_model: String,

    /// LoRA configuration
    pub config: LoraConfig,

    /// Training statistics
    pub stats: TrainingStats,

    /// Creation timestamp
    pub created_at: i64,
}

/// Training statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    /// Total training steps
    pub total_steps: u64,

    /// Final loss value
    pub final_loss: f32,

    /// Training duration in seconds
    pub duration_secs: u64,

    /// Number of training images
    pub num_images: usize,

    /// Number of epochs
    pub epochs: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_creator_creation() {
        let creator = LoraCreator::new("stabilityai/stable-diffusion-xl-base-1.0".to_string());
        assert_eq!(creator.base_model, "stabilityai/stable-diffusion-xl-base-1.0");
        assert!(creator.dataset.is_none());
    }
}
