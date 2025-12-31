//! Application state without Bevy dependencies

use glam::Quat;
use crate::mesh_wgpu::MeshType;

/// View modes for visualizing different texture channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Lit,
    BaseColor,
    Normals,
    Roughness,
    Metallic,
    AO,
    Emissive,
    Height,
}

impl ViewMode {
    pub fn all() -> &'static [ViewMode] {
        &[
            ViewMode::Lit,
            ViewMode::BaseColor,
            ViewMode::Normals,
            ViewMode::Roughness,
            ViewMode::Metallic,
            ViewMode::AO,
            ViewMode::Emissive,
            ViewMode::Height,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ViewMode::Lit => "Lit",
            ViewMode::BaseColor => "Base Color",
            ViewMode::Normals => "Normals",
            ViewMode::Roughness => "Roughness",
            ViewMode::Metallic => "Metallic",
            ViewMode::AO => "AO",
            ViewMode::Emissive => "Emissive",
            ViewMode::Height => "Height",
        }
    }
}

/// Material parameters controlled by sliders
#[derive(Debug, Clone)]
pub struct MaterialParams {
    pub metallic_multiplier: f32,
    pub roughness_multiplier: f32,
    pub normal_strength: f32,
    pub ao_strength: f32,
    pub emissive_strength: f32,
    pub displacement_strength: f32,
    pub base_color_tint: [f32; 3],
    pub uv_scale: f32,
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self {
            metallic_multiplier: 0.0,
            roughness_multiplier: 0.5,
            normal_strength: 1.0,
            ao_strength: 1.0,
            emissive_strength: 0.0,
            displacement_strength: 0.1,
            base_color_tint: [0.8, 0.8, 0.8],
            uv_scale: 1.0,
        }
    }
}

/// GPU Tessellation parameters
#[derive(Debug, Clone)]
pub struct GpuTessellationParams {
    pub enabled: bool,
    pub min_tess_factor: f32,
    pub max_tess_factor: f32,
    pub displacement_scale: f32,
    pub displacement_midpoint: f32,
    pub displacement_bias: f32,
    pub displacement_clamp_min: f32,
    pub displacement_clamp_max: f32,
    pub screen_space_scale: f32,
    pub distance_scale: f32,
    pub quality_cap: f32,
    pub debug_visualization: TessellationDebugMode,
}

impl Default for GpuTessellationParams {
    fn default() -> Self {
        Self {
            enabled: false,
            min_tess_factor: 1.0,
            max_tess_factor: 64.0,
            displacement_scale: 0.1,
            displacement_midpoint: 0.5,
            displacement_bias: 0.0,
            displacement_clamp_min: -1.0,
            displacement_clamp_max: 1.0,
            screen_space_scale: 100.0,
            distance_scale: 0.1,
            quality_cap: 64.0,
            debug_visualization: TessellationDebugMode::None,
        }
    }
}

/// Debug visualization modes for tessellation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TessellationDebugMode {
    None,
    TessellationDensity,
    Wireframe,
    DisplacementOnly,
}

/// Light parameters
#[derive(Debug, Clone)]
pub struct LightParams {
    pub direction: glam::Vec3,
    pub intensity: f32,
    pub color: [f32; 3],
    pub ambient_intensity: f32,
}

impl Default for LightParams {
    fn default() -> Self {
        Self {
            direction: glam::Vec3::new(-1.0, -1.0, -1.0).normalize(),
            intensity: 15.0,
            color: [1.0, 1.0, 1.0],
            ambient_intensity: 0.4,
        }
    }
}

/// Texture handles (using paths for now, will load into wgpu later)
#[derive(Debug, Clone, Default)]
pub struct TextureHandles {
    pub base_color: Option<String>,
    pub normal: Option<String>,
    pub roughness: Option<String>,
    pub metallic: Option<String>,
    pub orm: Option<String>,
    pub ao: Option<String>,
    pub emissive: Option<String>,
    pub height: Option<String>,
}

impl TextureHandles {
    pub fn get_file_name(&self, texture_type: &str) -> Option<String> {
        let path = match texture_type {
            "base_color" => &self.base_color,
            "normal" => &self.normal,
            "roughness" => &self.roughness,
            "metallic" => &self.metallic,
            "orm" => &self.orm,
            "ao" => &self.ao,
            "emissive" => &self.emissive,
            "height" => &self.height,
            _ => return None,
        };
        
        path.as_ref().and_then(|p| {
            std::path::Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
    }
}

/// Tracks which textures have been loaded
#[derive(Debug, Clone, Default)]
pub struct LoadedTextures {
    pub base_color: bool,
    pub normal: bool,
    pub roughness: bool,
    pub metallic: bool,
    pub orm: bool,
    pub ao: bool,
    pub emissive: bool,
    pub height: bool,
}

impl LoadedTextures {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Main application state
pub struct AppState {
    // Current settings
    pub current_mesh: MeshType,
    pub view_mode: ViewMode,
    pub material_params: MaterialParams,
    pub light_params: LightParams,
    
    // CPU Tessellation (for mesh generation)
    pub tessellation_level: u32,
    
    // GPU Tessellation parameters
    pub gpu_tessellation: GpuTessellationParams,
    
    // Texture folder
    pub texture_folder: Option<String>,
    
    // Loaded texture info
    pub loaded_textures: LoadedTextures,
    
    // Texture handles (paths)
    pub texture_handles: TextureHandles,
    
    // Model rotation
    pub model_rotation: Quat,
    pub is_rotating_model: bool,
    
    // Change flags
    pub mesh_changed: bool,
    pub material_changed: bool,
    pub textures_need_reload: bool,
    
    // Drag and drop hover state
    pub drag_hover_path: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_mesh: MeshType::Sphere,
            view_mode: ViewMode::Lit,
            material_params: MaterialParams::default(),
            light_params: LightParams::default(),
            tessellation_level: 32,
            gpu_tessellation: GpuTessellationParams::default(),
            texture_folder: None,
            loaded_textures: LoadedTextures::default(),
            texture_handles: TextureHandles::default(),
            model_rotation: Quat::IDENTITY,
            is_rotating_model: false,
            mesh_changed: false,
            material_changed: false,
            textures_need_reload: false,
            drag_hover_path: None,
        }
    }
}

