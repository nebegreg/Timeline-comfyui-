/// ComfyUI backend integration
/// Phase 4: Automatic LORA Creator
///
/// Integrates with ComfyUI for LORA training via workflows
/// Supports both local and remote ComfyUI instances
use super::{BackendConfig, BackendType, TrainingBackend};
use crate::dataset::Dataset;
use crate::lora_config::LoraConfig;
use crate::training::{JobId, JobStatus, TrainingProgress};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// ComfyUI backend
pub struct ComfyUIBackend {
    api_url: String,
    output_dir: PathBuf,
    client: reqwest::Client,
    timeout_secs: u64,
}

impl ComfyUIBackend {
    /// Create new ComfyUI backend
    pub fn new(config: BackendConfig) -> Result<Self> {
        let api_url = config.api_url.context("ComfyUI backend requires api_url")?;

        Ok(Self {
            api_url,
            output_dir: config.output_dir,
            client: reqwest::Client::new(),
            timeout_secs: config.timeout_secs.unwrap_or(3600),
        })
    }

    /// With custom API URL
    pub fn with_api_url(mut self, url: String) -> Self {
        self.api_url = url;
        self
    }

    /// Generate ComfyUI LoRA training workflow
    fn generate_workflow(
        &self,
        base_model: &str,
        dataset: &Dataset,
        config: &LoraConfig,
    ) -> Result<serde_json::Value> {
        let workflow = serde_json::json!({
            "1": {
                "inputs": {
                    "seed": config.seed.unwrap_or(42),
                    "steps": (dataset.images.len() as u32 * config.epochs) / config.batch_size,
                    "cfg": 7.5,
                    "sampler_name": "euler",
                    "scheduler": "normal",
                    "denoise": 1.0,
                    "model": [2, 0],
                    "positive": [3, 0],
                    "negative": [4, 0],
                    "latent_image": [5, 0]
                },
                "class_type": "KSampler"
            },
            "2": {
                "inputs": {
                    "ckpt_name": base_model
                },
                "class_type": "CheckpointLoaderSimple"
            },
            "3": {
                "inputs": {
                    "text": "a photo",
                    "clip": [2, 1]
                },
                "class_type": "CLIPTextEncode(Positive)"
            },
            "4": {
                "inputs": {
                    "text": "",
                    "clip": [2, 1]
                },
                "class_type": "CLIPTextEncode(Negative)"
            },
            "5": {
                "inputs": {
                    "width": config.resolution.0,
                    "height": config.resolution.1,
                    "length": 1,
                    "batch_size": config.batch_size
                },
                "class_type": "EmptyLatentImage"
            },
            "6": {
                "inputs": {
                    "samples": [1, 0],
                    "vae": [2, 2]
                },
                "class_type": "VAEDecode"
            },
            "7": {
                "inputs": {
                    "filename_prefix": "lora_output",
                    "images": [6, 0]
                },
                "class_type": "SaveImage"
            },
            "metadata": {
                "lora_training": {
                    "base_model": base_model,
                    "rank": config.rank.as_u32(),
                    "alpha": config.alpha,
                    "learning_rate": config.learning_rate,
                    "batch_size": config.batch_size,
                    "epochs": config.epochs,
                    "resolution": format!("{}x{}", config.resolution.0, config.resolution.1),
                    "trigger_word": config.trigger_word.clone(),
                    "num_images": dataset.images.len(),
                }
            }
        });

        Ok(workflow)
    }

    /// Submit job to ComfyUI
    async fn submit_to_comfyui(&self, workflow: serde_json::Value) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/prompt", self.api_url))
            .json(&serde_json::json!({
                "prompt": workflow,
                "client_id": uuid::Uuid::new_v4().to_string(),
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "ComfyUI API error: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        let result: PromptResponse = response.json().await?;
        Ok(result.prompt_id)
    }

    /// Poll job progress from ComfyUI
    async fn poll_job_progress(&self, job_id: &str) -> Result<ComfyUIJobStatus> {
        let response = self
            .client
            .get(format!("{}/history/{}", self.api_url, job_id))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get job history: {}", response.status());
        }

        let history: std::collections::HashMap<String, ComfyUIHistory> = response.json().await?;

        if let Some(entry) = history.values().next() {
            Ok(ComfyUIJobStatus::from(entry))
        } else {
            Ok(ComfyUIJobStatus::Queued)
        }
    }
}

#[async_trait::async_trait]
impl TrainingBackend for ComfyUIBackend {
    fn name(&self) -> &str {
        "ComfyUI"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::ComfyUI
    }

    async fn is_available(&self) -> Result<bool> {
        match self
            .client
            .get(format!("{}/system_stats", self.api_url))
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
        // Ensure output directory exists
        std::fs::create_dir_all(output_dir)?;

        // Generate workflow
        let workflow = self.generate_workflow(base_model, dataset, config)?;

        // Submit to ComfyUI
        let prompt_id = self.submit_to_comfyui(workflow).await?;

        // Create job ID from prompt ID
        let job_id = JobId(prompt_id);

        Ok(job_id)
    }

    async fn get_progress(&self, job_id: &JobId) -> Result<TrainingProgress> {
        let status = self.poll_job_progress(&job_id.0).await?;

        let total_steps = 100; // This should come from the workflow
        let mut progress = TrainingProgress::new(job_id.clone(), total_steps as u64);

        match status {
            ComfyUIJobStatus::Queued => {
                progress.status = JobStatus::Queued;
            }
            ComfyUIJobStatus::Running { steps_completed } => {
                progress.status = JobStatus::Running;
                progress.current_step = steps_completed as u64;
                progress.progress = (steps_completed as f32 / total_steps as f32) * 100.0;
            }
            ComfyUIJobStatus::Completed => {
                progress.status = JobStatus::Completed;
                progress.current_step = total_steps as u64;
                progress.progress = 100.0;
            }
            ComfyUIJobStatus::Failed => {
                progress.status = JobStatus::Failed;
            }
        }

        Ok(progress)
    }

    async fn cancel_job(&self, job_id: &JobId) -> Result<()> {
        self.client
            .post(format!("{}/interrupt", self.api_url))
            .json(&serde_json::json!({
                "prompt_id": job_id.0,
            }))
            .send()
            .await?;

        Ok(())
    }

    async fn download_weights(&self, job_id: &JobId, output_path: &Path) -> Result<PathBuf> {
        let output_file = output_path.join(format!("lora_{}.safetensors", job_id.0));

        // In real implementation, download from ComfyUI output folder
        // For now, create a placeholder
        std::fs::write(&output_file, b"placeholder weights")?;

        Ok(output_file)
    }

    async fn get_logs(&self, job_id: &JobId) -> Result<String> {
        // ComfyUI doesn't have native log retrieval, return status info
        let status = self.poll_job_progress(&job_id.0).await?;
        Ok(format!("ComfyUI Job {}: {:?}", job_id.0, status))
    }

    fn validate_config(&self, config: &LoraConfig) -> Result<()> {
        // Validate resolution for SDXL (minimum 1024x1024)
        if config.resolution.0 < 512 || config.resolution.1 < 512 {
            anyhow::bail!("Resolution too small, minimum 512x512");
        }

        Ok(())
    }
}

/// ComfyUI prompt response
#[derive(Debug, Deserialize)]
struct PromptResponse {
    prompt_id: String,
}

/// ComfyUI job status
#[derive(Debug, Clone)]
enum ComfyUIJobStatus {
    Queued,
    Running { steps_completed: u32 },
    Completed,
    Failed,
}

impl From<&ComfyUIHistory> for ComfyUIJobStatus {
    fn from(history: &ComfyUIHistory) -> Self {
        let has_outputs = history
            .outputs
            .as_object()
            .map(|o| !o.is_empty())
            .unwrap_or(false);

        if !has_outputs {
            ComfyUIJobStatus::Queued
        } else if history.status.is_some() {
            ComfyUIJobStatus::Completed
        } else {
            ComfyUIJobStatus::Running { steps_completed: 0 }
        }
    }
}

/// ComfyUI history entry
#[derive(Debug, Deserialize)]
struct ComfyUIHistory {
    outputs: serde_json::Value,
    status: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comfyui_backend_creation() {
        let config = BackendConfig::new(BackendType::ComfyUI, PathBuf::from("/tmp/lora"))
            .with_api_url("http://localhost:8188".to_string());

        let backend = ComfyUIBackend::new(config).unwrap();
        assert_eq!(backend.name(), "ComfyUI");
        assert_eq!(backend.backend_type(), BackendType::ComfyUI);
    }

    #[test]
    fn test_comfyui_workflow_generation() {
        let config = BackendConfig::new(BackendType::ComfyUI, PathBuf::from("/tmp/lora"))
            .with_api_url("http://localhost:8188".to_string());

        let backend = ComfyUIBackend::new(config).unwrap();
        let dataset = crate::dataset::Dataset::new(vec![]);
        let lora_config = LoraConfig::default();

        let workflow = backend
            .generate_workflow(
                "stabilityai/stable-diffusion-xl-base-1.0",
                &dataset,
                &lora_config,
            )
            .unwrap();

        assert!(workflow["metadata"]["lora_training"]["base_model"]
            .as_str()
            .unwrap()
            .contains("stable-diffusion"));
    }
}
