//! Application state and resources

use bevy::prelude::*;
use bevy::asset::Handle;
use bevy::gltf::Gltf;
use crate::mesh::MeshType;

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

/// Light parameters
#[derive(Debug, Clone)]
pub struct LightParams {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: [f32; 3],
    pub ambient_intensity: f32,
}

impl Default for LightParams {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-1.0, -1.0, -1.0).normalize(),
            intensity: 15.0,
            color: [1.0, 1.0, 1.0],
            ambient_intensity: 0.4,
        }
    }
}

/// Stored texture handles for view mode switching
#[derive(Debug, Clone, Default)]
pub struct TextureHandles {
    pub base_color: Option<Handle<Image>>,
    pub normal: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
    pub metallic: Option<Handle<Image>>,
    pub orm: Option<Handle<Image>>,
    pub ao: Option<Handle<Image>>,
    pub emissive: Option<Handle<Image>>,
    pub height: Option<Handle<Image>>,
}

/// Main application state resource
#[derive(Resource)]
pub struct AppState {
    // Current settings
    pub current_mesh: MeshType,
    pub view_mode: ViewMode,
    pub material_params: MaterialParams,
    pub light_params: LightParams,
    
    // Tessellation
    pub tessellation_level: u32,
    
    // Texture folder
    pub texture_folder: Option<String>,
    
    // Loaded texture info (booleans for UI)
    pub loaded_textures: LoadedTextures,
    
    // Actual texture handles
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
    
    // Asset handles
    pub mesh_handle: Option<Handle<Mesh>>,
    pub material_handle: Option<Handle<StandardMaterial>>,
    
    // Custom model loading
    pub custom_model_path: Option<String>,
    pub custom_model_handle: Option<Handle<Gltf>>,
    pub using_custom_model: bool,
    pub custom_model_needs_load: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_mesh: MeshType::Sphere,
            view_mode: ViewMode::Lit,
            material_params: MaterialParams::default(),
            light_params: LightParams::default(),
            tessellation_level: 32,
            texture_folder: None,
            loaded_textures: LoadedTextures::default(),
            texture_handles: TextureHandles::default(),
            model_rotation: Quat::IDENTITY,
            is_rotating_model: false,
            mesh_changed: false,
            material_changed: false,
            textures_need_reload: false,
            drag_hover_path: None,
            mesh_handle: None,
            material_handle: None,
            custom_model_path: None,
            custom_model_handle: None,
            using_custom_model: false,
            custom_model_needs_load: false,
        }
    }
}

/// Tracks which textures have been loaded (for UI display)
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
