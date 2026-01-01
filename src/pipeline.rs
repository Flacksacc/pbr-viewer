//! Render pipeline setup for PBR rendering

use wgpu::*;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// Uniform buffer for camera/view matrices
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &crate::camera_wgpu::Camera) {
        self.view_proj = camera.view_proj_matrix().to_cols_array_2d();
        self.view = camera.view_matrix().to_cols_array_2d();
        self.proj = camera.projection_matrix().to_cols_array_2d();
    }
}

/// Uniform buffer for model matrix
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
}

impl ModelUniform {
    pub fn new() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

/// Material parameters uniform
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct MaterialUniform {
    pub base_color_tint: [f32; 3],
    pub _padding0: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub normal_strength: f32,
    pub uv_scale: f32,
    pub view_mode: u32,  // ViewMode as u32
    pub texture_flags: u32,  // Bit flags: bit 0=base_color, bit 1=normal, bit 2=metallic_roughness, bit 3=ao, bit 4=emissive, bit 5=height
    pub light_direction: [f32; 3],  // Light direction (normalized)
    pub _padding1: f32,  // Padding to maintain 16-byte alignment
}

unsafe impl bytemuck::Pod for MaterialUniform {}
unsafe impl bytemuck::Zeroable for MaterialUniform {}

impl MaterialUniform {
    pub fn new() -> Self {
        Self {
            base_color_tint: [0.8, 0.8, 0.8],
            _padding0: 0.0,
            metallic: 0.0,
            roughness: 0.5,
            normal_strength: 1.0,
            uv_scale: 1.0,
            view_mode: 0,  // Lit
            texture_flags: 0,
            light_direction: [-1.0, -1.0, -1.0],  // Default light direction
            _padding1: 0.0,
        }
    }
}

/// Render pipeline and resources
pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,
    pub model_uniform: ModelUniform,
    pub model_buffer: Buffer,
    pub material_uniform: MaterialUniform,
    pub material_buffer: Buffer,
    pub material_bind_group: BindGroup,
}

impl RenderPipeline {
    pub fn new(
        device: &Device,
        shader: &ShaderModule,
        surface_format: TextureFormat,
    ) -> Result<Self, anyhow::Error> {
        // Create camera uniform buffer
        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create camera bind group layout (view_proj and model)
        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("camera_bind_group_layout"),
        });

        // Create model uniform buffer (needed before camera bind group)
        let model_uniform = ModelUniform::new();
        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[model_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create camera bind group (view_proj and model)
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: model_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });

        // Model bind group is the same as camera bind group - we reuse it
        // since both contain view_proj and model matrices in the same layout

        // Create material uniform buffer
        let material_uniform = MaterialUniform::new();
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create texture bind group layout (will be provided by TextureSet)
        // Note: This layout should match TextureSet::bind_group_layout
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
        });

        // Create material bind group layout
        // Make it accessible in both vertex and fragment stages for UV scale
        let material_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("material_bind_group_layout"),
        });

        // Create material bind group
        let material_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &material_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            }],
            label: Some("material_bind_group"),
        });

        // Create render pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[crate::mesh_wgpu::Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Self {
            pipeline,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            model_uniform,
            model_buffer,
            material_uniform,
            material_buffer,
            material_bind_group,
        })
    }

    pub fn update_camera(&mut self, queue: &Queue, camera: &crate::camera_wgpu::Camera) {
        self.camera_uniform.update_view_proj(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn update_model(&mut self, queue: &Queue, model_matrix: Mat4) {
        self.model_uniform.model = model_matrix.to_cols_array_2d();
        queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&[self.model_uniform]));
    }

    pub fn update_material(
        &mut self,
        queue: &Queue,
        material: &crate::state_wgpu::MaterialParams,
        view_mode: crate::state_wgpu::ViewMode,
        loaded_textures: &crate::state_wgpu::LoadedTextures,
    ) {
        self.material_uniform.base_color_tint = material.base_color_tint;
        self.material_uniform.metallic = material.metallic_multiplier;
        self.material_uniform.roughness = material.roughness_multiplier;
        self.material_uniform.normal_strength = material.normal_strength;
        self.material_uniform.uv_scale = material.uv_scale;
        
        // Set view mode as u32
        self.material_uniform.view_mode = view_mode as u32;
        
        // Pack texture availability flags into a u32
        let mut flags = 0u32;
        if loaded_textures.base_color { flags |= 1 << 0; }
        if loaded_textures.normal { flags |= 1 << 1; }
        if loaded_textures.metallic || loaded_textures.orm { flags |= 1 << 2; }
        if loaded_textures.ao || loaded_textures.orm { flags |= 1 << 3; }
        if loaded_textures.emissive { flags |= 1 << 4; }
        if loaded_textures.height { flags |= 1 << 5; }
        self.material_uniform.texture_flags = flags;
        
        queue.write_buffer(&self.material_buffer, 0, bytemuck::cast_slice(&[self.material_uniform]));
    }
    
    pub fn update_light_direction(
        &mut self,
        queue: &Queue,
        light_direction: glam::Vec3,
    ) {
        let normalized = light_direction.normalize();
        self.material_uniform.light_direction = [normalized.x, normalized.y, normalized.z];
        queue.write_buffer(&self.material_buffer, 0, bytemuck::cast_slice(&[self.material_uniform]));
    }
}


