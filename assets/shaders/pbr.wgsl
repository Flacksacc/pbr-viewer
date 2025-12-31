//! PBR Shader for wgpu

// Vertex shader inputs
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

// Vertex shader outputs
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

// Uniforms
@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> model: mat4x4<f32>;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(1)
var base_color_sampler: sampler;

@group(1) @binding(2)
var normal_texture: texture_2d<f32>;
@group(1) @binding(3)
var normal_sampler: sampler;

@group(1) @binding(4)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(5)
var metallic_roughness_sampler: sampler;

@group(2) @binding(0)
var<uniform> material_params: MaterialParams;

struct MaterialParams {
    base_color_tint: vec3<f32>,
    _padding0: f32,
    metallic: f32,
    roughness: f32,
    normal_strength: f32,
    uv_scale: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_pos = model * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = view_proj * world_pos;
    
    // Calculate proper normal matrix (inverse transpose of upper-left 3x3 of model matrix)
    // For rotation matrices, transpose equals inverse, so transpose is correct
    // Extract upper-left 3x3 from model matrix
    let m = mat3x3<f32>(
        model[0].xyz,
        model[1].xyz,
        model[2].xyz,
    );
    
    // For rotation matrices: transpose = inverse, so normal_matrix = transpose(m)
    // For general matrices, we'd need inverse(transpose(m)), but for pure rotations this is correct
    let normal_matrix = transpose(m);
    
    // Transform normal to world space
    out.world_normal = normalize(normal_matrix * in.normal);
    
    // Apply UV tiling with center pivot
    // The UI "scale" is tile size: smaller scale = more repeats (finer pattern)
    // So tiling = 1/scale. Scale around center (0.5, 0.5) to prevent drift
    let epsilon = 0.0001;
    let tiling = 1.0 / max(material_params.uv_scale, epsilon);
    let pivot = vec2<f32>(0.5, 0.5);
    out.uv = (in.uv - pivot) * tiling + pivot;
    
    // Transform tangent to world space
    let T = normalize(normal_matrix * in.tangent.xyz);
    let N = out.world_normal;
    // Recalculate bitangent to ensure orthogonality
    let B = cross(N, T) * in.tangent.w;
    
    out.tangent = T;
    out.bitangent = normalize(B);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    let base_color_sample = textureSample(base_color_texture, base_color_sampler, in.uv);
    let base_color = base_color_sample.rgb * material_params.base_color_tint;
    
    let normal_sample = textureSample(normal_texture, normal_sampler, in.uv);
    let metallic_roughness = textureSample(metallic_roughness_texture, metallic_roughness_sampler, in.uv);
    
    // Simple PBR (simplified for now)
    let metallic = metallic_roughness.b * material_params.metallic;
    let roughness = metallic_roughness.g * material_params.roughness;
    
    // Simple lighting (directional light)
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let N = normalize(in.world_normal);
    let NDotL = max(dot(N, light_dir), 0.0);
    
    let color = base_color * (0.3 + 0.7 * NDotL); // Simple ambient + diffuse
    
    return vec4<f32>(color, 1.0);
}

