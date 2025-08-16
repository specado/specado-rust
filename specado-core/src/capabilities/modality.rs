//! Multimodal support definitions for LLM providers

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Supported modality types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
    Document,  // PDFs, Word docs, etc.
    Code,      // Specialized code understanding
    Custom(String),
}

/// Modality support configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModalitySupport {
    /// Input modalities the provider accepts
    pub input: HashSet<Modality>,
    
    /// Output modalities the provider can generate
    pub output: HashSet<Modality>,
    
    /// Modality-specific configurations
    pub configs: ModalityConfigs,
}

/// Configuration for specific modalities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModalityConfigs {
    /// Image modality configuration
    pub image: Option<ImageConfig>,
    
    /// Audio modality configuration
    pub audio: Option<AudioConfig>,
    
    /// Video modality configuration
    pub video: Option<VideoConfig>,
    
    /// Document modality configuration
    pub document: Option<DocumentConfig>,
}

/// Image modality configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageConfig {
    /// Supported image formats
    pub formats: HashSet<String>,  // ["jpeg", "png", "gif", "webp"]
    
    /// Maximum image dimensions
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    
    /// Maximum file size in bytes
    pub max_size_bytes: Option<u64>,
    
    /// Maximum number of images per request
    pub max_images_per_request: Option<u32>,
    
    /// Support for image generation dimensions
    pub generation_sizes: Option<HashSet<String>>,  // ["256x256", "512x512", "1024x1024"]
}

/// Audio modality configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioConfig {
    /// Supported audio formats
    pub formats: HashSet<String>,  // ["mp3", "wav", "m4a", "ogg"]
    
    /// Maximum audio duration in seconds
    pub max_duration_seconds: Option<u32>,
    
    /// Maximum file size in bytes
    pub max_size_bytes: Option<u64>,
    
    /// Supported languages for transcription
    pub transcription_languages: Option<HashSet<String>>,
    
    /// Support for voice selection in generation
    pub voice_options: Option<HashSet<String>>,
}

/// Video modality configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoConfig {
    /// Supported video formats
    pub formats: HashSet<String>,  // ["mp4", "avi", "mov", "webm"]
    
    /// Maximum video duration in seconds
    pub max_duration_seconds: Option<u32>,
    
    /// Maximum file size in bytes
    pub max_size_bytes: Option<u64>,
    
    /// Frame extraction support
    pub frame_extraction: bool,
}

/// Document modality configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentConfig {
    /// Supported document formats
    pub formats: HashSet<String>,  // ["pdf", "docx", "txt", "html", "md"]
    
    /// Maximum file size in bytes
    pub max_size_bytes: Option<u64>,
    
    /// Maximum pages for PDFs
    pub max_pages: Option<u32>,
    
    /// OCR support for scanned documents
    pub ocr_support: bool,
}

impl Default for ModalitySupport {
    fn default() -> Self {
        let mut input = HashSet::new();
        input.insert(Modality::Text);
        
        let mut output = HashSet::new();
        output.insert(Modality::Text);
        
        Self {
            input,
            output,
            configs: ModalityConfigs::default(),
        }
    }
}

impl Default for ModalityConfigs {
    fn default() -> Self {
        Self {
            image: None,
            audio: None,
            video: None,
            document: None,
        }
    }
}

impl ModalitySupport {
    /// Create text-only modality support
    pub fn text_only() -> Self {
        Self::default()
    }
    
    /// Create multimodal support with text and images
    pub fn text_and_image() -> Self {
        let mut support = Self::default();
        support.input.insert(Modality::Image);
        support.configs.image = Some(ImageConfig::default());
        support
    }
    
    /// Check if a specific input modality is supported
    pub fn supports_input(&self, modality: &Modality) -> bool {
        self.input.contains(modality)
    }
    
    /// Check if a specific output modality is supported
    pub fn supports_output(&self, modality: &Modality) -> bool {
        self.output.contains(modality)
    }
    
    /// Get all supported input formats for a modality
    pub fn get_input_formats(&self, modality: &Modality) -> Option<HashSet<String>> {
        match modality {
            Modality::Image => self.configs.image.as_ref().map(|c| c.formats.clone()),
            Modality::Audio => self.configs.audio.as_ref().map(|c| c.formats.clone()),
            Modality::Video => self.configs.video.as_ref().map(|c| c.formats.clone()),
            Modality::Document => self.configs.document.as_ref().map(|c| c.formats.clone()),
            _ => None,
        }
    }
}

impl Default for ImageConfig {
    fn default() -> Self {
        let mut formats = HashSet::new();
        formats.insert("jpeg".to_string());
        formats.insert("jpg".to_string());
        formats.insert("png".to_string());
        formats.insert("gif".to_string());
        formats.insert("webp".to_string());
        
        Self {
            formats,
            max_width: Some(4096),
            max_height: Some(4096),
            max_size_bytes: Some(20 * 1024 * 1024),  // 20MB
            max_images_per_request: Some(10),
            generation_sizes: None,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        let mut formats = HashSet::new();
        formats.insert("mp3".to_string());
        formats.insert("wav".to_string());
        formats.insert("m4a".to_string());
        formats.insert("ogg".to_string());
        
        Self {
            formats,
            max_duration_seconds: Some(600),  // 10 minutes
            max_size_bytes: Some(25 * 1024 * 1024),  // 25MB
            transcription_languages: None,
            voice_options: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_modality_support_default() {
        let support = ModalitySupport::default();
        assert!(support.supports_input(&Modality::Text));
        assert!(support.supports_output(&Modality::Text));
        assert!(!support.supports_input(&Modality::Image));
    }
    
    #[test]
    fn test_text_and_image_support() {
        let support = ModalitySupport::text_and_image();
        assert!(support.supports_input(&Modality::Text));
        assert!(support.supports_input(&Modality::Image));
        assert!(support.supports_output(&Modality::Text));
        assert!(!support.supports_output(&Modality::Image));
        
        let formats = support.get_input_formats(&Modality::Image);
        assert!(formats.is_some());
        let formats = formats.unwrap();
        assert!(formats.contains("jpeg"));
        assert!(formats.contains("png"));
    }
    
    #[test]
    fn test_custom_modality() {
        let mut support = ModalitySupport::default();
        support.input.insert(Modality::Custom("3D".to_string()));
        assert!(support.supports_input(&Modality::Custom("3D".to_string())));
        assert!(!support.supports_input(&Modality::Custom("4D".to_string())));
    }
}