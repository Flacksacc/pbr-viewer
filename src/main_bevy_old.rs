//! PBR Texture Set Viewer
//!
//! A utility application to visualize PBR texture sets on various mesh primitives.

mod camera;
mod mesh;
mod state;
mod ui;
mod tessellation;

use bevy::prelude::*;
use bevy::gltf::Gltf;
use bevy_egui::EguiPlugin;
use camera::{OrbitCameraController, OrbitCameraPlugin};
use mesh::MeshType;
use state::{AppState, ViewMode};
use tessellation::{TessellationMaterialPlugin, TessellationMaterial};
use std::path::PathBuf;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "PBR Texture Viewer".to_string(),
                    resolution: (1600.0, 900.0).into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(
                    bevy::render::settings::WgpuSettings {
                        backends: Some(bevy::render::settings::Backends::VULKAN),
                        ..default()
                    }
                ),
                ..default()
            })
        )
        .add_plugins(EguiPlugin)
        .add_plugins(OrbitCameraPlugin)
        .add_plugins(TessellationMaterialPlugin)
        .init_resource::<AppState>()
        .add_systems(Startup, (setup_scene, log_startup_info))
        .add_systems(Update, (
            ui::ui_system,
            handle_mesh_change,
            load_custom_model,
            spawn_custom_model,
            load_textures,
            apply_material,
            apply_tessellation_material,
            handle_light_update,
            handle_dropped_files,
            update_model_rotation,
        ))
        .run();
}

fn log_startup_info() {
    info!("==============================================");
    info!("PBR Texture Viewer started!");
    info!("Drag and drop textures/folders onto the window");
    info!("==============================================");
}

/// Marker component for the main model
#[derive(Component)]
pub struct MainModel;

/// Marker component for the directional light
#[derive(Component)]
pub struct MainLight;

/// Set up the initial scene
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<AppState>,
) {
    // Create default sphere mesh
    let mesh_handle = meshes.add(mesh::create_sphere(state.tessellation_level));
    state.mesh_handle = Some(mesh_handle.clone());

    // Create default material
    let material = StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        metallic: 0.0,
        perceptual_roughness: 0.5,
        ..default()
    };
    let material_handle = materials.add(material);
    state.material_handle = Some(material_handle.clone());

    // Spawn the main model
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            transform: Transform::IDENTITY,
            ..default()
        },
        MainModel,
        UsingTessellationMaterial(false),
    ));

    // Create camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCameraController::default(),
    ));

    // Create directional light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 15000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainLight,
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });
}

/// Handle mesh type changes
fn handle_mesh_change(
    mut state: ResMut<AppState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Handle<Mesh>, With<MainModel>>,
) {
    if !state.mesh_changed {
        return;
    }
    
    // Skip if using custom model - that's handled by spawn_custom_model
    if state.using_custom_model {
        state.mesh_changed = false;
        return;
    }
    
    state.mesh_changed = false;

    let new_mesh = match state.current_mesh {
        MeshType::Sphere => mesh::create_sphere(state.tessellation_level),
        MeshType::Cube => mesh::create_cube(),
        MeshType::Plane => mesh::create_plane(state.tessellation_level),
        MeshType::RoundedRect => mesh::create_rounded_rect(state.tessellation_level, 0.2),
        MeshType::Custom => return, // Handled by spawn_custom_model
    };

    let mesh_handle = meshes.add(new_mesh);
    state.mesh_handle = Some(mesh_handle.clone());

    for mut handle in query.iter_mut() {
        *handle = mesh_handle.clone();
    }
}

/// Load textures from folder (only when folder changes)
fn load_textures(
    mut state: ResMut<AppState>,
    asset_server: Res<AssetServer>,
) {
    if !state.textures_need_reload {
        return;
    }
    state.textures_need_reload = false;
    
    // Reset texture handles
    state.texture_handles = state::TextureHandles::default();
    state.loaded_textures.reset();
    
    // Clone folder to avoid borrow issues
    let folder_opt = state.texture_folder.clone();
    
    if let Some(folder) = folder_opt {
        let folder_path = PathBuf::from(&folder);
        
        // Base color texture
        if let Some(path) = find_texture_in_folder(&folder_path, &["basecolor", "albedo", "diffuse", "color", "base"]) {
            state.texture_handles.base_color = Some(asset_server.load(path));
            state.loaded_textures.base_color = true;
        }
        
        // Normal map
        if let Some(path) = find_texture_in_folder(&folder_path, &["normal", "normalmap", "nrm", "nor"]) {
            state.texture_handles.normal = Some(asset_server.load(path));
            state.loaded_textures.normal = true;
        }
        
        // Roughness (separate)
        if let Some(path) = find_texture_in_folder(&folder_path, &["roughness", "rough", "rgh"]) {
            state.texture_handles.roughness = Some(asset_server.load(path));
            state.loaded_textures.roughness = true;
        }
        
        // Metallic (separate)
        if let Some(path) = find_texture_in_folder(&folder_path, &["metallic", "metal", "metalness", "met"]) {
            state.texture_handles.metallic = Some(asset_server.load(path));
            state.loaded_textures.metallic = true;
        }
        
        // ORM (combined AO/Roughness/Metallic)
        if let Some(path) = find_texture_in_folder(&folder_path, &["orm", "arm", "rma", "metallic_roughness"]) {
            state.texture_handles.orm = Some(asset_server.load(path));
            state.loaded_textures.orm = true;
        }
        
        // Ambient Occlusion (separate)
        if let Some(path) = find_texture_in_folder(&folder_path, &["ao", "occlusion", "ambient_occlusion", "ambientocclusion"]) {
            state.texture_handles.ao = Some(asset_server.load(path));
            state.loaded_textures.ao = true;
        }
        
        // Emissive
        if let Some(path) = find_texture_in_folder(&folder_path, &["emissive", "emission", "emit", "glow"]) {
            state.texture_handles.emissive = Some(asset_server.load(path));
            state.loaded_textures.emissive = true;
        }
        
        // Height/Displacement
        if let Some(path) = find_texture_in_folder(&folder_path, &["height", "displacement", "disp", "bump", "depth"]) {
            state.texture_handles.height = Some(asset_server.load(path));
            state.loaded_textures.height = true;
        }
        
        info!("Loaded textures from: {}", folder);
    }
    
    // Trigger material update after loading textures
    state.material_changed = true;
}

/// Component to track which material type is being used
#[derive(Component)]
pub struct UsingTessellationMaterial(bool);

/// Apply tessellation material when GPU tessellation is enabled
fn apply_tessellation_material(
    mut commands: Commands,
    mut state: ResMut<AppState>,
    mut tess_materials: ResMut<Assets<TessellationMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, Option<&UsingTessellationMaterial>), With<MainModel>>,
    _mesh_handles: Res<Assets<Mesh>>,
) {
    // Check if we need to switch material types or update
    let should_use_tess = state.gpu_tessellation.enabled;
    let needs_update = state.material_changed || state.gpu_tessellation.enabled;
    
    if !needs_update {
        return;
    }
    
    for (entity, using_tess_opt) in query.iter() {
        let currently_using_tess = using_tess_opt.map(|c| c.0).unwrap_or(false);
        
        // If we need to switch material types, despawn and respawn
        if should_use_tess != currently_using_tess {
            // Get mesh handle
            let mesh_handle = if let Some(ref handle) = state.mesh_handle {
                handle.clone()
            } else {
                continue;
            };
            
            // Despawn old entity
            commands.entity(entity).despawn_recursive();
            
            // Create new material
            let params = &state.material_params;
            let tess_params = &state.gpu_tessellation;
            let tex = &state.texture_handles;
            
            if should_use_tess {
                // Create tessellation material
                let mut tess_material = TessellationMaterial::default();
                
                let tint = params.base_color_tint;
                tess_material.base_color = LinearRgba::rgb(tint[0], tint[1], tint[2]);
                tess_material.base_color_texture = tex.base_color.clone();
                tess_material.normal_map_texture = tex.normal.clone();
                tess_material.metallic = params.metallic_multiplier;
                tess_material.perceptual_roughness = params.roughness_multiplier;
                tess_material.metallic_roughness_texture = tex.orm.clone();
                tess_material.occlusion_texture = tex.ao.clone();
                tess_material.emissive_texture = tex.emissive.clone();
                tess_material.emissive = LinearRgba::rgb(
                    params.emissive_strength,
                    params.emissive_strength,
                    params.emissive_strength,
                );
                tess_material.height_texture = tex.height.clone();
                
                // Update tessellation parameters
                tess_material.min_tess_factor = tess_params.min_tess_factor;
                tess_material.max_tess_factor = tess_params.max_tess_factor;
                tess_material.displacement_scale = tess_params.displacement_scale;
                tess_material.displacement_midpoint = tess_params.displacement_midpoint;
                tess_material.displacement_bias = tess_params.displacement_bias;
                tess_material.displacement_clamp_min = tess_params.displacement_clamp_min;
                tess_material.displacement_clamp_max = tess_params.displacement_clamp_max;
                tess_material.screen_space_scale = tess_params.screen_space_scale;
                tess_material.distance_scale = tess_params.distance_scale;
                tess_material.quality_cap = tess_params.quality_cap;
                
                let material_handle = tess_materials.add(tess_material);
                // Note: material_handle is Handle<TessellationMaterial>, not Handle<StandardMaterial>
                // so we can't store it in state.material_handle
                
                // Spawn with tessellation material using MaterialMeshBundle
                commands.spawn((
                    MaterialMeshBundle::<TessellationMaterial> {
                        mesh: mesh_handle,
                        material: material_handle,
                        transform: Transform::IDENTITY,
                        ..default()
                    },
                    MainModel,
                    UsingTessellationMaterial(true),
                ));
            } else {
                // Create standard material (handled by apply_material)
                // Just mark that we're not using tessellation
                commands.spawn((
                    PbrBundle {
                        mesh: mesh_handle,
                        material: standard_materials.add(StandardMaterial::default()),
                        transform: Transform::IDENTITY,
                        ..default()
                    },
                    MainModel,
                    UsingTessellationMaterial(false),
                ));
                state.material_changed = true; // Trigger standard material update
            }
        } else if should_use_tess {
            // Update existing tessellation material by finding the entity and updating the material
            // For now, we'll trigger a material update by respawning
            // In a more complete implementation, we'd update the material asset directly
            if state.material_changed {
                // Get mesh handle
                let mesh_handle = if let Some(ref handle) = state.mesh_handle {
                    handle.clone()
                } else {
                    continue;
                };
                
                // Despawn and respawn with updated material
                commands.entity(entity).despawn_recursive();
                
                let params = &state.material_params;
                let tess_params = &state.gpu_tessellation;
                let tex = &state.texture_handles;
                
                let mut tess_material = TessellationMaterial::default();
                
                let tint = params.base_color_tint;
                tess_material.base_color = LinearRgba::rgb(tint[0], tint[1], tint[2]);
                tess_material.base_color_texture = tex.base_color.clone();
                tess_material.normal_map_texture = tex.normal.clone();
                tess_material.metallic = params.metallic_multiplier;
                tess_material.perceptual_roughness = params.roughness_multiplier;
                tess_material.metallic_roughness_texture = tex.orm.clone();
                tess_material.occlusion_texture = tex.ao.clone();
                tess_material.emissive_texture = tex.emissive.clone();
                tess_material.emissive = LinearRgba::rgb(
                    params.emissive_strength,
                    params.emissive_strength,
                    params.emissive_strength,
                );
                tess_material.height_texture = tex.height.clone();
                
                tess_material.min_tess_factor = tess_params.min_tess_factor;
                tess_material.max_tess_factor = tess_params.max_tess_factor;
                tess_material.displacement_scale = tess_params.displacement_scale;
                tess_material.displacement_midpoint = tess_params.displacement_midpoint;
                tess_material.displacement_bias = tess_params.displacement_bias;
                tess_material.displacement_clamp_min = tess_params.displacement_clamp_min;
                tess_material.displacement_clamp_max = tess_params.displacement_clamp_max;
                tess_material.screen_space_scale = tess_params.screen_space_scale;
                tess_material.distance_scale = tess_params.distance_scale;
                tess_material.quality_cap = tess_params.quality_cap;
                
                let material_handle = tess_materials.add(tess_material);
                
                commands.spawn((
                    MaterialMeshBundle::<TessellationMaterial> {
                        mesh: mesh_handle,
                        material: material_handle,
                        transform: Transform::IDENTITY,
                        ..default()
                    },
                    MainModel,
                    UsingTessellationMaterial(true),
                ));
            }
        }
    }
    
    state.material_changed = false;
}

/// Apply material based on current state and view mode
fn apply_material(
    mut state: ResMut<AppState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Handle<StandardMaterial>, (With<MainModel>, Without<UsingTessellationMaterial>)>,
    using_tess_query: Query<&UsingTessellationMaterial, With<MainModel>>,
) {
    // Skip if using tessellation material
    if state.gpu_tessellation.enabled && !using_tess_query.is_empty() {
        return;
    }
    
    if !state.material_changed {
        return;
    }
    state.material_changed = false;

    for handle in query.iter() {
        if let Some(material) = materials.get_mut(handle) {
            let params = &state.material_params;
            let tex = &state.texture_handles;
            
            // Reset material to defaults first
            *material = StandardMaterial::default();
            
            // Apply UV scale transform to all textures
            let uv_scale = params.uv_scale;
            material.uv_transform = bevy::math::Affine2::from_scale(Vec2::splat(uv_scale));
            
            match state.view_mode {
                ViewMode::Lit => {
                    // Full PBR rendering
                    let tint = params.base_color_tint;
                    material.base_color = Color::srgb(tint[0], tint[1], tint[2]);
                    material.base_color_texture = tex.base_color.clone();
                    material.normal_map_texture = tex.normal.clone();
                    material.metallic = params.metallic_multiplier;
                    material.perceptual_roughness = params.roughness_multiplier;
                    
                    // Use ORM if available, otherwise use separate textures
                    if tex.orm.is_some() {
                        material.metallic_roughness_texture = tex.orm.clone();
                        material.metallic = 1.0; // Let texture control it
                        material.perceptual_roughness = 1.0;
                    }
                    
                    material.occlusion_texture = tex.ao.clone();
                    material.emissive_texture = tex.emissive.clone();
                    material.emissive = LinearRgba::rgb(
                        params.emissive_strength,
                        params.emissive_strength,
                        params.emissive_strength,
                    );
                    
                    // Parallax/displacement
                    if tex.height.is_some() {
                        material.depth_map = tex.height.clone();
                        material.parallax_depth_scale = params.displacement_strength;
                        material.parallax_mapping_method = bevy::pbr::ParallaxMappingMethod::Relief { max_steps: 8 };
                    }
                    
                    material.unlit = false;
                }
                
                ViewMode::BaseColor => {
                    // Show only base color texture
                    material.base_color = Color::WHITE;
                    material.base_color_texture = tex.base_color.clone();
                    material.unlit = true;
                }
                
                ViewMode::Normals => {
                    // Show normal map as color (displayed on model)
                    material.base_color = Color::WHITE;
                    material.base_color_texture = tex.normal.clone();
                    material.unlit = true;
                }
                
                ViewMode::Roughness => {
                    // Show roughness texture
                    material.base_color = Color::WHITE;
                    // Use separate roughness if available, otherwise use ORM
                    if tex.roughness.is_some() {
                        material.base_color_texture = tex.roughness.clone();
                    } else if tex.orm.is_some() {
                        // ORM stores roughness in G channel - we show whole texture
                        material.base_color_texture = tex.orm.clone();
                    }
                    material.unlit = true;
                }
                
                ViewMode::Metallic => {
                    // Show metallic texture
                    material.base_color = Color::WHITE;
                    if tex.metallic.is_some() {
                        material.base_color_texture = tex.metallic.clone();
                    } else if tex.orm.is_some() {
                        // ORM stores metallic in B channel - we show whole texture
                        material.base_color_texture = tex.orm.clone();
                    }
                    material.unlit = true;
                }
                
                ViewMode::AO => {
                    // Show AO texture
                    material.base_color = Color::WHITE;
                    if tex.ao.is_some() {
                        material.base_color_texture = tex.ao.clone();
                    } else if tex.orm.is_some() {
                        // ORM stores AO in R channel - we show whole texture
                        material.base_color_texture = tex.orm.clone();
                    }
                    material.unlit = true;
                }
                
                ViewMode::Emissive => {
                    // Show emissive texture
                    material.base_color = Color::WHITE;
                    material.base_color_texture = tex.emissive.clone();
                    material.unlit = true;
                }
                
                ViewMode::Height => {
                    // Show height/displacement texture
                    material.base_color = Color::WHITE;
                    material.base_color_texture = tex.height.clone();
                    material.unlit = true;
                }
            }
        }
    }
}

/// Load custom GLTF/GLB model
fn load_custom_model(
    mut state: ResMut<AppState>,
    asset_server: Res<AssetServer>,
) {
    if !state.custom_model_needs_load {
        return;
    }
    
    if let Some(ref path) = state.custom_model_path {
        info!("Loading custom model: {}", path);
        let handle: Handle<Gltf> = asset_server.load(path.clone());
        state.custom_model_handle = Some(handle);
        state.custom_model_needs_load = false;
    }
}

/// Marker for custom model scene root
#[derive(Component)]
pub struct CustomModelRoot;

/// Spawn the custom model once loaded
fn spawn_custom_model(
    mut commands: Commands,
    mut state: ResMut<AppState>,
    gltf_assets: Res<Assets<Gltf>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    main_model_query: Query<Entity, With<MainModel>>,
    custom_model_query: Query<Entity, With<CustomModelRoot>>,
) {
    // Only process if we have a custom model handle and want to use it
    if !state.using_custom_model {
        // If we switched away from custom model, clean up custom model entities
        for entity in custom_model_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }
    
    let Some(ref gltf_handle) = state.custom_model_handle else {
        return;
    };
    
    // Check if GLTF is loaded
    let Some(gltf) = gltf_assets.get(gltf_handle) else {
        return; // Not loaded yet
    };
    
    // Check if we already spawned this model
    if !custom_model_query.is_empty() {
        return;
    }
    
    info!("Spawning custom model");
    
    // Remove existing main model (the primitive)
    for entity in main_model_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Create new material for the custom model
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..default()
    });
    state.material_handle = Some(material);
    
    // Spawn the GLTF scene
    if let Some(scene_handle) = gltf.scenes.first() {
        commands.spawn((
            SceneBundle {
                scene: scene_handle.clone(),
                ..default()
            },
            CustomModelRoot,
            MainModel,
        ));
    }
    
    state.material_changed = true;
}

/// Find a texture file in a folder matching any of the patterns
fn find_texture_in_folder(folder: &PathBuf, patterns: &[&str]) -> Option<PathBuf> {
    let extensions = ["png", "jpg", "jpeg", "tga", "bmp", "tiff"];
    
    if let Ok(entries) = std::fs::read_dir(folder) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            
            let file_name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            let extension = path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            if !extensions.contains(&extension.as_str()) {
                continue;
            }
            
            for pattern in patterns {
                if file_name.contains(pattern) {
                    return Some(path);
                }
            }
        }
    }
    
    None
}

/// Handle light parameter updates
fn handle_light_update(
    state: Res<AppState>,
    mut lights: Query<(&mut DirectionalLight, &mut Transform), With<MainLight>>,
    mut ambient: ResMut<AmbientLight>,
) {
    for (mut light, mut transform) in lights.iter_mut() {
        let dir = state.light_params.direction;
        let color = state.light_params.color;
        
        light.color = Color::srgb(color[0], color[1], color[2]);
        light.illuminance = state.light_params.intensity * 1000.0;
        
        // Point light in the direction specified
        *transform = Transform::from_xyz(-dir.x * 10.0, -dir.y * 10.0, -dir.z * 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
    
    ambient.brightness = state.light_params.ambient_intensity * 500.0;
}

/// Handle drag and drop of files/folders
fn handle_dropped_files(
    mut state: ResMut<AppState>,
    mut events: EventReader<bevy::window::FileDragAndDrop>,
) {
    let texture_extensions = ["png", "jpg", "jpeg", "tga", "bmp", "tiff", "exr", "hdr"];
    let model_extensions = ["gltf", "glb", "obj"];
    
    for event in events.read() {
        info!("Drag event received: {:?}", event);
        match event {
            bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } => {
                info!("File dropped: {:?}", path_buf);
                
                // Clear hover state
                state.drag_hover_path = None;
                
                // Check file extension
                let ext = path_buf.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase())
                    .unwrap_or_default();
                
                let is_texture = texture_extensions.contains(&ext.as_str());
                let is_model = model_extensions.contains(&ext.as_str());
                
                if is_model {
                    // Load as custom model
                    state.custom_model_path = Some(path_buf.to_string_lossy().to_string());
                    state.custom_model_needs_load = true;
                    state.using_custom_model = true;
                    state.current_mesh = mesh::MeshType::Custom;
                    state.mesh_changed = true;
                    info!("Loading custom model: {:?}", path_buf);
                } else if is_texture || path_buf.is_dir() {
                    // Load texture folder
                    let folder = if path_buf.is_file() {
                        path_buf.parent().map(|p| p.to_path_buf())
                    } else {
                        Some(path_buf.clone())
                    };
                    
                    if let Some(folder_path) = folder {
                        state.texture_folder = Some(folder_path.to_string_lossy().to_string());
                        state.textures_need_reload = true;
                        info!("Loading texture set from: {:?}", folder_path);
                    }
                }
            }
            bevy::window::FileDragAndDrop::HoveredFile { path_buf, .. } => {
                info!("File hovering: {:?}", path_buf);
                state.drag_hover_path = Some(path_buf.to_string_lossy().to_string());
            }
            bevy::window::FileDragAndDrop::HoveredFileCanceled { .. } => {
                info!("Hover canceled");
                state.drag_hover_path = None;
            }
        }
    }
}

/// Update model rotation based on mouse input
fn update_model_rotation(
    state: Res<AppState>,
    mut query: Query<&mut Transform, With<MainModel>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation = state.model_rotation;
    }
}
