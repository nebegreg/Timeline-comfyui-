/// LoRA training configuration
/// Phase 4: Automatic LORA Creator
use serde::{Deserialize, Serialize};

/// LoRA rank (dimensionality)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoraRank {
    /// Rank 4 - Very small, fast training
    Rank4 = 4,
    /// Rank 8 - Small, balanced
    Rank8 = 8,
    /// Rank 16 - Medium, good quality
    Rank16 = 16,
    /// Rank 32 - Large, high quality
    Rank32 = 32,
    /// Rank 64 - Very large, best quality
    Rank64 = 64,
    /// Rank 128 - Huge, experimental
    Rank128 = 128,
}

impl LoraRank {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl Default for LoraRank {
    fn default() -> Self {
        Self::Rank16
    }
}

/// LoRA training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// LoRA rank (dimensionality)
    pub rank: LoraRank,

    /// LoRA alpha (scaling factor)
    pub alpha: f32,

    /// Learning rate
    pub learning_rate: f32,

    /// Batch size
    pub batch_size: u32,

    /// Number of training epochs
    pub epochs: u32,

    /// Training resolution (width, height)
    pub resolution: (u32, u32),

    /// Trigger word for activating LoRA
    pub trigger_word: Option<String>,

    /// Whether to train text encoder
    pub train_text_encoder: bool,

    /// Gradient accumulation steps
    pub gradient_accumulation_steps: u32,

    /// Learning rate scheduler
    pub lr_scheduler: LrScheduler,

    /// Warmup steps
    pub warmup_steps: u32,

    /// Save checkpoint every N steps
    pub save_every_n_steps: Option<u32>,

    /// Mixed precision training
    pub mixed_precision: MixedPrecision,

    /// Use 8-bit Adam optimizer
    pub use_8bit_adam: bool,

    /// Seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            rank: LoraRank::Rank16,
            alpha: 16.0,
            learning_rate: 1e-4,
            batch_size: 1,
            epochs: 10,
            resolution: (512, 512),
            trigger_word: None,
            train_text_encoder: false,
            gradient_accumulation_steps: 1,
            lr_scheduler: LrScheduler::Constant,
            warmup_steps: 0,
            save_every_n_steps: None,
            mixed_precision: MixedPrecision::Fp16,
            use_8bit_adam: true,
            seed: Some(42),
        }
    }
}

impl LoraConfig {
    /// Create config optimized for SDXL
    pub fn sdxl_preset() -> Self {
        Self {
            rank: LoraRank::Rank32,
            alpha: 32.0,
            learning_rate: 1e-4,
            batch_size: 1,
            epochs: 10,
            resolution: (1024, 1024),
            mixed_precision: MixedPrecision::Bf16,
            ..Default::default()
        }
    }

    /// Create config optimized for SD 1.5
    pub fn sd15_preset() -> Self {
        Self {
            rank: LoraRank::Rank16,
            alpha: 16.0,
            learning_rate: 1e-4,
            batch_size: 2,
            epochs: 15,
            resolution: (512, 512),
            mixed_precision: MixedPrecision::Fp16,
            ..Default::default()
        }
    }

    /// Create fast training preset (lower quality)
    pub fn fast_preset() -> Self {
        Self {
            rank: LoraRank::Rank8,
            alpha: 8.0,
            learning_rate: 2e-4,
            batch_size: 2,
            epochs: 5,
            resolution: (512, 512),
            ..Default::default()
        }
    }

    /// Create high quality preset (slower training)
    pub fn high_quality_preset() -> Self {
        Self {
            rank: LoraRank::Rank64,
            alpha: 64.0,
            learning_rate: 5e-5,
            batch_size: 1,
            epochs: 20,
            resolution: (768, 768),
            train_text_encoder: true,
            ..Default::default()
        }
    }

    /// Estimate training time in minutes
    pub fn estimate_training_time(&self, num_images: usize) -> f32 {
        // Rough estimate: ~1-2 seconds per step on consumer GPU
        let steps_per_epoch = (num_images as f32 / self.batch_size as f32).ceil();
        let total_steps = steps_per_epoch * self.epochs as f32;
        let seconds_per_step = 1.5; // Average

        (total_steps * seconds_per_step) / 60.0
    }

    /// Estimate VRAM usage in GB
    pub fn estimate_vram_gb(&self) -> f32 {
        let base_vram = match self.resolution {
            (512, 512) => 6.0,
            (768, 768) => 10.0,
            (1024, 1024) => 16.0,
            _ => 8.0,
        };

        let rank_multiplier = self.rank.as_u32() as f32 / 16.0;
        base_vram * rank_multiplier
    }
}

/// Learning rate scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LrScheduler {
    /// Constant learning rate
    Constant,
    /// Linear decay
    Linear,
    /// Cosine annealing
    Cosine,
    /// Cosine with restarts
    CosineWithRestarts,
    /// Polynomial decay
    Polynomial,
}

/// Mixed precision training mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MixedPrecision {
    /// No mixed precision (FP32)
    No,
    /// Half precision (FP16)
    Fp16,
    /// Brain float 16 (BF16) - better for training
    Bf16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoraConfig::default();
        assert_eq!(config.rank, LoraRank::Rank16);
        assert_eq!(config.resolution, (512, 512));
    }

    #[test]
    fn test_training_time_estimate() {
        let config = LoraConfig::default();
        let time = config.estimate_training_time(20);
        assert!(time > 0.0);
        assert!(time < 1000.0); // Sanity check
    }

    #[test]
    fn test_vram_estimate() {
        let config = LoraConfig::sdxl_preset();
        let vram = config.estimate_vram_gb();
        assert!(vram > 10.0); // SDXL requires more VRAM
    }
}
