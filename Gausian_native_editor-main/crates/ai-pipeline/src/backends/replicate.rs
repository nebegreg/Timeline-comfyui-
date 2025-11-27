/// Replicate API backend integration
/// Phase 4: Automatic LORA Creator
///
/// Integrates with Replicate API for cloud-based LORA training
/// Supports models like kohya-ss's LoRA training
use super::{BackendConfig, BackendType, TrainingBackend};
use crate::dataset::Dataset;
use crate::lora_config::LoraConfig;
use crate::training::{JobId, JobStatus, TrainingProgress};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Replicate API backend
pub struct ReplicateBackend {
    api_key: String,
    output_dir: PathBuf,
    client: reqwest::Client,
}

impl ReplicateBackend {
    /// Create new Replicate backend
    pub fn new(config: BackendConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .context("Replicate backend requires api_key")?;

        Ok(Self {
            api_key,
            output_dir: config.output_dir,
            client: reqwest::Client::new(),
        })
    }

    /// Create prediction for LoRA training
    async fn create_prediction(
        &self,
        base_model: &str,
        dataset_path: &Path,
        num_images: usize,
        config: &LoraConfig,
    ) -> Result<String> {
        let request = PredictionRequest {
            version: LORA_TRAINER_VERSION.to_string(),
            input: TrainingInput {
                instance_prompt: config
                    .trigger_word
                    .clone()
                    .unwrap_or_else(|| "a photo".to_string()),
                model_name: base_model.to_string(),
                image_zip: format!("file://{}", dataset_path.display()),
                num_class_images: 100,
                num_max_training_steps: (num_images as u32 * config.epochs) as u32,
                learning_rate: config.learning_rate,
                rank: config.rank.as_u32(),
                lora_alpha: config.alpha as u32,
                resolution: format!("{}x{}", config.resolution.0, config.resolution.1),
                batch_size: config.batch_size,
                train_text_encoder: config.train_text_encoder,
            },
        };

        let response = self
            .client
            .post("https://api.replicate.com/v1/predictions")
            .header("Authorization", format!("Token {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Replicate API error: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        let result: PredictionResponse = response.json().await?;
        Ok(result.id)
    }

    /// Get prediction status
    async fn get_prediction(&self, prediction_id: &str) -> Result<PredictionResponse> {
        let response = self
            .client
            .get(format!(
                "https://api.replicate.com/v1/predictions/{}",
                prediction_id
            ))
            .header("Authorization", format!("Token {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to get prediction: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        Ok(response.json().await?)
    }

    /// Download weights from output URL
    async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        let bytes = response.bytes().await?;
        std::fs::write(dest, bytes)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl TrainingBackend for ReplicateBackend {
    fn name(&self) -> &str {
        "Replicate"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Replicate
    }

    async fn is_available(&self) -> Result<bool> {
        // Try a simple API call to verify credentials
        match self
            .client
            .get("https://api.replicate.com/v1/account")
            .header("Authorization", format!("Token {}", self.api_key))
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn submit_job(
        &self,
        base_model: &str,
        dataset: &Dataset,
        config: &LoraConfig,
        output_dir: &Path,
    ) -> Result<JobId> {
        // Create a temporary directory for dataset
        let temp_dir = output_dir.join("temp_dataset");
        std::fs::create_dir_all(&temp_dir)?;

        // Save dataset images to temporary directory
        for (idx, image) in dataset.images.iter().enumerate() {
            let src = &image.path;
            let dst = temp_dir.join(format!("image_{:06}.png", idx));
            std::fs::copy(src, dst)?;
        }

        // Create prediction with Replicate
        let prediction_id = self
            .create_prediction(base_model, &temp_dir, dataset.images.len(), config)
            .await?;

        Ok(JobId(prediction_id))
    }

    async fn get_progress(&self, job_id: &JobId) -> Result<TrainingProgress> {
        let prediction = self.get_prediction(&job_id.0).await?;

        let mut progress = TrainingProgress::new(job_id.clone(), 100);

        match prediction.status.as_str() {
            "starting" | "processing" => {
                progress.status = JobStatus::Running;
                // Parse progress from logs if available
                if let Some(metrics) = prediction.metrics {
                    progress.current_step = metrics.predict_time as u64;
                }
            }
            "succeeded" => {
                progress.status = JobStatus::Completed;
                progress.current_step = 100;
                progress.progress = 100.0;
            }
            "failed" | "canceled" => {
                progress.status = if prediction.status == "failed" {
                    JobStatus::Failed
                } else {
                    JobStatus::Cancelled
                };
            }
            _ => {
                progress.status = JobStatus::Queued;
            }
        }

        Ok(progress)
    }

    async fn cancel_job(&self, job_id: &JobId) -> Result<()> {
        self.client
            .post(format!(
                "https://api.replicate.com/v1/predictions/{}/cancel",
                job_id.0
            ))
            .header("Authorization", format!("Token {}", self.api_key))
            .send()
            .await?;

        Ok(())
    }

    async fn download_weights(&self, job_id: &JobId, output_path: &Path) -> Result<PathBuf> {
        let prediction = self.get_prediction(&job_id.0).await?;

        if prediction.status != "succeeded" {
            anyhow::bail!("Job not completed");
        }

        // Download from output URL
        if let Some(output) = prediction.output {
            if let Some(weights_url) = output.get(0).and_then(|v| v.as_str()) {
                let output_file = output_path.join(format!("lora_{}.safetensors", job_id.0));
                self.download_file(weights_url, &output_file).await?;
                return Ok(output_file);
            }
        }

        anyhow::bail!("No weights found in output")
    }

    async fn get_logs(&self, job_id: &JobId) -> Result<String> {
        let prediction = self.get_prediction(&job_id.0).await?;
        Ok(format!(
            "Replicate Job {}\nStatus: {}\nLogs:\n{}",
            job_id.0,
            prediction.status,
            prediction.logs.unwrap_or_default()
        ))
    }

    async fn estimate_cost(
        &self,
        _base_model: &str,
        dataset: &Dataset,
        config: &LoraConfig,
    ) -> Result<Option<f64>> {
        // Replicate charges per second
        let steps = (dataset.images.len() as f64 / config.batch_size as f64) * config.epochs as f64;
        let estimated_seconds = steps * 0.5; // Rough estimate: 0.5 seconds per step
        let cost_per_second = 0.00015; // Approximate cost per second

        Ok(Some(estimated_seconds * cost_per_second))
    }

    fn validate_config(&self, config: &LoraConfig) -> Result<()> {
        // Validate resolution
        if config.resolution.0 < 512 || config.resolution.1 < 512 {
            anyhow::bail!("Resolution too small, minimum 512x512");
        }

        if config.resolution.0 > 2048 || config.resolution.1 > 2048 {
            anyhow::bail!("Resolution too large, maximum 2048x2048");
        }

        Ok(())
    }
}

/// Replicate API model version for LoRA training
/// This is the kohya-ss LoRA trainer model on Replicate
const LORA_TRAINER_VERSION: &str = "7c7a0dda-bf78-4e6b-a0fa-fd0c5d6cf0f1"; // Example version ID

/// Prediction request for Replicate
#[derive(Debug, Serialize)]
struct PredictionRequest {
    version: String,
    input: TrainingInput,
}

/// Training input parameters
#[derive(Debug, Serialize)]
struct TrainingInput {
    instance_prompt: String,
    model_name: String,
    image_zip: String,
    num_class_images: u32,
    num_max_training_steps: u32,
    learning_rate: f32,
    rank: u32,
    lora_alpha: u32,
    resolution: String,
    batch_size: u32,
    train_text_encoder: bool,
}

/// Prediction response from Replicate
#[derive(Debug, Deserialize)]
struct PredictionResponse {
    id: String,
    status: String,
    output: Option<Vec<serde_json::Value>>,
    error: Option<String>,
    logs: Option<String>,
    metrics: Option<PredictionMetrics>,
}

/// Prediction metrics
#[derive(Debug, Deserialize)]
struct PredictionMetrics {
    predict_time: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replicate_backend_creation() {
        let config = BackendConfig::new(BackendType::Replicate, PathBuf::from("/tmp/lora"))
            .with_api_key("test-key-123".to_string());

        let backend = ReplicateBackend::new(config).unwrap();
        assert_eq!(backend.name(), "Replicate");
        assert_eq!(backend.backend_type(), BackendType::Replicate);
    }

    #[test]
    fn test_training_input_serialization() {
        let input = TrainingInput {
            instance_prompt: "myloraname, a photo".to_string(),
            model_name: "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            image_zip: "file:///tmp/dataset.zip".to_string(),
            num_class_images: 100,
            num_max_training_steps: 500,
            learning_rate: 1e-4,
            rank: 16,
            lora_alpha: 16,
            resolution: "1024x1024".to_string(),
            batch_size: 1,
            train_text_encoder: false,
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("myloraname"));
        assert!(json.contains("stable-diffusion-xl"));
    }
}
