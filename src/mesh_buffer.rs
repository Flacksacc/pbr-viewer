//! Mesh buffer management for wgpu

use wgpu::*;
use wgpu::util::DeviceExt;
use crate::mesh_wgpu::MeshData;

/// GPU mesh buffers
pub struct MeshBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl MeshBuffer {
    pub fn new(device: &Device, mesh_data: &MeshData) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh_data.vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh_data.indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh_data.indices.len() as u32,
        }
    }
}

