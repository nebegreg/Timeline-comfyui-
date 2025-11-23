/// Configuration presets example
///
/// Demonstrates different LoRA training configurations optimized
/// for different use cases and hardware constraints
///
/// Run with:
/// cargo run --example configuration_presets --package ai-pipeline

use ai_pipeline::lora_config::{LoraConfig, LoraRank, LrScheduler, MixedPrecision};

fn main() {
    println!("=== LoRA Configuration Presets ===\n");

    println!("1. SDXL Preset (Recommended for SDXL models)");
    print_config(&LoraConfig::sdxl_preset());

    println!("\n2. SD 1.5 Preset (For Stable Diffusion 1.5)");
    print_config(&LoraConfig::sd15_preset());

    println!("\n3. Fast Training Preset (Quick iteration)");
    print_config(&LoraConfig::fast_preset());

    println!("\n4. High Quality Preset (Best results, slower)");
    print_config(&LoraConfig::high_quality_preset());

    println!("\n5. Custom Configuration Examples");
    print_custom_configs();

    println!("\n6. VRAM Requirements");
    print_vram_estimates();

    println!("\n7. Training Time Estimates");
    print_time_estimates();
}

fn print_config(config: &LoraConfig) {
    println!("   Rank: {:?} ({})", config.rank, config.rank.as_u32());
    println!("   Alpha: {}", config.alpha);
    println!("   Learning Rate: {}", config.learning_rate);
    println!("   Batch Size: {}", config.batch_size);
    println!("   Epochs: {}", config.epochs);
    println!("   Resolution: {}x{}", config.resolution.0, config.resolution.1);
    println!("   Mixed Precision: {:?}", config.mixed_precision);
    println!("   Train Text Encoder: {}", config.train_text_encoder);
    println!("   Gradient Accumulation: {}", config.gradient_accumulation_steps);
    println!("   Learning Rate Scheduler: {:?}", config.lr_scheduler);
    println!("   Warmup Steps: {}", config.warmup_steps);
    println!("   8-bit Adam: {}", config.use_8bit_adam);

    // Calculate estimates
    let num_images = 100; // Example
    let training_time = config.estimate_training_time(num_images);
    let vram = config.estimate_vram_gb();

    println!("   Est. training time (100 images): {:.1} minutes", training_time);
    println!("   Est. VRAM usage: {:.1} GB", vram);
}

fn print_custom_configs() {
    println!("   For Consumer GPU (8GB VRAM):");
    let consumer = LoraConfig {
        rank: LoraRank::Rank16,
        alpha: 16.0,
        learning_rate: 1e-4,
        batch_size: 1,
        epochs: 10,
        resolution: (512, 512),
        train_text_encoder: false,
        gradient_accumulation_steps: 4,
        mixed_precision: MixedPrecision::Fp16,
        lr_scheduler: LrScheduler::Linear,
        ..Default::default()
    };
    println!("   - Rank 16, 512x512, Batch 1 with gradient accumulation");
    println!("   - Est. VRAM: {:.1} GB", consumer.estimate_vram_gb());

    println!("\n   For Low VRAM (4-6GB):");
    let low_vram = LoraConfig {
        rank: LoraRank::Rank8,
        alpha: 8.0,
        learning_rate: 2e-4,
        batch_size: 1,
        epochs: 5,
        resolution: (512, 512),
        gradient_accumulation_steps: 8,
        mixed_precision: MixedPrecision::Fp16,
        use_8bit_adam: true,
        ..Default::default()
    };
    println!("   - Rank 8, 512x512, 8-bit Adam");
    println!("   - Est. VRAM: {:.1} GB", low_vram.estimate_vram_gb());

    println!("\n   For High-End GPU (24GB+):");
    let high_end = LoraConfig {
        rank: LoraRank::Rank64,
        alpha: 64.0,
        learning_rate: 5e-5,
        batch_size: 2,
        epochs: 20,
        resolution: (1024, 1024),
        train_text_encoder: true,
        gradient_accumulation_steps: 2,
        mixed_precision: MixedPrecision::Bf16,
        lr_scheduler: LrScheduler::Cosine,
        warmup_steps: 100,
        ..Default::default()
    };
    println!("   - Rank 64, 1024x1024, Text encoder training");
    println!("   - Est. VRAM: {:.1} GB", high_end.estimate_vram_gb());
}

fn print_vram_estimates() {
    let configs = vec![
        ("Default (Rank 16, 512x512)", LoraConfig::default()),
        ("SDXL (Rank 32, 1024x1024)", LoraConfig::sdxl_preset()),
        ("High Quality (Rank 64, 768x768)", LoraConfig::high_quality_preset()),
        ("Fast (Rank 8, 512x512)", LoraConfig::fast_preset()),
    ];

    println!("   VRAM Requirements:");
    for (name, config) in configs {
        let vram = config.estimate_vram_gb();
        println!("   - {}: ~{:.1} GB", name, vram);
    }
}

fn print_time_estimates() {
    println!("   Training time estimates (100 images):");

    let configs = vec![
        ("Fast (5 epochs)", LoraConfig::fast_preset()),
        ("Standard (10 epochs)", LoraConfig::default()),
        ("SDXL (10 epochs)", LoraConfig::sdxl_preset()),
        ("High Quality (20 epochs)", LoraConfig::high_quality_preset()),
    ];

    for (name, config) in configs {
        let time = config.estimate_training_time(100);
        let hours = time / 60.0;
        if hours < 1.0 {
            println!("   - {}: {:.0} minutes", name, time);
        } else {
            println!("   - {}: {:.1} hours", name, hours);
        }
    }

    println!("\n   Note: Actual times depend on GPU model, VRAM, and training resolution");
}
