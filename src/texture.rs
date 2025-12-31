//! Texture loading and management for wgpu

use wgpu::*;
use image::DynamicImage;

/// Load a texture from bytes
pub fn load_texture(
    device: &Device,
    queue: &Queue,
    bytes: &[u8],
    label: Option<&str>,
) -> Result<(Texture, TextureView, Sampler), anyhow::Error> {
    let img = image::load_from_memory(bytes)?;
    load_texture_from_image(device, queue, &img, label)
}

/// Load a texture from an image
pub fn load_texture_from_image(
    device: &Device,
    queue: &Queue,
    img: &DynamicImage,
    label: Option<&str>,
) -> Result<(Texture, TextureView, Sampler), anyhow::Error> {
    let rgba = img.to_rgba8();
    let dimensions = rgba.dimensions();
    
    let size = Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    
    let texture = device.create_texture(&TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    
    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &rgba,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        size,
    );
    
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,  // Use Linear for better mipmap quality
        ..Default::default()
    });
    
    Ok((texture, view, sampler))
}

/// Create a 1x1 placeholder texture
pub fn create_placeholder_texture(
    device: &Device,
    queue: &Queue,
    color: [u8; 4],
    label: Option<&str>,
) -> (Texture, TextureView, Sampler) {
    let size = Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    
    let texture = device.create_texture(&TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    
    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &color,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        size,
    );
    
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,  // Use Linear for better mipmap quality
        ..Default::default()
    });
    
    (texture, view, sampler)
}

