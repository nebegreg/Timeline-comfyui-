/// Image captioning for LoRA training
/// Phase 4: Automatic LORA Creator
///
/// Provides automatic caption generation using BLIP2, LLaVA, or other models
use anyhow::Result;
use async_trait::async_trait;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

/// Caption generated for an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caption {
    /// Caption text
    pub text: String,

    /// Confidence score (0-1)
    pub confidence: f32,

    /// Provider that generated the caption
    pub provider: String,
}

/// Caption provider trait
#[async_trait]
pub trait CaptionProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Generate caption for an image
    async fn caption(&self, image: &DynamicImage) -> Result<Caption>;

    /// Batch caption multiple images
    async fn caption_batch(&self, images: &[DynamicImage]) -> Result<Vec<Caption>> {
        let mut captions = Vec::new();
        for img in images {
            captions.push(self.caption(img).await?);
        }
        Ok(captions)
    }
}

/// BLIP2 captioning via API
pub struct Blip2ApiProvider {
    api_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl Blip2ApiProvider {
    /// Create new BLIP2 API provider
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            api_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Create provider for Hugging Face Inference API
    pub fn huggingface(api_key: String) -> Self {
        Self::new(
            "https://api-inference.huggingface.co/models/Salesforce/blip2-opt-2.7b".to_string(),
            Some(api_key),
        )
    }
}

#[async_trait]
impl CaptionProvider for Blip2ApiProvider {
    fn name(&self) -> &str {
        "blip2-api"
    }

    async fn caption(&self, image: &DynamicImage) -> Result<Caption> {
        // Convert image to bytes
        let mut bytes = Vec::new();
        image.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )?;

        // Build request
        let mut request = self.client.post(&self.api_url);

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        // Send request
        let response = request
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed: {}", response.status());
        }

        // Parse response
        let result: Vec<BlipResponse> = response.json().await?;

        if let Some(first) = result.first() {
            Ok(Caption {
                text: first.generated_text.clone(),
                confidence: 1.0, // API doesn't provide confidence
                provider: "blip2-api".to_string(),
            })
        } else {
            anyhow::bail!("No caption generated")
        }
    }
}

#[derive(Debug, Deserialize)]
struct BlipResponse {
    generated_text: String,
}

/// LLaVA captioning via API
pub struct LlavaApiProvider {
    api_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
    prompt: String,
}

impl LlavaApiProvider {
    /// Create new LLaVA API provider
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            api_url,
            api_key,
            client: reqwest::Client::new(),
            prompt: "Describe this image in detail for AI image generation.".to_string(),
        }
    }

    /// Set custom prompt
    pub fn with_prompt(mut self, prompt: String) -> Self {
        self.prompt = prompt;
        self
    }
}

#[async_trait]
impl CaptionProvider for LlavaApiProvider {
    fn name(&self) -> &str {
        "llava-api"
    }

    async fn caption(&self, image: &DynamicImage) -> Result<Caption> {
        // Convert image to base64
        let mut bytes = Vec::new();
        image.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )?;
        use base64::Engine;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&bytes);

        // Build request JSON
        let request_body = serde_json::json!({
            "model": "llava-v1.5-7b",
            "prompt": self.prompt,
            "images": [base64_image],
        });

        // Build request
        let mut request = self.client.post(&self.api_url);

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        // Send request
        let response = request.json(&request_body).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed: {}", response.status());
        }

        // Parse response
        let result: LlavaResponse = response.json().await?;

        Ok(Caption {
            text: result.response,
            confidence: 1.0,
            provider: "llava-api".to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct LlavaResponse {
    response: String,
}

/// Simple rule-based captioning (fallback)
pub struct SimpleCaptioner {
    default_caption: String,
}

impl SimpleCaptioner {
    pub fn new(default_caption: String) -> Self {
        Self { default_caption }
    }
}

#[async_trait]
impl CaptionProvider for SimpleCaptioner {
    fn name(&self) -> &str {
        "simple"
    }

    async fn caption(&self, _image: &DynamicImage) -> Result<Caption> {
        Ok(Caption {
            text: self.default_caption.clone(),
            confidence: 0.5,
            provider: "simple".to_string(),
        })
    }
}

/// Caption post-processing utilities
pub mod postprocess {
    /// Add trigger word to caption
    pub fn add_trigger_word(caption: &str, trigger: &str) -> String {
        if caption.is_empty() {
            trigger.to_string()
        } else {
            format!("{}, {}", trigger, caption)
        }
    }

    /// Clean up caption (remove extra spaces, lowercase, etc.)
    pub fn clean_caption(caption: &str) -> String {
        caption
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Truncate caption to max length
    pub fn truncate_caption(caption: &str, max_length: usize) -> String {
        if caption.len() <= max_length {
            caption.to_string()
        } else {
            let mut truncated = caption.chars().take(max_length - 3).collect::<String>();
            truncated.push_str("...");
            truncated
        }
    }

    /// Enhance caption with style tags
    pub fn add_style_tags(caption: &str, tags: &[&str]) -> String {
        let tag_str = tags.join(", ");
        format!("{}, {}", caption, tag_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_captioner() {
        let captioner = SimpleCaptioner::new("a photo".to_string());
        let img = DynamicImage::new_rgb8(512, 512);
        let caption = captioner.caption(&img).await.unwrap();

        assert_eq!(caption.text, "a photo");
        assert_eq!(caption.provider, "simple");
    }

    #[test]
    fn test_add_trigger_word() {
        let result = postprocess::add_trigger_word("a beautiful landscape", "myloraname");
        assert_eq!(result, "myloraname, a beautiful landscape");
    }

    #[test]
    fn test_clean_caption() {
        let dirty = "  extra   spaces   here  ";
        let clean = postprocess::clean_caption(dirty);
        assert_eq!(clean, "extra spaces here");
    }

    #[test]
    fn test_truncate_caption() {
        let long = "This is a very long caption that should be truncated";
        let truncated = postprocess::truncate_caption(long, 20);
        assert_eq!(truncated.len(), 20);
        assert!(truncated.ends_with("..."));
    }
}
