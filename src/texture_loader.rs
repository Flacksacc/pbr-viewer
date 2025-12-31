//! Texture loading from files

use std::path::{Path, PathBuf};
use wgpu::*;
use crate::texture;

/// Texture detection patterns for different texture types
pub struct TexturePatterns;

impl TexturePatterns {
    /// Check if a filename matches base color patterns
    pub fn is_base_color(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("basecolor") || 
        name_lower.contains("base_color") ||
        name_lower.contains("basecolor") ||
        name_lower.contains("diffuse") ||
        name_lower.contains("albedo") ||
        name_lower.contains("color") ||
        name_lower.contains("col")
    }
    
    /// Check if a filename matches normal map patterns
    pub fn is_normal(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("normal") ||
        name_lower.contains("norm") ||
        name_lower.contains("nrm")
    }
    
    /// Check if a filename matches metallic patterns
    pub fn is_metallic(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        (name_lower.contains("metallic") || name_lower.contains("metal")) &&
        !name_lower.contains("roughness") &&
        !name_lower.contains("rough")
    }
    
    /// Check if a filename matches roughness patterns
    pub fn is_roughness(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        (name_lower.contains("roughness") || name_lower.contains("rough")) &&
        !name_lower.contains("metallic") &&
        !name_lower.contains("metal")
    }
    
    /// Check if a filename matches metallic+roughness combined patterns
    pub fn is_metallic_roughness(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        // Exclude if it's clearly a different texture type
        if name_lower.contains("normal") || 
           name_lower.contains("basecolor") || 
           name_lower.contains("base_color") ||
           name_lower.contains("diffuse") ||
           name_lower.contains("albedo") ||
           name_lower.contains("emissive") ||
           name_lower.contains("height") ||
           name_lower.contains("displacement") ||
           name_lower.contains("orm") {
            return false;
        }
        // Match metallic+roughness patterns
        (name_lower.contains("metallic") || name_lower.contains("metal")) &&
        (name_lower.contains("roughness") || name_lower.contains("rough"))
    }
    
    /// Check if a filename matches ORM (Occlusion-Roughness-Metallic) patterns
    /// ORM is a combined texture, so we exclude files that are clearly other texture types
    pub fn is_orm(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        // Exclude if it's clearly a different texture type
        if name_lower.contains("normal") || 
           name_lower.contains("basecolor") || 
           name_lower.contains("base_color") ||
           name_lower.contains("diffuse") ||
           name_lower.contains("albedo") ||
           name_lower.contains("emissive") ||
           name_lower.contains("height") ||
           name_lower.contains("displacement") {
            return false;
        }
        // Match ORM patterns
        name_lower.contains("orm") ||
        (name_lower.contains("ao") && name_lower.contains("roughness") && name_lower.contains("metallic")) ||
        (name_lower.contains("occlusion") && name_lower.contains("roughness") && name_lower.contains("metallic"))
    }
    
    /// Check if a filename matches AO patterns
    pub fn is_ao(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        (name_lower.contains("ao") || name_lower.contains("occlusion") || name_lower.contains("ambient")) &&
        !name_lower.contains("roughness") &&
        !name_lower.contains("metallic")
    }
    
    /// Check if a filename matches emissive patterns
    pub fn is_emissive(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("emissive") ||
        name_lower.contains("emission") ||
        name_lower.contains("emiss") ||
        name_lower.contains("glow")
    }
    
    /// Check if a filename matches height/displacement patterns
    pub fn is_height(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("height") ||
        name_lower.contains("displacement") ||
        name_lower.contains("depth") ||
        name_lower.contains("tessellation") ||
        name_lower.contains("tess") ||
        name_lower.contains("bump") ||
        name_lower.contains("disp")
    }
}

/// Detect texture type from filename and return detected paths
pub fn detect_textures_in_directory(dir_path: &Path) -> Result<TexturePaths, anyhow::Error> {
    let entries = std::fs::read_dir(dir_path)?;
    let mut paths = TexturePaths::default();
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            
            // Check each texture type (order matters - check specific textures first, then combined)
            // Specific textures should be checked before combined textures to avoid misclassification
            if TexturePatterns::is_normal(file_name) {
                paths.normal = Some(path.clone());
            } else if TexturePatterns::is_base_color(file_name) {
                paths.base_color = Some(path.clone());
            } else if TexturePatterns::is_emissive(file_name) {
                paths.emissive = Some(path.clone());
            } else if TexturePatterns::is_height(file_name) {
                paths.height = Some(path.clone());
            } else if TexturePatterns::is_orm(file_name) {
                // ORM is a combined texture (Occlusion-Roughness-Metallic)
                paths.orm = Some(path.clone());
            } else if TexturePatterns::is_metallic_roughness(file_name) {
                // Metallic+Roughness combined texture
                paths.metallic_roughness = Some(path.clone());
            } else if TexturePatterns::is_metallic(file_name) {
                paths.metallic = Some(path.clone());
            } else if TexturePatterns::is_roughness(file_name) {
                paths.roughness = Some(path.clone());
            } else if TexturePatterns::is_ao(file_name) {
                paths.ao = Some(path.clone());
            }
        }
    }
    
    Ok(paths)
}

/// Detected texture file paths
#[derive(Debug, Clone, Default)]
pub struct TexturePaths {
    pub base_color: Option<PathBuf>,
    pub normal: Option<PathBuf>,
    pub metallic: Option<PathBuf>,
    pub roughness: Option<PathBuf>,
    pub metallic_roughness: Option<PathBuf>,
    pub orm: Option<PathBuf>,
    pub ao: Option<PathBuf>,
    pub emissive: Option<PathBuf>,
    pub height: Option<PathBuf>,
}

/// Load textures from a directory
pub struct TextureLoader;

impl TextureLoader {
    /// Load a single texture from a file path
    pub fn load_texture_file(
        device: &Device,
        queue: &Queue,
        path: &Path,
        label: Option<&str>,
    ) -> Result<(Texture, TextureView, Sampler), anyhow::Error> {
        let bytes = std::fs::read(path)?;
        let texture = texture::load_texture(device, queue, &bytes, label)?;
        Ok(texture)
    }
    
    /// Search for texture files in a directory and load them
    #[allow(dead_code)]
    pub fn load_from_directory(
        device: &Device,
        queue: &Queue,
        dir_path: &Path,
    ) -> Result<TextureSet, anyhow::Error> {
        let paths = detect_textures_in_directory(dir_path)?;
        
        // Load textures (use placeholder if not found)
        let base_color = if let Some(path) = &paths.base_color {
            Self::load_texture_file(device, queue, path, Some("base_color"))?
        } else {
            texture::create_placeholder_texture(device, queue, [128, 128, 128, 255], Some("base_color_placeholder"))
        };
        
        let normal = if let Some(path) = &paths.normal {
            Self::load_texture_file(device, queue, path, Some("normal"))?
        } else {
            texture::create_placeholder_texture(device, queue, [128, 128, 255, 255], Some("normal_placeholder"))
        };
        
        // Prefer ORM, then metallic_roughness, then separate metallic/roughness
        let metallic_roughness = if let Some(path) = &paths.orm {
            Self::load_texture_file(device, queue, path, Some("orm"))?
        } else if let Some(path) = &paths.metallic_roughness {
            Self::load_texture_file(device, queue, path, Some("metallic_roughness"))?
        } else {
            texture::create_placeholder_texture(device, queue, [0, 128, 0, 255], Some("metallic_roughness_placeholder"))
        };
        
        Ok(TextureSet {
            base_color,
            normal,
            metallic_roughness,
        })
    }
    
    /// Load textures from individual file paths (allows manual selection)
    pub fn load_from_paths(
        device: &Device,
        queue: &Queue,
        paths: &TexturePaths,
    ) -> Result<TextureSet, anyhow::Error> {
        let base_color = if let Some(path) = &paths.base_color {
            Self::load_texture_file(device, queue, path, Some("base_color"))?
        } else {
            texture::create_placeholder_texture(device, queue, [128, 128, 128, 255], Some("base_color_placeholder"))
        };
        
        let normal = if let Some(path) = &paths.normal {
            Self::load_texture_file(device, queue, path, Some("normal"))?
        } else {
            texture::create_placeholder_texture(device, queue, [128, 128, 255, 255], Some("normal_placeholder"))
        };
        
        // Prefer ORM, then metallic_roughness
        let metallic_roughness = if let Some(path) = &paths.orm {
            Self::load_texture_file(device, queue, path, Some("orm"))?
        } else if let Some(path) = &paths.metallic_roughness {
            Self::load_texture_file(device, queue, path, Some("metallic_roughness"))?
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

