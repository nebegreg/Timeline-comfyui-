# AI Pipeline - LORA Creator (Phase 4)

Production-ready LORA (Low-Rank Adaptation) training pipeline for the Gaussian Native Editor. Supports multiple training backends and integrates with the timeline system for automatic dataset generation.

## Features

- **Multi-Backend Support**
  - ComfyUI (local and remote)
  - Replicate API (cloud-based)
  - Candle (local, experimental)

- **Automatic Dataset Creation**
  - Frame extraction from timeline
  - Image preprocessing (crop, resize)
  - Automatic caption generation (BLIP2, LLaVA)
  - Dataset validation and statistics

- **Flexible Training Configuration**
  - Multiple model presets (SDXL, SD 1.5, Flux, SD 3.5)
  - Rank/alpha customization
  - Resolution optimization (512x512 to 2048x2048)
  - Learning rate scheduling
  - Mixed precision training

- **Job Management**
  - Async training orchestration
  - Progress tracking with ETAs
  - Job history and logging
  - Cost estimation (Replicate)

## Architecture

```
ai-pipeline/
├── lib.rs                  # Main LoraCreator interface
├── dataset.rs             # Dataset management & preprocessing
├── lora_config.rs         # Configuration and presets
├── captioning.rs          # Auto-captioning providers
├── training.rs            # Job management and tracking
└── backends/
    ├── mod.rs             # Backend trait and factory
    ├── comfyui.rs         # ComfyUI workflow integration
    └── replicate.rs       # Replicate API integration
```

## Quick Start

### Basic Usage

```rust
use ai_pipeline::{LoraCreator, LoraConfig, BackendConfig, BackendType};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Create LoRA creator
    let mut creator = LoraCreator::new(
        "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
        PathBuf::from("./lora_output"),
    )?;

    // Configure training
    let config = LoraConfig::sdxl_preset()
        .with_trigger_word("myloraname");
    creator = creator.with_config(config);

    // Setup backend (ComfyUI)
    let backend_config = BackendConfig::new(
        BackendType::ComfyUI,
        PathBuf::from("./lora_output"),
    )
    .with_api_url("http://localhost:8188".to_string());

    let backend = ai_pipeline::backends::ComfyUIBackend::new(backend_config)?;
    creator = creator.with_backend(Box::new(backend));

    // Load dataset
    creator.load_dataset(&PathBuf::from("./dataset"))?;

    // Prepare for training
    creator.prepare_dataset().await?;

    // Start training
    let job = creator.train().await?;
    println!("Training job started: {:?}", job.id);

    // Monitor progress
    loop {
        let progress = creator.monitor_progress(&job.id).await?;
        println!("Progress: {:.1}%", progress.progress);

        if progress.is_finished() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }

    // Download weights
    let weights_path = creator.download_weights(&job.id).await?;
    println!("Weights saved to: {:?}", weights_path);

    Ok(())
}
```

### Using Caption Providers

```rust
use ai_pipeline::captioning::{CaptionProvider, Blip2ApiProvider};

#[tokio::main]
async fn main() -> Result<()> {
    // Create BLIP2 captioner with HuggingFace API
    let captioner = Blip2ApiProvider::huggingface("hf_your_api_key".to_string());

    // Or LLaVA
    let captioner = LlavaApiProvider::new(
        "http://localhost:8000".to_string(),
        None,
    );

    // Generate captions
    let img = image::open("path/to/image.png")?;
    let caption = captioner.caption(&img).await?;
    println!("Caption: {}", caption.text);

    Ok(())
}
```

### Dataset Management

```rust
use ai_pipeline::{DatasetBuilder, Dataset};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Create dataset from directory
    let dataset = Dataset::from_directory(&PathBuf::from("./images"))?;
    println!("Loaded {} images", dataset.images.len());

    // Get statistics
    let stats = dataset.stats();
    println!("Images with captions: {}", stats.images_with_captions);

    // Or build dataset programmatically
    let mut builder = DatasetBuilder::new(PathBuf::from("./output_dataset"))?;

    let img = image::open("image1.png")?;
    builder.add_image(img, "a beautiful landscape".to_string())?;

    let dataset = builder.build();
    dataset.save_manifest(&PathBuf::from("dataset.json"))?;

    Ok(())
}
```

## Configuration Presets

### SDXL (Recommended)
```rust
let config = LoraConfig::sdxl_preset();
// Rank 32, Alpha 32, Resolution 1024x1024, 10 epochs, BF16 precision
```

### SD 1.5
```rust
let config = LoraConfig::sd15_preset();
// Rank 16, Alpha 16, Resolution 512x512, 15 epochs, FP16 precision
```

### Fast Training (Quick Iteration)
```rust
let config = LoraConfig::fast_preset();
// Rank 8, Alpha 8, Resolution 512x512, 5 epochs
```

### High Quality (Slower but Better)
```rust
let config = LoraConfig::high_quality_preset();
// Rank 64, Alpha 64, Resolution 768x768, 20 epochs, Text encoder training
```

## Training Best Practices

Based on Kohya_ss research:

### Dataset Preparation
- **Minimum images**: 10-20 images for good results
- **Recommended**: 100-200 images for high quality
- **Maximum**: 1000+ for exceptional quality (requires more VRAM)
- **Format**: PNG/JPG, 512x512 minimum
- **Aspect ratio**: Keep consistent across dataset

### LoRA Configuration
- **Rank**: 16-32 for most use cases
  - 4-8: Fast training, lower VRAM
  - 16-32: Balanced quality/speed
  - 64+: High quality, requires 24GB+ VRAM
- **Alpha**: Usually matches rank (e.g., rank 16 → alpha 16)
- **Learning rate**: 1e-4 for most cases
- **Batch size**: 1 for consumer GPUs

### Resolution
- **SD 1.5**: 512x512 optimal
- **SDXL**: 1024x1024 minimum (recommended)
- **Flux**: 1024x1024 recommended
- **SD 3.5**: 1024x1024 recommended

### Training Steps
- **Minimum**: 300 steps (100 images × 3 epochs)
- **Recommended**: 500-1200 steps (100 images × 5-12 epochs)
- **Flux**: 1200-1800 steps recommended
- **Formula**: `steps = num_images × epochs`

### Trigger Word
- Use unique, descriptive trigger words
- Examples: "myloraname", "xyz_style", "character_name"
- Avoid common words to prevent overfitting

## Backend Comparison

| Feature | ComfyUI | Replicate |
|---------|---------|-----------|
| **Speed** | Fast (local) | Medium (cloud) |
| **Cost** | Free (local) | $0.0001-0.001/sec |
| **Setup** | Requires installation | API key only |
| **Customization** | High (local) | Limited (cloud) |
| **VRAM** | Uses local GPU | Up to 80GB |
| **Batch Size** | Limited by GPU | Up to 8 |
| **Queue** | Unlimited | Per account |

## Examples

### Example 1: Train on Timeline Frames

```rust
use ai_pipeline::{LoraCreator, LoraConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup creator
    let mut creator = LoraCreator::new(
        "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
        PathBuf::from("./lora_output"),
    )?;

    // Use SDXL preset optimized for high quality
    let config = LoraConfig::sdxl_preset();
    creator = creator.with_config(config);

    // Load frames extracted from timeline
    creator.load_dataset(&PathBuf::from("./timeline_frames"))?;

    // Print dataset info
    if let Some(stats) = creator.dataset_stats() {
        println!("Dataset: {} images", stats.total_images);
        println!("Captions: {}/{}", stats.images_with_captions, stats.total_images);
        println!("Avg caption length: {:.0}", stats.avg_caption_length);
    }

    // Estimate requirements
    println!("Est. training time: {:.1} min", creator.estimate_training_time()?);
    println!("Est. VRAM: {:.1} GB", creator.estimate_vram());

    Ok(())
}
```

### Example 2: Cost-Aware Training with Replicate

```rust
use ai_pipeline::{LoraCreator, BackendConfig, BackendType};

#[tokio::main]
async fn main() -> Result<()> {
    let mut creator = LoraCreator::new(
        "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
        PathBuf::from("./lora_output"),
    )?;

    creator.load_dataset(&PathBuf::from("./dataset"))?;

    // Setup Replicate backend
    let backend_config = BackendConfig::new(
        BackendType::Replicate,
        PathBuf::from("./lora_output"),
    )
    .with_api_key("r8_your_key_here".to_string());

    let backend = ai_pipeline::backends::ReplicateBackend::new(backend_config)?;

    // Estimate cost before training
    if let Some(cost) = backend.estimate_cost(
        &creator.base_model,
        creator.dataset.as_ref().unwrap(),
        &creator.config,
    ).await? {
        println!("Estimated cost: ${:.2}", cost);
    }

    creator = creator.with_backend(Box::new(backend));

    // Proceed with training...
    Ok(())
}
```

## Performance Tuning

### For Consumer GPUs (8-12 GB VRAM)
```rust
let config = LoraConfig {
    rank: LoraRank::Rank16,
    resolution: (512, 512),
    batch_size: 2,
    gradient_accumulation_steps: 4,
    mixed_precision: MixedPrecision::Fp16,
    ..Default::default()
};
```

### For High-End GPUs (24+ GB VRAM)
```rust
let config = LoraConfig::high_quality_preset();
```

### For Minimal VRAM (4-6 GB)
```rust
let config = LoraConfig::fast_preset();
```

## Testing

```bash
# Run tests
cargo test --package ai-pipeline

# Run with output
cargo test --package ai-pipeline -- --nocapture

# Run specific test
cargo test --package ai-pipeline test_lora_creator_creation -- --exact
```

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client for API integration
- `image`: Image processing
- `serde`: Serialization/deserialization
- `sha2`: Job ID generation
- `chrono`: Timestamp handling
- `anyhow`: Error handling

## Known Limitations

1. **Candle backend**: Not yet implemented (experimental)
2. **Local inference**: Requires Candle feature flag
3. **Image captioning**: Requires external API or local setup
4. **Batch size**: Limited by available VRAM

## Roadmap

- [ ] Implement Candle backend for local training
- [ ] Add local BLIP2/LLaVA inference
- [ ] Support for video frame extraction
- [ ] Advanced caption enhancement with GPT
- [ ] Multi-model training orchestration
- [ ] Distributed training support
- [ ] WebUI for training management

## Contributing

To extend the pipeline:

1. **New Backend**: Implement `TrainingBackend` trait
2. **New Caption Provider**: Implement `CaptionProvider` trait
3. **New Model Preset**: Add to `LoraConfig::*_preset()`
4. **Dataset Processor**: Extend `dataset::preprocess` module

## License

Part of Gaussian Native Editor project.
