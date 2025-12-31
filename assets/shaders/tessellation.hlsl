// HLSL Tessellation Shaders for DX12
// Vertex Shader
struct VertexInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 tangent : TANGENT;
};

struct VertexOutput {
    float3 worldPos : TEXCOORD0;
    float3 worldNormal : TEXCOORD1;
    float2 uv : TEXCOORD2;
    float4 position : SV_POSITION;
};

cbuffer Transform : register(b0) {
    float4x4 model;
    float4x4 view_proj;
};

VertexOutput vs_main(VertexInput input) {
    VertexOutput output;
    float4 worldPos = mul(model, float4(input.position, 1.0));
    output.worldPos = worldPos.xyz;
    output.worldNormal = mul((float3x3)model, input.normal);
    output.uv = input.uv;
    output.position = mul(view_proj, worldPos);
    return output;
}

// Hull Shader (Tessellation Control)
struct HullInput {
    float3 worldPos : TEXCOORD0;
    float3 worldNormal : TEXCOORD1;
    float2 uv : TEXCOORD2;
    float4 position : SV_POSITION;
};

struct HullOutput {
    float3 worldPos : TEXCOORD0;
    float3 worldNormal : TEXCOORD1;
    float2 uv : TEXCOORD2;
};

struct PatchConstantOutput {
    float edges[3] : SV_TessFactor;
    float inside : SV_InsideTessFactor;
};

cbuffer TessellationParams : register(b1) {
    float min_tess_factor;
    float max_tess_factor;
    float screen_space_scale;
    float distance_scale;
    float quality_cap;
    float3 _padding;
};

// Calculate screen-space tessellation factor
float CalculateTessFactor(float3 worldPos, float3 viewPos, float edgeLength) {
    // Screen-space metric
    float screenEdge = edgeLength * screen_space_scale / max(abs(viewPos.z), 0.1);
    
    // Distance-based falloff
    float distanceFactor = 1.0 / (1.0 + length(viewPos) * distance_scale);
    
    // Quantize to prevent cracks (8 levels)
    float rawFactor = screenEdge * distanceFactor;
    float quantized = floor(rawFactor * 8.0) / 8.0;
    
    return clamp(quantized * max_tess_factor, min_tess_factor, quality_cap);
}

[domain("tri")]
[partitioning("fractional_odd")]
[outputtopology("triangle_cw")]
[outputcontrolpoints(3)]
[patchconstantfunc("hull_constant")]
HullOutput hs_main(InputPatch<HullInput, 3> patch, uint id : SV_OutputControlPointID) {
    HullOutput output;
    output.worldPos = patch[id].worldPos;
    output.worldNormal = patch[id].worldNormal;
    output.uv = patch[id].uv;
    return output;
}

PatchConstantOutput hull_constant(InputPatch<HullInput, 3> patch) {
    PatchConstantOutput output;
    
    // Calculate edge lengths in world space
    float edge0 = length(patch[1].worldPos - patch[0].worldPos);
    float edge1 = length(patch[2].worldPos - patch[1].worldPos);
    float edge2 = length(patch[0].worldPos - patch[2].worldPos);
    
    // Average position for view space calculation
    float3 avgPos = (patch[0].worldPos + patch[1].worldPos + patch[2].worldPos) / 3.0;
    float4 viewPos = mul(view_proj, float4(avgPos, 1.0));
    
    // Calculate tessellation factors for each edge
    output.edges[0] = CalculateTessFactor(patch[0].worldPos, viewPos.xyz, edge0);
    output.edges[1] = CalculateTessFactor(patch[1].worldPos, viewPos.xyz, edge1);
    output.edges[2] = CalculateTessFactor(patch[2].worldPos, viewPos.xyz, edge2);
    
    // Inside tessellation factor (average of edges)
    output.inside = (output.edges[0] + output.edges[1] + output.edges[2]) / 3.0;
    
    return output;
}

// Domain Shader (Tessellation Evaluation)
struct DomainOutput {
    float3 worldPos : TEXCOORD0;
    float3 worldNormal : TEXCOORD1;
    float2 uv : TEXCOORD2;
    float4 position : SV_POSITION;
};

Texture2D heightTexture : register(t0);
SamplerState heightSampler : register(s0);

cbuffer DisplacementParams : register(b2) {
    float displacement_scale;
    float displacement_midpoint;
    float displacement_bias;
    float displacement_clamp_min;
    float displacement_clamp_max;
    float3 _padding2;
};

[domain("tri")]
DomainOutput ds_main(
    PatchConstantOutput input,
    float3 baryCoords : SV_DomainLocation,
    const OutputPatch<HullOutput, 3> patch
) {
    DomainOutput output;
    
    // Interpolate vertex attributes
    float3 worldPos = baryCoords.x * patch[0].worldPos +
                      baryCoords.y * patch[1].worldPos +
                      baryCoords.z * patch[2].worldPos;
    
    float3 worldNormal = normalize(baryCoords.x * patch[0].worldNormal +
                                   baryCoords.y * patch[1].worldNormal +
                                   baryCoords.z * patch[2].worldNormal);
    
    float2 uv = baryCoords.x * patch[0].uv +
                baryCoords.y * patch[1].uv +
                baryCoords.z * patch[2].uv;
    
    // Sample height map and apply displacement
    float height = heightTexture.SampleLevel(heightSampler, uv, 0).r;
    
    // Apply midpoint and bias
    float displacedHeight = (height - displacement_midpoint) * displacement_scale + displacement_bias;
    float clampedHeight = clamp(displacedHeight, displacement_clamp_min, displacement_clamp_max);
    
    // Displace along world normal
    worldPos += worldNormal * clampedHeight;
    
    // Transform to clip space
    output.worldPos = worldPos;
    output.worldNormal = worldNormal;
    output.uv = uv;
    output.position = mul(view_proj, float4(worldPos, 1.0));
    
    return output;
}

// Fragment Shader (standard PBR - simplified for now)
struct FragmentOutput {
    float4 color : SV_Target0;
};

Texture2D baseColorTexture : register(t1);
SamplerState baseColorSampler : register(s1);

cbuffer MaterialParams : register(b3) {
    float4 base_color;
    float metallic;
    float perceptual_roughness;
    float3 _padding3;
};

FragmentOutput ps_main(DomainOutput input) {
    FragmentOutput output;
    
    float4 baseColor = baseColorTexture.Sample(baseColorSampler, input.uv) * base_color;
    output.color = baseColor;
    
    return output;
}

