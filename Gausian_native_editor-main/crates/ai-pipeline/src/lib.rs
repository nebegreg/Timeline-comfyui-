/// AI Pipeline for LORA Training
/// Phase 4: Automatic LORA Creator
///
/// Provides frame extraction, captioning, and LoRA training integration
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod backends;
pub mod captioning;
pub mod dataset;
pub mod lora_config;
pub mod training;

pub use backends::{BackendConfig, BackendFactory, BackendType, TrainingBackend};
pub use captioning::{Caption, CaptionProvider};
pub use dataset::{Dataset, DatasetBuilder, TrainingImage};
pub use lora_config::{LoraConfig, LoraRank};
pub use training::{JobId, JobStatus, TrainingJob, TrainingProgress};

/// Main LoRA creator interface
pub struct LoraCreator {
    /// Base model for LoRA training
    pub base_model: String,

    /// Training configuration
    pub config: LoraConfig,

    /// Dataset of images and captions
    pub dataset: Option<Dataset>,

    /// Training backend
    pub backend: Option<Box<dyn TrainingBackend>>,

    /// Job store
    pub job_store: training::JobStore,

    /// Output directory
    pub output_dir: PathBuf,
}

impl LoraCreator {
    /// Create new LoRA creator with default config
    pub fn new(base_model: String, output_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&output_dir)?;

        Ok(Self {
            base_model,
            config: LoraConfig::default(),
            dataset: None,
            backend: None,
            job_store: training::JobStore::new(),
            output_dir,
        })
    }

    /// Set training configuration
    pub fn with_config(mut self, config: LoraConfig) -> Self {
        self.config = config;
        self
    }

    /// Set training backend
    pub fn with_backend(mut self, backend: Box<dyn TrainingBackend>) -> Self {
        self.backend = Some(backend);
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

    /// Get dataset statistics
    pub fn dataset_stats(&self) -> Option<dataset::DatasetStats> {
        self.dataset.as_ref().map(|d| d.stats())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.dataset.is_none() {
            anyhow::bail!("No dataset loaded");
        }

        if self.backend.is_none() {
            anyhow::bail!("No training backend configured");
        }

        Ok(())
    }

    /// Prepare dataset for training
    pub async fn prepare_dataset(&mut self) -> Result<()> {
        if self.dataset.is_none() {
            anyhow::bail!("No dataset loaded");
        }

        let dataset = self.dataset.as_mut().unwrap();

        // Preprocess images
        let preprocess_dir = self.output_dir.join("preprocessed");
        dataset::preprocess::preprocess_dataset(dataset, self.config.resolution, &preprocess_dir)?;

        Ok(())
    }

    /// Train LoRA
    pub async fn train(&mut self) -> Result<TrainingJob> {
        self.validate()?;

        let dataset = self.dataset.as_ref().unwrap();
        let backend = self.backend.as_ref().unwrap();

        // Create training job
        let mut job = TrainingJob::new(
            format!("lora_{}", chrono::Utc::now().timestamp()),
            self.base_model.clone(),
            dataset.images.len(),
            (dataset.images.len() as u32 * self.config.epochs) as u64
                / self.config.batch_size as u64,
            self.output_dir.clone(),
            backend.name().to_string(),
        );

        // Submit job to backend
        let job_id = backend
            .submit_job(&self.base_model, dataset, &self.config, &self.output_dir)
            .await?;

        job.id = job_id;
        self.job_store.add_job(job.clone());

        Ok(job)
    }

    /// Monitor training progress
    pub async fn monitor_progress(&self, job_id: &JobId) -> Result<TrainingProgress> {
        if let Some(backend) = &self.backend {
            backend.get_progress(job_id).await
        } else {
            anyhow::bail!("No backend configured")
        }
    }

    /// Download trained weights
    pub async fn download_weights(&self, job_id: &JobId) -> Result<PathBuf> {
        if let Some(backend) = &self.backend {
            backend.download_weights(job_id, &self.output_dir).await
        } else {
            anyhow::bail!("No backend configured")
        }
    }

    /// Get job logs
    pub async fn get_logs(&self, job_id: &JobId) -> Result<String> {
        if let Some(backend) = &self.backend {
            backend.get_logs(job_id).await
        } else {
            anyhow::bail!("No backend configured")
        }
    }

    /// Cancel job
    pub async fn cancel_job(&self, job_id: &JobId) -> Result<()> {
        if let Some(backend) = &self.backend {
            backend.cancel_job(job_id).await
        } else {
            anyhow::bail!("No backend configured")
        }
    }

    /// Estimate training time
    pub fn estimate_training_time(&self) -> Result<f32> {
        if let Some(dataset) = &self.dataset {
            Ok(self.config.estimate_training_time(dataset.images.len()))
        } else {
            anyhow::bail!("No dataset loaded")
        }
    }

    /// Estimate VRAM requirements
    pub fn estimate_vram(&self) -> f32 {
        self.config.estimate_vram_gb()
    }

    /// Get list of running jobs
    pub fn list_running_jobs(&self) -> Vec<JobId> {
        self.job_store.list_jobs_by_status(JobStatus::Running)
    }

    /// Save configuration
    pub fn save_config(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load configuration
    pub fn load_config(&mut self, path: &Path) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        self.config = serde_json::from_str(&json)?;
        Ok(())
    }
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
    use std::path::PathBuf;

    #[test]
    fn test_lora_creator_creation() {
        let creator = LoraCreator::new(
            "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            PathBuf::from("/tmp/test_lora"),
        )
        .unwrap();
        assert_eq!(
            creator.base_model,
            "stabilityai/stable-diffusion-xl-base-1.0"
        );
        assert!(creator.dataset.is_none());
        assert!(creator.backend.is_none());
    }

    #[test]
    fn test_lora_creator_config() {
        let creator = LoraCreator::new(
            "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            PathBuf::from("/tmp/test_lora"),
        )
        .unwrap()
        .with_config(LoraConfig::sdxl_preset());

        assert_eq!(creator.config.rank, LoraRank::Rank32);
        assert_eq!(creator.config.resolution, (1024, 1024));
    }
}
