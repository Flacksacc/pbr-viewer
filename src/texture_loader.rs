//! Texture loading from files

use std::path::Path;
use wgpu::*;
use crate::texture;

/// Load textures from a directory
pub struct TextureLoader;

impl TextureLoader {
    /// Search for texture files in a directory and load them
    pub fn load_from_directory(
        device: &Device,
        queue: &Queue,
        dir_path: &Path,
    ) -> Result<TextureSet, anyhow::Error> {
        let entries = std::fs::read_dir(dir_path)?;
        
        let mut base_color_path: Option<std::path::PathBuf> = None;
        let mut normal_path: Option<std::path::PathBuf> = None;
        let mut metallic_roughness_path: Option<std::path::PathBuf> = None;
        let mut orm_path: Option<std::path::PathBuf> = None;
        
        // Search for texture files
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name_lower = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();
                
                // Check for common texture naming patterns
                if file_name_lower.contains("basecolor") || 
                   file_name_lower.contains("base_color") ||
                   file_name_lower.contains("diffuse") ||
                   file_name_lower.contains("albedo") {
                    base_color_path = Some(path);
                } else if file_name_lower.contains("normal") {
                    normal_path = Some(path);
                } else if file_name_lower.contains("metallic") && file_name_lower.contains("roughness") {
                    metallic_roughness_path = Some(path);
                } else if file_name_lower.contains("orm") ||
                          (file_name_lower.contains("ao") && file_name_lower.contains("roughness") && file_name_lower.contains("metallic")) {
                    orm_path = Some(path);
                }
            }
        }
        
        // Load textures (use placeholder if not found)
        let base_color = if let Some(path) = base_color_path {
            let bytes = std::fs::read(&path)?;
            texture::load_texture(device, queue, &bytes, Some("base_color"))?
        } else {
            texture::create_placeholder_texture(device, queue, [128, 128, 128, 255], Some("base_color_placeholder"))
        };
        
        let normal = if let Some(path) = normal_path {
            let bytes = std::fs::read(&path)?;
            texture::load_texture(device, queue, &bytes, Some("normal"))?
        } else {
            texture::create_placeholder_texture(device, queue, [128, 128, 255, 255], Some("normal_placeholder"))
        };
        
        let metallic_roughness = if let Some(path) = metallic_roughness_path.or(orm_path) {
            let bytes = std::fs::read(&path)?;
            texture::load_texture(device, queue, &bytes, Some("metallic_roughness"))?
        } else {
            texture::create_placeholder_texture(device, queue, [0, 128, 0, 255], Some("metallic_roughness_placeholder"))
        };
        
        Ok(TextureSet {
            base_color,
            normal,
            metallic_roughness,
        })
    }
}

use crate::texture_manager::TextureSet;

