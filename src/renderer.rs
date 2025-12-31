//! WGPU Renderer - Direct rendering without Bevy

use wgpu::*;
use winit::window::Window;

#[allow(deprecated)]
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};

/// Main renderer struct
pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: Texture,
    pub depth_texture_view: TextureView,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Self, anyhow::Error> {
        let size = window.inner_size();
        
        // Create instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });
        
        // Create surface using raw window handle
        // The 'static lifetime is safe here because the window lives as long as the renderer
        #[allow(deprecated)]
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: window.raw_display_handle()?,
                raw_window_handle: window.raw_window_handle()?,
            })?
        };
        
        // Request adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate adapter"))?;
        
        // Request device
        // Note: TESSELATION_SHADER feature may not be available on all hardware
        // We'll request it but handle fallback
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(), // Start without tessellation for compatibility
                    required_limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;
        
        // Get surface capabilities
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        // Configure surface
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        // Create depth texture
        let depth_texture = device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth_texture"),
            view_formats: &[],
        });
        
        let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());
        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            depth_texture_view,
        })
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Recreate depth texture
            self.depth_texture = self.device.create_texture(&TextureDescriptor {
                size: Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT,
                label: Some("depth_texture"),
                view_formats: &[],
            });
            
            self.depth_texture_view = self.depth_texture.create_view(&TextureViewDescriptor::default());
        }
    }
    
    pub fn get_current_texture(&mut self) -> Result<SurfaceTexture, SurfaceError> {
        self.surface.get_current_texture()
    }
}

