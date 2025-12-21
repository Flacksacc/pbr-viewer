//! Mesh primitives for the PBR viewer

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, SphereKind};
use bevy::render::render_asset::RenderAssetUsages;

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
        &[MeshType::Sphere, MeshType::Cube, MeshType::Plane, MeshType::RoundedRect]
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

/// Create a sphere mesh using Bevy's built-in primitive with proper UV mapping
pub fn create_sphere(subdivisions: u32) -> Mesh {
    // Use Bevy's built-in sphere with UV subdivision for proper texture mapping
    // The subdivisions parameter controls the detail level
    let detail = subdivisions.clamp(8, 128) as usize;
    
    // Use UV sphere for proper texture mapping
    Sphere::new(1.0)
        .mesh()
        .kind(SphereKind::Uv { 
            sectors: detail, 
            stacks: detail / 2 
        })
        .build()
}

/// Create a cube mesh using Bevy's built-in primitive
pub fn create_cube() -> Mesh {
    Cuboid::new(2.0, 2.0, 2.0).mesh().build()
}

/// Create a plane mesh with the given resolution for displacement
pub fn create_plane(subdivisions: u32) -> Mesh {
    let subdivs = subdivisions.clamp(1, 256);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let step = 2.0 / subdivs as f32;

    // Generate vertices
    for y in 0..=subdivs {
        for x in 0..=subdivs {
            let px = -1.0 + x as f32 * step;
            let pz = -1.0 + y as f32 * step;

            let u = x as f32 / subdivs as f32;
            let v = 1.0 - (y as f32 / subdivs as f32); // Flip V for proper orientation

            positions.push([px, 0.0, pz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, v]);
        }
    }

    // Generate indices (proper winding order)
    for y in 0..subdivs {
        for x in 0..subdivs {
            let top_left = y * (subdivs + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = top_left + subdivs + 1;
            let bottom_right = bottom_left + 1;

            // First triangle
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            // Second triangle
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    // Generate tangents for normal mapping
    if mesh.generate_tangents().is_err() {
        // Fallback: add default tangents if generation fails
        let vertex_count = mesh.count_vertices();
        let tangents: Vec<[f32; 4]> = vec![[1.0, 0.0, 0.0, 1.0]; vertex_count];
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    }
    
    mesh
}

/// Create a rounded rectangle mesh
pub fn create_rounded_rect(subdivisions: u32, corner_radius: f32) -> Mesh {
    let subdivs = subdivisions.clamp(8, 256);
    let radius = corner_radius.clamp(0.0, 0.45);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let step = 1.0 / subdivs as f32;

    // Generate vertices in a grid pattern
    for y in 0..=subdivs {
        for x in 0..=subdivs {
            let u = x as f32 * step;
            let v = y as f32 * step;

            // Map UV to position (-1 to 1 range)
            let mut px = (u - 0.5) * 2.0;
            let mut pz = (v - 0.5) * 2.0;

            // Calculate distance from edges for rounding
            let inner_size = 1.0 - radius;

            // Apply corner rounding
            if px.abs() > inner_size && pz.abs() > inner_size {
                // We're in a corner region
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

            positions.push([px, 0.0, pz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, 1.0 - v]); // Flip V for proper orientation
        }
    }

    // Generate indices
    for y in 0..subdivs {
        for x in 0..subdivs {
            let top_left = y * (subdivs + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = top_left + subdivs + 1;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    // Generate tangents for normal mapping
    if mesh.generate_tangents().is_err() {
        let vertex_count = mesh.count_vertices();
        let tangents: Vec<[f32; 4]> = vec![[1.0, 0.0, 0.0, 1.0]; vertex_count];
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    }
    
    mesh
}
