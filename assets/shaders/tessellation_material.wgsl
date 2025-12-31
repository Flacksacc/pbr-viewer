#import bevy_pbr::{
    mesh_view_bindings::view,
    mesh_bindings::mesh,
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, STANDARD_MATERIAL_FLAGS_UNLIT_BIT},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_bindings::pbr_input,
    utils::tone_mapping::tone_mapping,
}

// Material uniform
struct MaterialUniform {
    base_color: vec4<f32>,
    metallic: f32,
    perceptual_roughness: f32,
    emissive: vec4<f32>,
    _padding: vec3<f32>,
}

// Tessellation parameters uniform
struct TessellationUniform {
    min_tess_factor: f32,
    max_tess_factor: f32,
    displacement_scale: f32,
    displacement_midpoint: f32,
    displacement_bias: f32,
    displacement_clamp_min: f32,
    displacement_clamp_max: f32,
    screen_space_scale: f32,
    distance_scale: f32,
    quality_cap: f32,
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: MaterialUniform;

@group(2) @binding(11)
var<uniform> tess_params: TessellationUniform;

@group(2) @binding(12)
var height_texture: texture_2d<f32>;
@group(2) @binding(13)
var height_sampler: sampler;

// Fragment shader with PBR lighting
// Note: True vertex displacement requires HLSL domain shaders
// This implementation uses fragment-based effects for now
@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    
    // Apply standard PBR lighting
    output = apply_pbr_lighting(
        pbr_input,
        in.world_position,
        in.world_normal,
        in.view_position,
        in.is_front,
    );
    
    return output;
}
