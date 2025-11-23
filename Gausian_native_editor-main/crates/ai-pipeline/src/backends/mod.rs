/// Training backends abstraction
/// Phase 4: Automatic LORA Creator
///
/// Provides unified interface for different training backends:
/// - ComfyUI (cloud/local)
/// - Replicate API
/// - Local Candle runtime (experimental)

pub mod comfyui;
pub mod replicate;

use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub use comfyui::ComfyUIBackend;
pub use replicate::ReplicateBackend;

use crate::dataset::Dataset;
use crate::lora_config::LoraConfig;
use crate::training::{JobId, TrainingProgress};

/// Backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    /// ComfyUI workflow backend
    ComfyUI,
    /// Replicate API backend
    Replicate,
    /// Local Candle training (experimental)
    Candle,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComfyUI => write!(f, "comfyui"),
            Self::Replicate => write!(f, "replicate"),
            Self::Candle => write!(f, "candle"),
        }
    }
}

/// Training backend trait
#[async_trait::async_trait]
pub trait TrainingBackend: Send + Sync {
    /// Backend name
    fn name(&self) -> &str;

    /// Backend type
    fn backend_type(&self) -> BackendType;

    /// Check if backend is available/configured
    async fn is_available(&self) -> Result<bool>;

    /// Submit training job
    async fn submit_job(
        &self,
        base_model: &str,
        dataset: &Dataset,
        config: &LoraConfig,
        output_dir: &Path,
    ) -> Result<JobId>;

    /// Get job progress
    async fn get_progress(&self, job_id: &JobId) -> Result<TrainingProgress>;

    /// Cancel job
    async fn cancel_job(&self, job_id: &JobId) -> Result<()>;

    /// Download trained weights
    async fn download_weights(&self, job_id: &JobId, output_path: &Path) -> Result<PathBuf>;

    /// Get job logs
    async fn get_logs(&self, job_id: &JobId) -> Result<String>;

    /// Estimate training cost (if applicable)
    async fn estimate_cost(
        &self,
        _base_model: &str,
        _dataset: &Dataset,
        _config: &LoraConfig,
    ) -> Result<Option<f64>> {
        Ok(None) // Most backends return None
    }

    /// Validate configuration for this backend
    fn validate_config(&self, _config: &LoraConfig) -> Result<()> {
        // Default: no validation
        Ok(())
    }
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend type
    pub backend_type: BackendType,

    /// API endpoint URL (for cloud backends)
    pub api_url: Option<String>,

    /// API key or token
    pub api_key: Option<String>,

    /// Local output directory
    pub output_dir: PathBuf,

    /// Maximum concurrent jobs
    pub max_concurrent_jobs: Option<u32>,

    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
}

impl BackendConfig {
    /// Create new backend config
    pub fn new(backend_type: BackendType, output_dir: PathBuf) -> Self {
        Self {
            backend_type,
            api_url: None,
            api_key: None,
            output_dir,
            max_concurrent_jobs: None,
            timeout_secs: Some(3600), // 1 hour default
        }
    }

    /// With API endpoint
    pub fn with_api_url(mut self, url: String) -> Self {
        self.api_url = Some(url);
        self
    }

    /// With API key
    pub fn with_api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    /// With max concurrent jobs
    pub fn with_max_jobs(mut self, max: u32) -> Self {
        self.max_concurrent_jobs = Some(max);
        self
    }

    /// With timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Save configuration to JSON
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load configuration from JSON
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&json)?;
        Ok(config)
    }
}

/// Backend factory for creating backend instances
pub struct BackendFactory;

impl BackendFactory {
    /// Create backend from config
    pub async fn create(config: BackendConfig) -> Result<Box<dyn TrainingBackend>> {
        match config.backend_type {
            BackendType::ComfyUI => {
                let backend = ComfyUIBackend::new(config)?;
                Ok(Box::new(backend))
            }
            BackendType::Replicate => {
                let backend = ReplicateBackend::new(config)?;
                Ok(Box::new(backend))
            }
            BackendType::Candle => {
                anyhow::bail!("Candle backend not yet implemented")
            }
        }
    }

    /// Create default ComfyUI backend
    pub async fn comfyui(api_url: String, output_dir: PathBuf) -> Result<ComfyUIBackend> {
        let config = BackendConfig::new(BackendType::ComfyUI, output_dir)
            .with_api_url(api_url);
        ComfyUIBackend::new(config)
    }

    /// Create default Replicate backend
    pub async fn replicate(api_key: String, output_dir: PathBuf) -> Result<ReplicateBackend> {
        let config = BackendConfig::new(BackendType::Replicate, output_dir)
            .with_api_key(api_key);
        ReplicateBackend::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_config() {
        let config = BackendConfig::new(
            BackendType::ComfyUI,
            PathBuf::from("/tmp/lora"),
        )
        .with_api_url("http://localhost:8188".to_string())
        .with_max_jobs(5);

        assert_eq!(config.backend_type, BackendType::ComfyUI);
        assert_eq!(config.api_url, Some("http://localhost:8188".to_string()));
        assert_eq!(config.max_concurrent_jobs, Some(5));
    }

    #[test]
    fn test_backend_type_display() {
        assert_eq!(BackendType::ComfyUI.to_string(), "comfyui");
        assert_eq!(BackendType::Replicate.to_string(), "replicate");
        assert_eq!(BackendType::Candle.to_string(), "candle");
    }
}
