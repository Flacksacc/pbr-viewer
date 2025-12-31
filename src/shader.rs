//! Shader loading utilities

use wgpu::*;
use std::borrow::Cow;

/// Load a shader module from WGSL source
pub fn load_shader(device: &Device, source: &str, label: Option<&str>) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label,
        source: ShaderSource::Wgsl(Cow::Borrowed(source)),
    })
}

/// Load a shader module from a WGSL file (embedded in binary)
pub fn load_shader_from_str(device: &Device, source: &'static str, label: Option<&str>) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label,
        source: ShaderSource::Wgsl(Cow::Borrowed(source)),
    })
}

