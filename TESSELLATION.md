# GPU Tessellation & Displacement Implementation

This document describes the GPU tessellation and displacement mapping implementation for the PBR viewer.

## Overview

The implementation provides Unreal-style GPU tessellation with displacement mapping using DX12/Vulkan tessellation stages (Hull/Domain shaders or Tess Control/Eval shaders).

## Architecture

### Shader Files

1. **`assets/shaders/tessellation_material.wgsl`** - WGSL shader for Bevy compatibility
   - Provides basic displacement mapping in vertex/fragment stages
   - Note: WGSL doesn't have native tessellation stages, so this uses a simplified approach

2. **`assets/shaders/tessellation.hlsl`** - HLSL shaders for DX12
   - Complete implementation with:
     - Vertex Shader (`vs_main`)
     - Hull Shader (`hs_main`) - Tessellation Control
     - Domain Shader (`ds_main`) - Tessellation Evaluation with displacement
     - Fragment Shader (`ps_main`)

3. **`assets/shaders/tessellation.glsl`** - GLSL shaders for Vulkan
   - Complete implementation with:
     - Vertex Shader
     - Tessellation Control Shader
     - Tessellation Evaluation Shader with displacement
     - Fragment Shader

### Features

#### Screen-Space Tessellation
- Calculates tessellation factors based on screen-space edge length
- Distance-based falloff for performance
- Quantized per-edge tess factors (8 levels) to prevent cracks between adjacent triangles

#### Displacement Mapping
- Samples height map in UV space during domain/tessellation evaluation stage
- Displaces vertices along interpolated world normal
- User-controllable parameters:
  - **Displacement Scale**: Overall strength of displacement
  - **Midpoint**: Height value that maps to zero displacement (default 0.5)
  - **Bias**: Constant offset added to displacement
  - **Clamp Min/Max**: Limits displacement range

#### Quality Controls
- **Min/Max Tessellation Factor**: Controls tessellation density range
- **Quality Cap**: Maximum tessellation factor to prevent performance issues
- **Screen Space Scale**: Controls how screen-space size affects tessellation
- **Distance Scale**: Controls distance-based falloff

#### Debug Visualization
- **Tessellation Density**: Color-coded visualization of tessellation factor
- **Wireframe**: Shows tessellated geometry edges
- **Displacement Only**: Shows only displacement without lighting

## Usage

### Enabling GPU Tessellation

1. Open the UI panel
2. Navigate to "🔷 GPU Tessellation" section
3. Check "Enable GPU Tessellation"
4. Adjust parameters as needed

### Parameters

#### Tessellation Factors
- **Min Tessellation Factor** (1-16): Minimum subdivision level
- **Max Tessellation Factor** (1-128): Maximum subdivision level
- **Quality Cap** (1-128): Hard limit on tessellation to control performance

#### Displacement
- **Displacement Scale** (0-1): Strength of height map displacement
- **Displacement Midpoint** (0-1): Height value that results in no displacement
- **Displacement Bias** (-1 to 1): Constant offset
- **Clamp Min/Max**: Limits displacement range

#### Quality Settings
- **Screen Space Scale** (10-500): How screen size affects tessellation
- **Distance Scale** (0.01-1.0): Distance-based falloff rate

## Implementation Notes

### Current Status

The current implementation provides:
- ✅ Custom material system with tessellation parameters
- ✅ UI controls for all tessellation parameters
- ✅ HLSL and GLSL shaders ready for compilation
- ✅ WGSL shader with displacement support
- ✅ State management for tessellation parameters

### Hardware Tessellation Setup

To enable true hardware tessellation (Hull/Domain shaders), you need to:

1. **Compile HLSL/GLSL to SPIR-V**:
   ```bash
   # For HLSL (using DXC)
   dxc -spirv -T vs_6_0 -E vs_main tessellation.hlsl -o tess_vs.spv
   dxc -spirv -T hs_6_0 -E hs_main tessellation.hlsl -o tess_hs.spv
   dxc -spirv -T ds_6_0 -E ds_main tessellation.hlsl -o tess_ds.spv
   dxc -spirv -T ps_6_0 -E ps_main tessellation.hlsl -o tess_ps.spv
   
   # For GLSL (using glslangValidator)
   glslangValidator -V tessellation.glsl -o tessellation.spv
   ```

2. **Create Custom Render Pipeline**:
   - Extend Bevy's render pipeline to support tessellation stages
   - Load SPIR-V shaders
   - Configure pipeline state with tessellation

3. **Shadow Pass Support**:
   - Duplicate tessellation pipeline for shadow rendering
   - Ensure displacement is applied in shadow passes for correct silhouettes

### WGSL Limitations

WGSL (WebGPU Shading Language) doesn't have native tessellation stages. The current WGSL implementation uses:
- Vertex shader with displacement approximation
- Fragment shader for visualization

For true hardware tessellation, use the HLSL/GLSL shaders compiled to SPIR-V.

## Performance Considerations

- **Quality Cap**: Set this to limit maximum tessellation and prevent performance issues
- **Distance Scale**: Higher values reduce tessellation at distance, improving performance
- **Screen Space Scale**: Lower values reduce tessellation overall

## Future Enhancements

- [ ] Complete SPIR-V pipeline integration
- [ ] Shadow pass tessellation support
- [ ] Geometry shader fallback for platforms without tessellation
- [ ] Adaptive quality based on frame time
- [ ] LOD system integration

## References

- [Unreal Engine Tessellation Documentation](https://docs.unrealengine.com/en-US/Engine/Rendering/Materials/MaterialProperties/Tessellation/index.html)
- [Vulkan Tessellation Shaders](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#tessellation)
- [DirectX 12 Tessellation](https://docs.microsoft.com/en-us/windows/win32/direct3d12/tessellation-in-direct3d-12)


