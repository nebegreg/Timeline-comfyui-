/// Basic LoRA training example
///
/// Run with:
/// cargo run --example basic_training --package ai-pipeline

use ai_pipeline::{LoraCreator, LoraConfig};
use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LoRA Training Example ===\n");

    // Create a new LoRA creator
    let mut creator = LoraCreator::new(
        "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
        PathBuf::from("./lora_training"),
    )?;

    println!("✓ Created LoRA creator for SDXL\n");

    // Configure training using SDXL preset
    let config = LoraConfig::sdxl_preset();
    creator = creator.with_config(config);

    println!("Training Configuration:");
    println!("  Base Model: {}", creator.base_model);
    println!("  Rank: {:?}", creator.config.rank);
    println!("  Alpha: {}", creator.config.alpha);
    println!("  Learning Rate: {}", creator.config.learning_rate);
    println!("  Resolution: {}x{}", creator.config.resolution.0, creator.config.resolution.1);
    println!("  Epochs: {}", creator.config.epochs);
    println!("  Batch Size: {}", creator.config.batch_size);
    println!("  Mixed Precision: {:?}\n", creator.config.mixed_precision);

    // Save configuration for reference
    creator.save_config(&PathBuf::from("./lora_training/config.json"))?;
    println!("✓ Saved configuration to config.json\n");

    // In a real scenario, you would:
    // 1. Load dataset from directory or timeline frames
    // creator.load_dataset(&PathBuf::from("./dataset"))?;

    // 2. Setup a backend (ComfyUI or Replicate)
    // let backend = backends::ComfyUIBackend::new(config)?;
    // creator = creator.with_backend(Box::new(backend));

    // 3. Prepare dataset (preprocessing, resizing, etc.)
    // creator.prepare_dataset().await?;

    // 4. Start training
    // let job = creator.train().await?;

    // 5. Monitor progress
    // loop {
    //     let progress = creator.monitor_progress(&job.id).await?;
    //     println!("Progress: {:.1}% ({}/{})",
    //         progress.progress,
    //         progress.current_step,
    //         progress.total_steps);
    //
    //     if progress.is_finished() { break; }
    //     tokio::time::sleep(Duration::from_secs(30)).await;
    // }

    println!("Example completed successfully!");
    println!("Note: This example shows configuration only.");
    println!("To actually train, configure a backend and provide a dataset.");

    Ok(())
}
