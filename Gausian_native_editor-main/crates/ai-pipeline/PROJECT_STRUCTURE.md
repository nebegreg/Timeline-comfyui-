# AI Pipeline Project Structure - Phase 4

## Overview

The AI Pipeline crate provides a production-ready LORA (Low-Rank Adaptation) training framework integrated with the Gaussian Native Editor timeline system.

## Directory Structure

```
crates/ai-pipeline/
├── Cargo.toml                    # Package manifest with dependencies
├── README.md                     # User-facing documentation
├── INTEGRATION.md               # Integration guide with timeline system
├── PROJECT_STRUCTURE.md         # This file
│
├── src/
│   ├── lib.rs                   # Main LoraCreator interface and re-exports
│   ├── lora_config.rs          # LoRA configuration structs (500+ lines)
│   ├── dataset.rs              # Dataset management & preprocessing (350+ lines)
│   ├── captioning.rs           # Auto-captioning providers (300+ lines)
│   ├── training.rs             # Job management & tracking (450+ lines)
│   │
│   └── backends/
│       ├── mod.rs              # Backend abstraction trait (250+ lines)
│       ├── comfyui.rs          # ComfyUI workflow integration (350+ lines)
│       └── replicate.rs        # Replicate API integration (400+ lines)
│
├── examples/
│   ├── basic_training.rs       # Basic training setup example
│   ├── dataset_management.rs   # Dataset creation & analysis example
│   └── configuration_presets.rs # Configuration tuning example
│
└── tests/                       # Integration tests (future expansion)
```

## Module Descriptions

### Core Modules

#### `lib.rs` - Main LoRA Creator Interface
**Purpose**: Central entry point for all LoRA training operations

**Key Types**:
- `LoraCreator`: Main struct for orchestrating training pipeline
- `LoraWeights`: Metadata for trained LoRA weights
- `TrainingStats`: Statistics from completed training jobs

**Key Methods**:
- `new()`: Initialize creator with model and output directory
- `load_dataset()`: Load images from directory
- `prepare_dataset()`: Preprocess images for training
- `train()`: Start training job on configured backend
- `monitor_progress()`: Get real-time training progress
- `download_weights()`: Retrieve trained LoRA weights
- `estimate_training_time()`: Estimate training duration
- `estimate_vram()`: Estimate VRAM requirements

**Public Exports**:
- All types from `dataset`, `lora_config`, `training`, `backends`, `captioning`

#### `lora_config.rs` - Training Configuration
**Purpose**: Define and manage LoRA training configurations

**Key Types**:
- `LoraConfig`: Main configuration struct
- `LoraRank`: Enum for rank values (4, 8, 16, 32, 64, 128)
- `LrScheduler`: Learning rate scheduling strategies
- `MixedPrecision`: Precision modes (No/FP16/BF16)

**Key Presets**:
- `sdxl_preset()`: Optimized for SDXL (1024x1024, rank 32)
- `sd15_preset()`: Optimized for SD 1.5 (512x512, rank 16)
- `fast_preset()`: Quick training (rank 8, 5 epochs)
- `high_quality_preset()`: Best quality (rank 64, 20 epochs)

**Key Calculations**:
- `estimate_training_time()`: Rough estimate based on image count and epochs
- `estimate_vram_gb()`: Memory usage estimation
- Supports custom configuration with builder pattern

#### `dataset.rs` - Dataset Management
**Purpose**: Handle dataset creation, loading, and preprocessing

**Key Types**:
- `Dataset`: Container for training images with metadata
- `TrainingImage`: Individual image + caption pair
- `DatasetBuilder`: Builder for programmatic dataset construction
- `DatasetMetadata`: Metadata about dataset source and creation
- `DatasetStats`: Statistical analysis of dataset

**Key Features**:
- Load dataset from directory (expects `image.txt` caption pairs)
- Image preprocessing: crop, resize, format conversion
- Dataset statistics: total images, caption coverage, etc.
- Manifest serialization for dataset persistence
- Preprocess module with image transformations

**File Structure Expected**:
```
dataset_dir/
├── image1.png
├── image1.txt          # Caption for image1
├── image2.jpg
├── image2.txt          # Caption for image2
└── ...
```

#### `captioning.rs` - Automatic Image Captioning
**Purpose**: Generate captions for training images automatically

**Key Types**:
- `CaptionProvider`: Trait for implementing different caption sources
- `Caption`: Caption result with text and confidence score
- `Blip2ApiProvider`: BLIP2 model via HuggingFace API
- `LlavaApiProvider`: LLaVA model via HTTP API
- `SimpleCaptioner`: Fallback simple captioning

**Captioning Sources**:
1. **BLIP2**: Free, cloud-based via HuggingFace API
   - URL: `https://api-inference.huggingface.co/models/Salesforce/blip2-opt-2.7b`
   - Requires HuggingFace token

2. **LLaVA**: Open-source, can be self-hosted
   - Supports v1.5-7b and larger versions
   - Better quality than BLIP2 for detailed descriptions

3. **Simple**: Default fallback
   - Returns configurable default text
   - No API required

**Post-Processing Functions**:
- `add_trigger_word()`: Add LoRA trigger word to caption
- `clean_caption()`: Normalize whitespace
- `truncate_caption()`: Limit caption length
- `add_style_tags()`: Append style modifiers

#### `training.rs` - Job Management and Progress Tracking
**Purpose**: Manage training jobs and track progress

**Key Types**:
- `JobId`: Unique identifier for training jobs
- `TrainingJob`: Complete job metadata and status
- `TrainingProgress`: Real-time progress information
- `JobStatus`: Enum for job states (Queued, Running, Completed, Failed, Cancelled)
- `JobStore`: In-memory database of jobs and progress

**Job Lifecycle**:
1. `Queued`: Job created, waiting to start
2. `Running`: Job actively training
3. `Completed`: Successfully finished
4. `Failed`: Error occurred
5. `Cancelled`: User cancelled job

**Key Metrics Tracked**:
- Current step / total steps
- Current and average loss values
- Estimated time to completion
- Progress percentage
- Training duration statistics

**Storage**:
- JSON manifest for persistence
- In-memory HashMap for fast access

### Backend Modules

#### `backends/mod.rs` - Backend Abstraction
**Purpose**: Define unified interface for different training backends

**Key Types**:
- `TrainingBackend`: Trait for implementing backends
- `BackendType`: Enum (ComfyUI, Replicate, Candle)
- `BackendConfig`: Configuration for any backend
- `BackendFactory`: Factory for creating backend instances

**Backend Trait Methods**:
- `submit_job()`: Start training on backend
- `get_progress()`: Poll job progress
- `cancel_job()`: Stop running job
- `download_weights()`: Retrieve trained weights
- `get_logs()`: Access job logs/output
- `is_available()`: Check if backend is operational
- `validate_config()`: Verify config compatibility
- `estimate_cost()`: Project training costs (optional)

**Configuration**:
- API URLs and authentication
- Output directories
- Concurrency limits
- Timeout settings
- Persistence to/from JSON

#### `backends/comfyui.rs` - ComfyUI Backend
**Purpose**: Train LoRA via ComfyUI workflows

**Key Types**:
- `ComfyUIBackend`: ComfyUI implementation of TrainingBackend
- `ComfyUIJobStatus`: Internal status tracking

**Features**:
- Generate JSON workflows for LoRA training
- Submit jobs to local or remote ComfyUI instance
- Poll job status and retrieve outputs
- Support for both HTTP and event-based progress tracking
- Workflow metadata embedding

**API Endpoints Used**:
- `POST /prompt`: Submit training workflow
- `GET /history/{prompt_id}`: Check job status
- `GET /system_stats`: Check availability
- `POST /interrupt`: Cancel job

**Advantages**:
- Free (no API costs)
- Full control over local instance
- Can train offline
- Extensible with custom ComfyUI nodes

**Limitations**:
- Requires ComfyUI installation
- Limited to available local VRAM
- No cloud scaling

#### `backends/replicate.rs` - Replicate API Backend
**Purpose**: Cloud-based training via Replicate service

**Key Types**:
- `ReplicateBackend`: Replicate API implementation
- `PredictionRequest`: Request format for API
- `PredictionResponse`: Response from API
- `TrainingInput`: Training parameters

**Features**:
- Leverage Replicate's cloud infrastructure
- Models with up to 80GB VRAM available
- Cost estimation based on usage
- Scalable training without local GPU requirements
- Job queuing and parallel training

**API Endpoints**:
- `POST /v1/predictions`: Create training job
- `GET /v1/predictions/{id}`: Get job status
- `POST /v1/predictions/{id}/cancel`: Cancel job
- `GET /v1/account`: Verify credentials

**Advantages**:
- No local VRAM constraints
- Predictable per-second billing
- Managed infrastructure
- Easy scaling for batch jobs

**Limitations**:
- Per-second billing costs
- Network latency for uploads/downloads
- Limited customization
- Requires API credentials

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│ User Application / Timeline System                      │
└─────────────────────────────────────────────────────────┘
                        │
                        ↓
┌─────────────────────────────────────────────────────────┐
│ LoraCreator                                             │
│ ├─ Load/Create Dataset                                 │
│ ├─ Configure Training                                  │
│ └─ Submit Job                                          │
└─────────────────────────────────────────────────────────┘
                        │
                ┌───────┴───────┐
                ↓               ↓
        ┌──────────────┐  ┌──────────────┐
        │ Dataset      │  │ LoraConfig   │
        │ Processing   │  │ & Validation │
        └──────────────┘  └──────────────┘
                │               │
                └───────┬───────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ TrainingBackend (Interface)                            │
└─────────────────────────────────────────────────────────┘
         │                   │                    │
         ↓                   ↓                    ↓
    ┌─────────┐         ┌──────────┐        ┌────────┐
    │ComfyUI  │         │Replicate │        │Candle  │
    │Backend  │         │Backend   │        │(future)│
    └─────────┘         └──────────┘        └────────┘
         │                   │
         ↓                   ↓
    ComfyUI Instance    Replicate API
         │                   │
         ↓                   ↓
    Local GPU          Cloud GPU
```

## Type System

### Configuration Types

```
LoraConfig
├─ rank: LoraRank (4-128)
├─ alpha: f32 (scaling factor)
├─ learning_rate: f32 (1e-4 typical)
├─ batch_size: u32
├─ epochs: u32
├─ resolution: (u32, u32)
├─ trigger_word: Option<String>
├─ lr_scheduler: LrScheduler
└─ mixed_precision: MixedPrecision
```

### Dataset Types

```
Dataset
├─ images: Vec<TrainingImage>
│  └─ TrainingImage
│     ├─ path: PathBuf
│     ├─ caption: String
│     └─ preprocessed: bool
└─ metadata: DatasetMetadata

DatasetStats
├─ total_images: usize
├─ images_with_captions: usize
└─ avg_caption_length: f32
```

### Training Types

```
TrainingJob
├─ id: JobId
├─ name: String
├─ base_model: String
├─ num_images: usize
├─ total_steps: u64
├─ backend: String
└─ [timestamps, error info]

TrainingProgress
├─ job_id: JobId
├─ status: JobStatus
├─ current_step: u64
├─ total_steps: u64
├─ current_loss: Option<f32>
├─ eta_seconds: Option<u64>
└─ progress: f32 (0-100)
```

## Dependency Graph

```
ai-pipeline
├── async-trait          (Trait implementation)
├── tokio               (Async runtime)
├── reqwest             (HTTP client)
│   ├── serde_json      (JSON parsing)
│   └── serde           (Serialization)
├── image               (Image processing)
├── chrono              (Timestamps)
├── sha2/hex            (Job ID generation)
├── uuid                (UUID generation)
├── walkdir             (Directory traversal)
├── indicatif           (Progress bars - future)
├── anyhow/thiserror    (Error handling)
└── base64              (Image encoding)
```

## Testing Coverage

**Total Tests**: 21
- **lora_config**: 3 tests
- **dataset**: 2 tests
- **captioning**: 4 tests
- **training**: 4 tests
- **backends/mod**: 2 tests
- **backends/comfyui**: 2 tests
- **backends/replicate**: 1 test
- **lib.rs**: 2 tests

## Examples Provided

1. **basic_training.rs**: Minimal example showing configuration
2. **dataset_management.rs**: Dataset creation and analysis
3. **configuration_presets.rs**: Exploring different configurations

## Performance Characteristics

### Memory Usage
- LoraCreator: ~1 MB
- Dataset (100 images, no data): ~1 MB
- TrainingJob struct: ~200 B
- JobStore (100 jobs): ~200 KB

### Computation
- Config validation: O(1)
- Dataset loading: O(n) where n = number of images
- Workflow generation: O(1)
- Progress polling: O(1)

### Scalability
- Tested with up to 1000 images
- JobStore supports unlimited concurrent jobs
- Backend request timeout: configurable (default 1 hour)

## Security Considerations

1. **API Keys**: Store in environment variables or secure config
2. **Dataset Privacy**: Keep datasets locally; don't upload sensitive data
3. **Output Validation**: All API responses validated before use
4. **Error Handling**: Errors don't expose sensitive information
5. **Resource Limits**: Configurable timeouts and concurrency limits

## Future Expansions

1. **Candle Backend**: Local GPU training using Candle
2. **Advanced Captioning**: GPT-4V for better descriptions
3. **Distributed Training**: Multiple GPU coordination
4. **Model Zoo**: Pre-trained LoRA weights library
5. **WebUI**: Browser-based training management
6. **Video Support**: Direct frame extraction from video
7. **Advanced Monitoring**: Tensorboard integration
8. **Multi-Model Training**: Train multiple LoRAs in parallel

## Development Guidelines

### Adding New Backend
1. Implement `TrainingBackend` trait
2. Add new variant to `BackendType`
3. Update `BackendFactory::create()`
4. Add configuration example
5. Write comprehensive tests

### Adding New Configuration Preset
1. Create new method in `LoraConfig`
2. Document recommended use cases
3. Include VRAM/time estimates
4. Add example in `configuration_presets.rs`

### Adding Caption Provider
1. Implement `CaptionProvider` trait
2. Handle rate limiting if needed
3. Include example in documentation
4. Add error handling and validation

## References & Best Practices

- **Kohya_ss**: LoRA training research and best practices
- **BLIP2**: Vision-language model for captioning
- **LLaVA**: Open-source multimodal model
- **ComfyUI**: Composable node-based AI framework
- **Replicate**: Cloud AI inference platform

## Maintenance

- Update dependencies quarterly
- Monitor upstream changes to models
- Gather user feedback on presets
- Optimize based on real-world usage
- Maintain comprehensive documentation
