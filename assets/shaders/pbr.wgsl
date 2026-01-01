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
    view_mode: u32,
    texture_flags: u32,
    light_direction: vec3<f32>,
    _padding1: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_pos = model * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = view_proj * world_pos;
    
    // Transform normal to world space
    // For rotation-only matrices, we simply multiply by the rotation matrix
    // Extract upper-left 3x3 from model matrix (the rotation part)
    let rotation = mat3x3<f32>(
        model[0].xyz,
        model[1].xyz,
        model[2].xyz,
    );
    
    // Transform normal by the rotation matrix to get world-space normal
    // This makes the normal rotate WITH the model, keeping light fixed in world space
    out.world_normal = normalize(rotation * in.normal);
    
    // Apply UV tiling with center pivot
    // The UI "scale" is tile size: smaller scale = more repeats (finer pattern)
    // So tiling = 1/scale. Scale around center (0.5, 0.5) to prevent drift
    let epsilon = 0.0001;
    let tiling = 1.0 / max(material_params.uv_scale, epsilon);
    let pivot = vec2<f32>(0.5, 0.5);
    out.uv = (in.uv - pivot) * tiling + pivot;
    
    // Transform tangent to world space
    let T = normalize(rotation * in.tangent.xyz);
    let N = out.world_normal;
    // Recalculate bitangent to ensure orthogonality
    let B = cross(N, T) * in.tangent.w;
    
    out.tangent = T;
    out.bitangent = normalize(B);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Check which textures are available
    let has_base_color = (material_params.texture_flags & (1u << 0u)) != 0u;
    let has_normal = (material_params.texture_flags & (1u << 1u)) != 0u;
    let has_metallic_roughness = (material_params.texture_flags & (1u << 2u)) != 0u;
    let has_ao = (material_params.texture_flags & (1u << 3u)) != 0u;
    let has_emissive = (material_params.texture_flags & (1u << 4u)) != 0u;
    let has_height = (material_params.texture_flags & (1u << 5u)) != 0u;
    
    // Sample textures only if they exist, otherwise use defaults
    // For non-lit view modes, we'll sample raw textures without processing
    var base_color_sample: vec4<f32>;
    if has_base_color {
        base_color_sample = textureSample(base_color_texture, base_color_sampler, in.uv);
    } else {
        base_color_sample = vec4<f32>(material_params.base_color_tint, 1.0);
    }
    
    var normal_sample: vec4<f32>;
    if has_normal {
        normal_sample = textureSample(normal_texture, normal_sampler, in.uv);
    } else {
        normal_sample = vec4<f32>(0.5, 0.5, 1.0, 1.0);  // Default flat normal
    }
    
    var metallic_roughness: vec4<f32>;
    if has_metallic_roughness {
        metallic_roughness = textureSample(metallic_roughness_texture, metallic_roughness_sampler, in.uv);
    } else {
        metallic_roughness = vec4<f32>(0.0, 0.5, 0.0, 1.0);  // Default: no metallic, medium roughness
    }
    
    // Handle different view modes
    let view_mode = material_params.view_mode;
    
    if view_mode == 0u {  // Lit
        // Apply processing for lit mode
        let base_color = base_color_sample.rgb * material_params.base_color_tint;
        let metallic = metallic_roughness.b * material_params.metallic;
        let roughness = metallic_roughness.g * material_params.roughness;
        
        // Simple lighting (directional light)
        let light_dir = normalize(material_params.light_direction);
        let N = normalize(in.world_normal);
        let NDotL = max(dot(N, light_dir), 0.0);
        let color = base_color * (0.3 + 0.7 * NDotL);
        return vec4<f32>(color, 1.0);
    } else if view_mode == 1u {  // BaseColor - show raw texture
        if has_base_color {
            return base_color_sample;  // Raw texture, no tint applied
        } else {
            return vec4<f32>(0.5, 0.5, 0.5, 1.0);  // Gray if no texture
        }
    } else if view_mode == 2u {  // Normals - show raw texture
        if has_normal {
            return normal_sample;  // Raw normal map texture
        } else {
            // Show geometry normals if no normal map
            let N = normalize(in.world_normal);
            return vec4<f32>(N * 0.5 + 0.5, 1.0);
        }
    } else if view_mode == 3u {  // Roughness - show raw texture channel
        if has_metallic_roughness {
            // Show raw roughness channel (green channel) as grayscale
            return vec4<f32>(vec3<f32>(metallic_roughness.g), 1.0);
        } else {
            return vec4<f32>(0.5, 0.5, 0.5, 1.0);  // Gray if no texture
        }
    } else if view_mode == 4u {  // Metallic - show raw texture channel
        if has_metallic_roughness {
            // Show raw metallic channel (blue channel) as grayscale
            return vec4<f32>(vec3<f32>(metallic_roughness.b), 1.0);
        } else {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // Black if no texture
        }
    } else if view_mode == 5u {  // AO - show raw texture channel
        if has_metallic_roughness {
            // AO is typically in the alpha channel of metallic_roughness or ORM texture
            // Show raw alpha channel as grayscale
            return vec4<f32>(vec3<f32>(metallic_roughness.a), 1.0);
        } else {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);  // White if no texture
        }
    } else if view_mode == 6u {  // Emissive - show raw texture
        // For now, emissive would need a separate texture binding
        // If we had it, we'd show it here as raw texture
        // For now, try to use base_color as fallback or show black
        if has_base_color {
            return base_color_sample;  // Fallback to base color
        } else {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // Black if no texture
        }
    } else if view_mode == 7u {  // Height - show raw texture
        // Height would typically be in a separate texture
        // For now, try to use metallic_roughness as fallback (some formats put height in red channel)
        // Or we could use normal map as fallback
        if has_metallic_roughness {
            // Try red channel as height (common in some texture formats)
            return vec4<f32>(vec3<f32>(metallic_roughness.r), 1.0);
        } else if has_normal {
            // Fallback to normal map (height maps are often similar to normal maps)
            return vec4<f32>(normal_sample.rgb, 1.0);
        } else {
            return vec4<f32>(0.5, 0.5, 0.5, 1.0);  // Gray if no texture
        }
    }
    
    // Fallback - return base color sample
    return base_color_sample;
}

