# AI Pipeline Integration Guide

Complete guide for integrating the AI Pipeline LoRA Creator into the Gaussian Native Editor timeline system.

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│         Gaussian Native Editor                  │
│  ┌───────────────────────────────────────────┐  │
│  │        Timeline System                    │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  Video/Frame Sequence               │  │  │
│  │  │  - Timeline clips                   │  │  │
│  │  │  - Keyframes                        │  │  │
│  │  │  - Effects/Transitions              │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│              ↓ (Frame Extraction)               │
│  ┌───────────────────────────────────────────┐  │
│  │   AI Pipeline - LORA Creator              │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  Dataset Management                 │  │  │
│  │  │  - Frame preprocessing              │  │  │
│  │  │  - Automatic captioning             │  │  │
│  │  │  - Dataset statistics               │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  LoRA Training                      │  │  │
│  │  │  - Configuration management         │  │  │
│  │  │  - Multi-backend support            │  │  │
│  │  │  - Job tracking                     │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  Training Backends                  │  │  │
│  │  │  - ComfyUI (local/remote)           │  │  │
│  │  │  - Replicate API (cloud)            │  │  │
│  │  │  - Candle (experimental)            │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│              ↓ (Trained LoRA Weights)           │
│  ┌───────────────────────────────────────────┐  │
│  │        Effects/Filters System             │  │
│  │  - Apply trained LoRA to timeline         │  │
│  │  - Generate variations                    │  │
│  │  - Batch processing                       │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Integration Steps

### 1. Frame Extraction from Timeline

Extract frames from a timeline sequence:

```rust
use ai_pipeline::dataset::DatasetBuilder;
use timeline::Timeline;  // Your timeline module

async fn extract_timeline_frames(
    timeline: &Timeline,
    frame_interval: u32,  // Every N frames
) -> Result<PathBuf> {
    let dataset_dir = PathBuf::from("./timeline_lora_dataset");
    let mut builder = DatasetBuilder::new(dataset_dir.clone())?;

    // Get timeline clips
    for clip in timeline.clips() {
        // Extract frames at interval
        for (frame_num, frame_data) in clip.frames().enumerate() {
            if frame_num % frame_interval as usize == 0 {
                let img = frame_data.to_dynamic_image()?;

                // Generate caption from clip metadata
                let caption = clip.get_description()
                    .unwrap_or_else(|| "a frame from video".to_string());

                builder.add_image(img, caption)?;
            }
        }
    }

    let dataset = builder.build();
    Ok(dataset_dir)
}
```

### 2. Automatic Captioning Integration

Set up caption providers:

```rust
use ai_pipeline::captioning::{CaptionProvider, Blip2ApiProvider};

async fn setup_captioning() -> Result<Box<dyn CaptionProvider>> {
    // Option 1: BLIP2 via HuggingFace
    if let Ok(hf_token) = std::env::var("HUGGINGFACE_TOKEN") {
        return Ok(Box::new(Blip2ApiProvider::huggingface(hf_token)));
    }

    // Option 2: Local LLaVA
    if let Ok(_) = std::env::var("LLAVA_API_URL") {
        let url = std::env::var("LLAVA_API_URL")?;
        return Ok(Box::new(ai_pipeline::captioning::LlavaApiProvider::new(
            url,
            None,
        )));
    }

    // Fallback: Simple captions
    Ok(Box::new(ai_pipeline::captioning::SimpleCaptioner::new(
        "a photo from video".to_string(),
    )))
}

async fn auto_caption_dataset(
    dataset_dir: &Path,
    captioner: &dyn CaptionProvider,
) -> Result<()> {
    let dataset = ai_pipeline::Dataset::from_directory(dataset_dir)?;

    for image in &dataset.images {
        let img = image.load_image()?;
        let caption = captioner.caption(&img).await?;

        // Save caption
        let caption_path = image.path.with_extension("txt");
        std::fs::write(&caption_path, caption.text)?;
    }

    Ok(())
}
```

### 3. UI Integration

Add LoRA creation panel to the Effects timeline:

```rust
// In your effects UI module
pub struct LoraCreationPanel {
    creator: Option<LoraCreator>,
    dataset_path: Option<PathBuf>,
    config: LoraConfig,
    current_job: Option<JobId>,
}

impl LoraCreationPanel {
    pub fn new() -> Result<Self> {
        Ok(Self {
            creator: None,
            dataset_path: None,
            config: LoraConfig::default(),
            current_job: None,
        })
    }

    pub async fn initialize(&mut self, output_dir: PathBuf) -> Result<()> {
        let mut creator = LoraCreator::new(
            "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            output_dir,
        )?;

        // Setup default backend
        let config = BackendConfig::new(
            BackendType::ComfyUI,
            creator.output_dir.clone(),
        )
        .with_api_url("http://localhost:8188".to_string());

        if let Ok(backend) = ComfyUIBackend::new(config) {
            creator = creator.with_backend(Box::new(backend));
        }

        self.creator = Some(creator);
        Ok(())
    }

    pub async fn start_training(&mut self) -> Result<()> {
        if let Some(creator) = &mut self.creator {
            if let Some(dataset_path) = &self.dataset_path {
                creator.load_dataset(dataset_path)?;
                creator = creator.with_config(self.config.clone());

                let job = creator.train().await?;
                self.current_job = Some(job.id);
            }
        }
        Ok(())
    }

    pub async fn update_progress(&self) -> Result<Option<TrainingProgress>> {
        if let (Some(creator), Some(job_id)) = (&self.creator, &self.current_job) {
            return Ok(Some(creator.monitor_progress(&job_id).await?));
        }
        Ok(None)
    }
}
```

### 4. Environment Configuration

Create `.env.local` for your development environment:

```env
# ComfyUI Backend
COMFYUI_API_URL=http://localhost:8188

# Replicate Backend (if using cloud training)
REPLICATE_API_TOKEN=r8_xxxxxxxxxxxxxxxxxxxxxxxxxxxx

# HuggingFace API (for image captioning)
HUGGINGFACE_TOKEN=hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Local LLaVA (optional)
LLAVA_API_URL=http://localhost:8000

# LoRA Training Output
LORA_OUTPUT_DIR=./data/lora_models
DATASET_OUTPUT_DIR=./data/datasets
```

### 5. State Management

Integrate with your application state:

```rust
pub struct AppState {
    // ... existing state ...
    lora_creator: Option<LoraCreator>,
    training_jobs: Vec<TrainingJob>,
    job_progress: HashMap<JobId, TrainingProgress>,
}

impl AppState {
    pub async fn initialize_lora_pipeline(&mut self) -> Result<()> {
        let mut creator = LoraCreator::new(
            "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            PathBuf::from(
                std::env::var("LORA_OUTPUT_DIR")
                    .unwrap_or_else(|_| "./lora_output".to_string())
            ),
        )?;

        // Configure backend
        let api_url = std::env::var("COMFYUI_API_URL")
            .unwrap_or_else(|_| "http://localhost:8188".to_string());

        let backend_config = BackendConfig::new(
            BackendType::ComfyUI,
            creator.output_dir.clone(),
        )
        .with_api_url(api_url);

        if let Ok(backend) = ComfyUIBackend::new(backend_config) {
            creator = creator.with_backend(Box::new(backend));
        }

        self.lora_creator = Some(creator);
        Ok(())
    }

    pub async fn process_training_job(
        &mut self,
        job_id: &JobId,
    ) -> Result<()> {
        if let Some(creator) = &self.lora_creator {
            let progress = creator.monitor_progress(job_id).await?;
            self.job_progress.insert(job_id.clone(), progress);
        }
        Ok(())
    }
}
```

### 6. Timeline Integration Points

#### Extracting Frames from Specific Segments

```rust
pub async fn extract_segment_frames(
    timeline: &Timeline,
    segment_id: &str,
    output_dir: &Path,
) -> Result<()> {
    let mut builder = DatasetBuilder::new(output_dir.to_path_buf())?;

    if let Some(segment) = timeline.get_segment(segment_id) {
        // Extract every N frames from segment
        let frame_interval = 10;

        for (idx, frame) in segment.frames().enumerate() {
            if idx % frame_interval == 0 {
                let img = frame.to_image()?;
                let caption = format!(
                    "Frame {} from segment {}",
                    idx,
                    segment.name()
                );

                builder.add_image(img, caption)?;
            }
        }
    }

    let dataset = builder.build();
    dataset.save_manifest(&output_dir.join("manifest.json"))?;

    Ok(())
}
```

#### Applying Trained LoRA to Timeline

```rust
pub async fn apply_lora_to_timeline(
    timeline: &mut Timeline,
    lora_weights_path: &Path,
) -> Result<()> {
    // Load the trained LoRA weights
    let lora_data = std::fs::read(lora_weights_path)?;

    // Apply to timeline using ComfyUI workflow
    // This would typically involve:
    // 1. Creating a ComfyUI workflow that applies the LoRA
    // 2. Processing timeline frames through the workflow
    // 3. Replacing original frames with processed ones

    Ok(())
}
```

## Configuration Examples

### Local ComfyUI Training

```rust
async fn setup_local_training() -> Result<LoraCreator> {
    let mut creator = LoraCreator::new(
        "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
        PathBuf::from("./lora_training"),
    )?;

    let backend_config = BackendConfig::new(
        BackendType::ComfyUI,
        PathBuf::from("./lora_training"),
    )
    .with_api_url("http://localhost:8188".to_string())
    .with_max_jobs(1);

    let backend = ComfyUIBackend::new(backend_config)?;
    creator = creator.with_backend(Box::new(backend));

    Ok(creator)
}
```

### Cloud-Based Replicate Training

```rust
async fn setup_cloud_training() -> Result<LoraCreator> {
    let api_key = std::env::var("REPLICATE_API_TOKEN")?;

    let mut creator = LoraCreator::new(
        "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
        PathBuf::from("./lora_training"),
    )?;

    let backend_config = BackendConfig::new(
        BackendType::Replicate,
        PathBuf::from("./lora_training"),
    )
    .with_api_key(api_key)
    .with_timeout(7200); // 2 hours

    let backend = ReplicateBackend::new(backend_config)?;
    creator = creator.with_backend(Box::new(backend));

    Ok(creator)
}
```

## Performance Optimization

### Memory-Efficient Settings

```rust
pub fn get_efficient_config() -> LoraConfig {
    LoraConfig {
        rank: LoraRank::Rank8,
        alpha: 8.0,
        learning_rate: 2e-4,
        batch_size: 1,
        epochs: 5,
        resolution: (512, 512),
        train_text_encoder: false,
        gradient_accumulation_steps: 8,
        mixed_precision: MixedPrecision::Fp16,
        use_8bit_adam: true,
        ..Default::default()
    }
}
```

### High-Quality Settings

```rust
pub fn get_quality_config() -> LoraConfig {
    LoraConfig {
        rank: LoraRank::Rank32,
        alpha: 32.0,
        learning_rate: 1e-4,
        batch_size: 2,
        epochs: 15,
        resolution: (1024, 1024),
        train_text_encoder: true,
        gradient_accumulation_steps: 2,
        mixed_precision: MixedPrecision::Bf16,
        lr_scheduler: LrScheduler::Cosine,
        warmup_steps: 100,
        ..Default::default()
    }
}
```

## Monitoring and Logging

```rust
pub async fn monitor_training_job(
    creator: &LoraCreator,
    job_id: &JobId,
) -> Result<()> {
    loop {
        let progress = creator.monitor_progress(job_id).await?;

        println!(
            "[{:.0}%] Step {}/{} - Loss: {:?}",
            progress.progress,
            progress.current_step,
            progress.total_steps,
            progress.current_loss
        );

        if progress.is_finished() {
            match progress.status {
                JobStatus::Completed => println!("Training completed!"),
                JobStatus::Failed => println!("Training failed!"),
                JobStatus::Cancelled => println!("Training cancelled!"),
                _ => {}
            }
            break;
        }

        // Get detailed logs if available
        if let Ok(logs) = creator.get_logs(job_id).await {
            println!("Job logs:\n{}", logs);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }

    Ok(())
}
```

## Testing Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lora_pipeline_setup() -> Result<()> {
        let creator = LoraCreator::new(
            "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            PathBuf::from("./test_output"),
        )?;

        assert_eq!(creator.base_model, "stabilityai/stable-diffusion-xl-base-1.0");
        Ok(())
    }

    #[test]
    fn test_dataset_creation() {
        let mut builder = DatasetBuilder::new(PathBuf::from("./test")).unwrap();
        assert_eq!(builder.len(), 0);
    }
}
```

## Troubleshooting

### ComfyUI Not Available

```rust
// Check ComfyUI availability
async fn check_backend_availability() -> Result<()> {
    let config = BackendConfig::new(
        BackendType::ComfyUI,
        PathBuf::from("./lora"),
    )
    .with_api_url("http://localhost:8188".to_string());

    let backend = ComfyUIBackend::new(config)?;

    if backend.is_available().await? {
        println!("✓ ComfyUI is available");
    } else {
        eprintln!("✗ ComfyUI not responding at http://localhost:8188");
        eprintln!("  Make sure ComfyUI is running and accessible");
    }

    Ok(())
}
```

### Dataset Issues

```rust
// Validate dataset before training
fn validate_dataset(dataset: &Dataset) -> Result<()> {
    let stats = dataset.stats();

    if stats.total_images < 10 {
        anyhow::bail!("Dataset too small (minimum 10 images)");
    }

    if stats.images_with_captions == 0 {
        anyhow::bail!("No images have captions");
    }

    let coverage = (stats.images_with_captions as f32 / stats.total_images as f32) * 100.0;
    if coverage < 50.0 {
        eprintln!("⚠ Warning: Low caption coverage ({:.0}%)", coverage);
    }

    Ok(())
}
```

## Next Steps

1. **Implement Timeline Integration**: Connect frame extraction to your timeline module
2. **Setup UI**: Create training control panels in your effects UI
3. **Configure Backend**: Set up ComfyUI or Replicate credentials
4. **Test Pipeline**: Run examples and test with small datasets
5. **Optimize**: Profile and optimize for your target hardware
6. **Deploy**: Integrate into your application workflow
