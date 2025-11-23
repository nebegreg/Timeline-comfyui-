/// Dataset management for LoRA training
/// Phase 4: Automatic LORA Creator

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Training dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Training images with captions
    pub images: Vec<TrainingImage>,

    /// Dataset metadata
    pub metadata: DatasetMetadata,
}

impl Dataset {
    /// Create new empty dataset
    pub fn new(images: Vec<TrainingImage>) -> Self {
        let metadata = DatasetMetadata {
            total_images: images.len(),
            created_at: chrono::Utc::now().timestamp(),
            source: "manual".to_string(),
        };

        Self { images, metadata }
    }

    /// Load dataset from directory
    /// Expects images + .txt files with same name for captions
    pub fn from_directory(path: &Path) -> Result<Self> {
        let mut images = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Check if it's an image
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(ext_str.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                    // Load caption from .txt file
                    let caption_path = path.with_extension("txt");
                    let caption = if caption_path.exists() {
                        std::fs::read_to_string(&caption_path)?
                    } else {
                        String::new()
                    };

                    images.push(TrainingImage {
                        path: path.to_path_buf(),
                        caption,
                        preprocessed: false,
                    });
                }
            }
        }

        Ok(Self::new(images))
    }

    /// Save dataset manifest to JSON
    pub fn save_manifest(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load dataset from manifest
    pub fn load_manifest(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let dataset = serde_json::from_str(&json)?;
        Ok(dataset)
    }

    /// Get dataset statistics
    pub fn stats(&self) -> DatasetStats {
        let total_captions = self.images.iter().filter(|i| !i.caption.is_empty()).count();
        let avg_caption_len = if total_captions > 0 {
            self.images
                .iter()
                .map(|i| i.caption.len())
                .sum::<usize>() as f32
                / total_captions as f32
        } else {
            0.0
        };

        DatasetStats {
            total_images: self.images.len(),
            images_with_captions: total_captions,
            avg_caption_length: avg_caption_len,
        }
    }
}

/// Training image with caption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingImage {
    /// Path to image file
    pub path: PathBuf,

    /// Caption/description
    pub caption: String,

    /// Whether image has been preprocessed
    pub preprocessed: bool,
}

impl TrainingImage {
    /// Load image from disk
    pub fn load_image(&self) -> Result<DynamicImage> {
        image::open(&self.path)
            .with_context(|| format!("Failed to load image: {:?}", self.path))
    }

    /// Get image dimensions without loading full image
    pub fn dimensions(&self) -> Result<(u32, u32)> {
        let reader = image::ImageReader::open(&self.path)?;
        let dimensions = reader.into_dimensions()?;
        Ok(dimensions)
    }
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// Total number of images
    pub total_images: usize,

    /// Creation timestamp
    pub created_at: i64,

    /// Dataset source (timeline, manual, etc.)
    pub source: String,
}

/// Dataset statistics
#[derive(Debug, Clone)]
pub struct DatasetStats {
    pub total_images: usize,
    pub images_with_captions: usize,
    pub avg_caption_length: f32,
}

/// Dataset builder for extracting frames from timeline
pub struct DatasetBuilder {
    images: Vec<TrainingImage>,
    output_dir: PathBuf,
}

impl DatasetBuilder {
    /// Create new dataset builder
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&output_dir)?;

        Ok(Self {
            images: Vec::new(),
            output_dir,
        })
    }

    /// Add image from memory
    pub fn add_image(&mut self, img: DynamicImage, caption: String) -> Result<()> {
        let filename = format!("frame_{:06}.png", self.images.len());
        let path = self.output_dir.join(&filename);

        // Save image
        img.save_with_format(&path, ImageFormat::Png)?;

        // Save caption
        let caption_path = path.with_extension("txt");
        std::fs::write(&caption_path, &caption)?;

        self.images.push(TrainingImage {
            path,
            caption,
            preprocessed: false,
        });

        Ok(())
    }

    /// Add image from file
    pub fn add_image_file(&mut self, src_path: &Path, caption: String) -> Result<()> {
        let filename = format!(
            "frame_{:06}.{}",
            self.images.len(),
            src_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
        );
        let dest_path = self.output_dir.join(&filename);

        // Copy image
        std::fs::copy(src_path, &dest_path)?;

        // Save caption
        let caption_path = dest_path.with_extension("txt");
        std::fs::write(&caption_path, &caption)?;

        self.images.push(TrainingImage {
            path: dest_path,
            caption,
            preprocessed: false,
        });

        Ok(())
    }

    /// Build final dataset
    pub fn build(self) -> Dataset {
        Dataset::new(self.images)
    }

    /// Get current image count
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Check if builder is empty
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

/// Image preprocessing utilities
pub mod preprocess {
    use super::*;
    use image::imageops::{crop_imm, resize, FilterType};

    /// Resize image to target resolution
    pub fn resize_image(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(resize(img, width, height, FilterType::Lanczos3))
    }

    /// Center crop image to target aspect ratio, then resize
    pub fn crop_and_resize(
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> DynamicImage {
        let (src_width, src_height) = img.dimensions();
        let src_aspect = src_width as f32 / src_height as f32;
        let target_aspect = target_width as f32 / target_height as f32;

        let (crop_width, crop_height) = if src_aspect > target_aspect {
            // Source is wider, crop width
            let new_width = (src_height as f32 * target_aspect) as u32;
            (new_width, src_height)
        } else {
            // Source is taller, crop height
            let new_height = (src_width as f32 / target_aspect) as u32;
            (src_width, new_height)
        };

        // Center crop
        let x = (src_width - crop_width) / 2;
        let y = (src_height - crop_height) / 2;

        let cropped = crop_imm(img, x, y, crop_width, crop_height).to_image();

        // Resize to target
        DynamicImage::ImageRgba8(resize(
            &cropped,
            target_width,
            target_height,
            FilterType::Lanczos3,
        ))
    }

    /// Preprocess entire dataset
    pub fn preprocess_dataset(
        dataset: &mut Dataset,
        resolution: (u32, u32),
        output_dir: &Path,
    ) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        for (idx, image) in dataset.images.iter_mut().enumerate() {
            if image.preprocessed {
                continue;
            }

            let img = image.load_image()?;
            let processed = crop_and_resize(&img, resolution.0, resolution.1);

            let new_path = output_dir.join(format!("processed_{:06}.png", idx));
            processed.save(&new_path)?;

            image.path = new_path;
            image.preprocessed = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_creation() {
        let images = vec![
            TrainingImage {
                path: PathBuf::from("test1.png"),
                caption: "A test image".to_string(),
                preprocessed: false,
            },
            TrainingImage {
                path: PathBuf::from("test2.png"),
                caption: "Another test".to_string(),
                preprocessed: false,
            },
        ];

        let dataset = Dataset::new(images);
        assert_eq!(dataset.images.len(), 2);
        assert_eq!(dataset.metadata.total_images, 2);
    }

    #[test]
    fn test_dataset_stats() {
        let images = vec![
            TrainingImage {
                path: PathBuf::from("test1.png"),
                caption: "A test image".to_string(),
                preprocessed: false,
            },
            TrainingImage {
                path: PathBuf::from("test2.png"),
                caption: String::new(),
                preprocessed: false,
            },
        ];

        let dataset = Dataset::new(images);
        let stats = dataset.stats();

        assert_eq!(stats.total_images, 2);
        assert_eq!(stats.images_with_captions, 1);
    }
}
