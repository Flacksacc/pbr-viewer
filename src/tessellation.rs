//! GPU Tessellation and Displacement Material System

use bevy::prelude::*;
use bevy::pbr::Material;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

/// Tessellation parameters for GPU tessellation
#[derive(Debug, Clone, Copy)]
pub struct TessellationParams {
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
}

impl Default for TessellationParams {
    fn default() -> Self {
        Self {
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
        }
    }
}


/// Custom material with tessellation and displacement support
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TessellationMaterial {
    // Material properties
    #[uniform(0)]
    pub base_color: LinearRgba,
    
    #[uniform(0)]
    pub metallic: f32,
    
    #[uniform(0)]
    pub perceptual_roughness: f32,
    
    #[uniform(0)]
    pub emissive: LinearRgba,
    
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,
    
    #[texture(3)]
    #[sampler(4)]
    pub metallic_roughness_texture: Option<Handle<Image>>,
    
    #[texture(5)]
    #[sampler(6)]
    pub normal_map_texture: Option<Handle<Image>>,
    
    #[texture(7)]
    #[sampler(8)]
    pub occlusion_texture: Option<Handle<Image>>,
    
    #[texture(9)]
    #[sampler(10)]
    pub emissive_texture: Option<Handle<Image>>,
    
    // Tessellation and displacement
    #[uniform(11)]
    pub min_tess_factor: f32,
    #[uniform(11)]
    pub max_tess_factor: f32,
    #[uniform(11)]
    pub displacement_scale: f32,
    #[uniform(11)]
    pub displacement_midpoint: f32,
    #[uniform(11)]
    pub displacement_bias: f32,
    #[uniform(11)]
    pub displacement_clamp_min: f32,
    #[uniform(11)]
    pub displacement_clamp_max: f32,
    #[uniform(11)]
    pub screen_space_scale: f32,
    #[uniform(11)]
    pub distance_scale: f32,
    #[uniform(11)]
    pub quality_cap: f32,
    
    #[texture(12)]
    #[sampler(13)]
    pub height_texture: Option<Handle<Image>>,
    
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
    pub unlit: bool,
}

impl Material for TessellationMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tessellation_material.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/tessellation_material.wgsl".into()
    }

    // Note: Actual hardware tessellation (Hull/Domain shaders) requires:
    // 1. Compiling HLSL/GLSL shaders to SPIR-V
    // 2. Creating a custom render pipeline with tessellation stages
    // 3. Integrating with Bevy's render graph
    // 
    // The current WGSL implementation provides displacement mapping
    // but uses vertex/fragment stages rather than true tessellation.
    // 
    // For shadow pass support, duplicate the tessellation pipeline
    // and apply it to shadow rendering passes.
}

impl Default for TessellationMaterial {
    fn default() -> Self {
        let tess_params = TessellationParams::default();
        Self {
            base_color: LinearRgba::WHITE,
            metallic: 0.0,
            perceptual_roughness: 0.5,
            emissive: LinearRgba::BLACK,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_map_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            min_tess_factor: tess_params.min_tess_factor,
            max_tess_factor: tess_params.max_tess_factor,
            displacement_scale: tess_params.displacement_scale,
            displacement_midpoint: tess_params.displacement_midpoint,
            displacement_bias: tess_params.displacement_bias,
            displacement_clamp_min: tess_params.displacement_clamp_min,
            displacement_clamp_max: tess_params.displacement_clamp_max,
            screen_space_scale: tess_params.screen_space_scale,
            distance_scale: tess_params.distance_scale,
            quality_cap: tess_params.quality_cap,
            height_texture: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            unlit: false,
        }
    }
}

/// Plugin for tessellation material system
pub struct TessellationMaterialPlugin;

impl Plugin for TessellationMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TessellationMaterial>::default());
    }
}

