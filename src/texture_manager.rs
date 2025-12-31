//! Texture management for PBR rendering

use wgpu::*;
use crate::texture;

/// Texture resources for PBR material
pub struct TextureSet {
    pub base_color: (Texture, TextureView, Sampler),
    pub normal: (Texture, TextureView, Sampler),
    pub metallic_roughness: (Texture, TextureView, Sampler),
}

impl TextureSet {
    pub fn create_placeholder(device: &Device, queue: &Queue) -> Self {
        let base_color = texture::create_placeholder_texture(
            device,
            queue,
            [128, 128, 128, 255], // Gray
            Some("base_color_placeholder"),
        );

        let normal = texture::create_placeholder_texture(
            device,
            queue,
            [128, 128, 255, 255], // Normal map default (flat blue)
            Some("normal_placeholder"),
        );

        let metallic_roughness = texture::create_placeholder_texture(
            device,
            queue,
            [0, 128, 0, 255], // Default metallic/roughness
            Some("metallic_roughness_placeholder"),
        );

        Self {
            base_color,
            normal,
            metallic_roughness,
        }
    }

    pub fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    pub fn create_bind_group(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&self.base_color.1),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.base_color.2),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&self.normal.1),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&self.normal.2),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&self.metallic_roughness.1),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Sampler(&self.metallic_roughness.2),
                },
            ],
            label: Some("texture_bind_group"),
        })
    }
}

