/// Dataset management example
///
/// Demonstrates dataset creation, preprocessing, and statistics
///
/// Run with:
/// cargo run --example dataset_management --package ai-pipeline

use ai_pipeline::dataset::{Dataset, DatasetBuilder};
use ai_pipeline::dataset::preprocess;
use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("=== Dataset Management Example ===\n");

    // Create a simple in-memory dataset
    println!("1. Creating dataset from images...");
    create_sample_dataset()?;

    // Load and analyze
    println!("\n2. Loading and analyzing dataset...");
    analyze_dataset()?;

    // Preprocessing example
    println!("\n3. Dataset preprocessing...");
    preprocess_example()?;

    println!("\n✓ Example completed!");

    Ok(())
}

fn create_sample_dataset() -> Result<()> {
    use ai_pipeline::dataset::TrainingImage;

    // Create sample training images
    let images = vec![
        TrainingImage {
            path: PathBuf::from("path/to/image1.png"),
            caption: "a beautiful landscape with mountains".to_string(),
            preprocessed: false,
        },
        TrainingImage {
            path: PathBuf::from("path/to/image2.png"),
            caption: "a serene lake at sunset".to_string(),
            preprocessed: false,
        },
        TrainingImage {
            path: PathBuf::from("path/to/image3.png"),
            caption: "forest trees with golden light".to_string(),
            preprocessed: false,
        },
    ];

    let dataset = Dataset::new(images);

    // Print dataset info
    println!("   Created dataset with {} images", dataset.images.len());
    for (i, img) in dataset.images.iter().enumerate() {
        println!("   Image {}: {}", i + 1, img.caption);
    }

    // Save dataset manifest
    dataset.save_manifest(&PathBuf::from("./dataset_manifest.json"))?;
    println!("   ✓ Saved manifest to dataset_manifest.json");

    Ok(())
}

fn analyze_dataset() -> Result<()> {
    use ai_pipeline::dataset::TrainingImage;

    let images = vec![
        TrainingImage {
            path: PathBuf::from("image1.png"),
            caption: "a photo of a dog".to_string(),
            preprocessed: false,
        },
        TrainingImage {
            path: PathBuf::from("image2.png"),
            caption: "a portrait of a woman".to_string(),
            preprocessed: false,
        },
        TrainingImage {
            path: PathBuf::from("image3.png"),
            caption: String::new(), // No caption
            preprocessed: false,
        },
        TrainingImage {
            path: PathBuf::from("image4.png"),
            caption: "a landscape painting".to_string(),
            preprocessed: false,
        },
    ];

    let dataset = Dataset::new(images);
    let stats = dataset.stats();

    println!("   Dataset Statistics:");
    println!("   - Total images: {}", stats.total_images);
    println!("   - Images with captions: {}", stats.images_with_captions);
    println!("   - Images without captions: {}", stats.total_images - stats.images_with_captions);
    println!("   - Average caption length: {:.1} chars", stats.avg_caption_length);

    // Data quality assessment
    let caption_coverage = (stats.images_with_captions as f32 / stats.total_images as f32) * 100.0;
    println!("   - Caption coverage: {:.1}%", caption_coverage);

    if caption_coverage < 50.0 {
        println!("   ⚠ Warning: Low caption coverage. Consider adding more captions.");
    } else if caption_coverage < 80.0 {
        println!("   ⚠ Warning: Some images missing captions.");
    } else {
        println!("   ✓ Good caption coverage");
    }

    Ok(())
}

fn preprocess_example() -> Result<()> {
    println!("   Preprocessing workflow:");
    println!("   1. Load original image");
    println!("   2. Center crop to target aspect ratio");
    println!("   3. Resize to target resolution");
    println!("   4. Convert to model-compatible format");
    println!("   5. Save preprocessed image");

    // Example: Show resolution transformations
    let resolutions = vec![
        ("SD 1.5", (512, 512)),
        ("SDXL", (1024, 1024)),
        ("Flux", (1024, 1024)),
        ("SD 3.5", (1024, 1024)),
    ];

    println!("\n   Target resolutions by model:");
    for (model, (w, h)) in resolutions {
        println!("   - {}: {}x{}", model, w, h);
    }

    println!("\n   VRAM estimates:");
    let vram_estimates = vec![
        ("Rank 8, 512x512", 4.0),
        ("Rank 16, 512x512", 6.0),
        ("Rank 16, 1024x1024", 14.0),
        ("Rank 32, 1024x1024", 24.0),
        ("Rank 64, 1024x1024", 32.0),
    ];

    for (config, vram) in vram_estimates {
        println!("   - {}: ~{:.1} GB", config, vram);
    }

    Ok(())
}
