//! Mesh generation for wgpu renderer

use glam::{Vec3, Vec2};

/// Vertex data structure for wgpu
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Mesh data structure
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Types of meshes available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeshType {
    #[default]
    Sphere,
    Cube,
    Plane,
    RoundedRect,
    Custom,
}

impl MeshType {
    pub fn primitives() -> &'static [MeshType] {
        &[MeshType::Sphere, MeshType::Cube]
    }

    pub fn name(&self) -> &'static str {
        match self {
            MeshType::Sphere => "Sphere",
            MeshType::Cube => "Cube",
            MeshType::Plane => "Plane",
            MeshType::RoundedRect => "Rounded Rect",
            MeshType::Custom => "Custom Model",
        }
    }
}

/// Create a sphere mesh
pub fn create_sphere(subdivisions: u32) -> MeshData {
    let detail = subdivisions.clamp(8, 128) as usize;
    let sectors = detail;
    let stacks = detail / 2;
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let radius = 1.0;
    
    // Generate vertices
    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * std::f32::consts::PI / stacks as f32;
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();
        
        for j in 0..=sectors {
            let sector_angle = j as f32 * 2.0 * std::f32::consts::PI / sectors as f32;
            
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();
            // Correct coordinate system: [x, y, z] not [x, z, y]
            let position = [x, y, z];
            let normal = [x, y, z];  // Normal is same as position for unit sphere
            let u = j as f32 / sectors as f32;
            let v = i as f32 / stacks as f32;
            let uv = [u, 1.0 - v];
            
            // Calculate proper tangent vector for sphere
            // Tangent points along the direction of increasing u (around the sphere)
            let tangent = [
                -sector_angle.sin(),
                sector_angle.cos(),
                0.0,
                1.0,  // Handedness
            ];
            
            vertices.push(Vertex {
                position,
                normal,
                uv,
                tangent,
            });
        }
    }
    
    // Generate indices
    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;
        
        for j in 0..sectors {
            if i != 0 {
                indices.push(k1 as u32);
                indices.push(k2 as u32);
                indices.push((k1 + 1) as u32);
            }
            
            if i != (stacks - 1) {
                indices.push((k1 + 1) as u32);
                indices.push(k2 as u32);
                indices.push((k2 + 1) as u32);
            }
            
            k1 += 1;
            k2 += 1;
        }
    }
    
    MeshData { vertices, indices }
}

/// Create a cube mesh
pub fn create_cube() -> MeshData {
    let size = 1.0;
    let vertices = vec![
        // Front face
        Vertex { position: [-size, -size, size], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, -size, size], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, size, size], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [-size, size, size], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        // Back face
        Vertex { position: [-size, -size, -size], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [-size, size, -size], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, size, -size], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, -size, -size], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
        // Top face
        Vertex { position: [-size, size, -size], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [-size, size, size], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, size, size], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, size, -size], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        // Bottom face
        Vertex { position: [-size, -size, -size], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, -size, -size], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [size, -size, size], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [-size, -size, size], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        // Right face
        Vertex { position: [size, -size, -size], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0], tangent: [0.0, 0.0, 1.0, 1.0] },
        Vertex { position: [size, size, -size], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0], tangent: [0.0, 0.0, 1.0, 1.0] },
        Vertex { position: [size, size, size], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0], tangent: [0.0, 0.0, 1.0, 1.0] },
        Vertex { position: [size, -size, size], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0], tangent: [0.0, 0.0, 1.0, 1.0] },
        // Left face
        Vertex { position: [-size, -size, -size], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0], tangent: [0.0, 0.0, -1.0, 1.0] },
        Vertex { position: [-size, -size, size], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0], tangent: [0.0, 0.0, -1.0, 1.0] },
        Vertex { position: [-size, size, size], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0], tangent: [0.0, 0.0, -1.0, 1.0] },
        Vertex { position: [-size, size, -size], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0], tangent: [0.0, 0.0, -1.0, 1.0] },
    ];
    
    let indices = vec![
        0, 1, 2, 2, 3, 0, // front
        4, 5, 6, 6, 7, 4, // back
        8, 9, 10, 10, 11, 8, // top
        12, 13, 14, 14, 15, 12, // bottom
        16, 17, 18, 18, 19, 16, // right
        20, 21, 22, 22, 23, 20, // left
    ];
    
    MeshData { vertices, indices }
}

/// Create a plane mesh
pub fn create_plane(subdivisions: u32) -> MeshData {
    let subdivs = subdivisions.clamp(1, 256);
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let step = 2.0 / subdivs as f32;
    
    // Generate vertices
    for y in 0..=subdivs {
        for x in 0..=subdivs {
            let px = -1.0 + x as f32 * step;
            let pz = -1.0 + y as f32 * step;
            
            let u = x as f32 / subdivs as f32;
            let v = 1.0 - (y as f32 / subdivs as f32);
            
            vertices.push(Vertex {
                position: [px, 0.0, pz],
                normal: [0.0, 1.0, 0.0],
                uv: [u, v],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
    }
    
    // Generate indices
    for y in 0..subdivs {
        for x in 0..subdivs {
            let top_left = y * (subdivs + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = top_left + subdivs + 1;
            let bottom_right = bottom_left + 1;
            
            indices.push(top_left as u32);
            indices.push(bottom_left as u32);
            indices.push(top_right as u32);
            
            indices.push(top_right as u32);
            indices.push(bottom_left as u32);
            indices.push(bottom_right as u32);
        }
    }
    
    MeshData { vertices, indices }
}

/// Create a rounded rectangle mesh
pub fn create_rounded_rect(subdivisions: u32, corner_radius: f32) -> MeshData {
    let subdivs = subdivisions.clamp(8, 256);
    let radius = corner_radius.clamp(0.0, 0.45);
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let step = 1.0 / subdivs as f32;
    
    for y in 0..=subdivs {
        for x in 0..=subdivs {
            let u = x as f32 * step;
            let v = y as f32 * step;
            
            let mut px = (u - 0.5) * 2.0;
            let mut pz = (v - 0.5) * 2.0;
            
            let inner_size = 1.0 - radius;
            
            if px.abs() > inner_size && pz.abs() > inner_size {
                let corner_x = px.signum() * inner_size;
                let corner_z = pz.signum() * inner_size;
                
                let dx = px - corner_x;
                let dz = pz - corner_z;
                let dist = (dx * dx + dz * dz).sqrt();
                
                if dist > radius && dist > 0.0001 {
                    let scale = radius / dist;
                    px = corner_x + dx * scale;
                    pz = corner_z + dz * scale;
                }
            }
            
            vertices.push(Vertex {
                position: [px, 0.0, pz],
                normal: [0.0, 1.0, 0.0],
                uv: [u, 1.0 - v],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
    }
    
    for y in 0..subdivs {
        for x in 0..subdivs {
            let top_left = y * (subdivs + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = top_left + subdivs + 1;
            let bottom_right = bottom_left + 1;
            
            indices.push(top_left as u32);
            indices.push(bottom_left as u32);
            indices.push(top_right as u32);
            
            indices.push(top_right as u32);
            indices.push(bottom_left as u32);
            indices.push(bottom_right as u32);
        }
    }
    
    MeshData { vertices, indices }
}

