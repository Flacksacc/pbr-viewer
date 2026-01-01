//! PBR Texture Set Viewer - wgpu version

mod renderer;
mod camera_wgpu;
mod mesh_wgpu;
mod state_wgpu;
mod texture;
mod shader;
mod pipeline;
mod mesh_buffer;
mod texture_manager;
mod texture_loader;
mod input;
mod ui_wgpu;
mod egui_integration;

// Re-export for convenience
pub use mesh_wgpu::MeshType;
pub use state_wgpu::{AppState, ViewMode, TessellationDebugMode};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use renderer::Renderer;
use state_wgpu::{AppState as WgpuAppState, DEFAULT_UI_PANEL_WIDTH};
use camera_wgpu::{OrbitCamera, Camera};
use pipeline::RenderPipeline;
use mesh_wgpu::{create_sphere, create_cube};
use mesh_buffer::MeshBuffer;
use texture_manager::TextureSet;
use shader::load_shader_from_str;
use glam::Mat4;
use input::InputState;
use egui_integration::EguiState;
use ui_wgpu::build_ui;

// Embed shader source
const PBR_SHADER: &str = include_str!("../assets/shaders/pbr.wgsl");

// Store render state
struct RenderState {
    render_pipeline: RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    mesh_buffer: MeshBuffer,
    orbit_camera: OrbitCamera,
    app_state: WgpuAppState,
    camera: Camera,
    input_state: InputState,
    egui_state: EguiState,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    log::info!("PBR Texture Viewer started!");
    
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("PBR Texture Viewer")
        .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0))
        .build(&event_loop)?;
    
    let window_ref = &window; // Store reference for closure
    
    let mut renderer = pollster::block_on(async {
        Renderer::new(window_ref).await
    })?;
    
    // Initialize egui
    let egui_state = EguiState::new(
        &renderer.device,
        &renderer.config,
        window_ref,
    );
    
    // Create shader
    let shader = load_shader_from_str(&renderer.device, PBR_SHADER, Some("pbr_shader"));
    
    // Create render pipeline
    let mut render_pipeline = RenderPipeline::new(
        &renderer.device,
        &shader,
        renderer.config.format,
    )?;
    
    // Create placeholder textures
    let texture_set = TextureSet::create_placeholder(&renderer.device, &renderer.queue);
    let texture_bind_group_layout = TextureSet::bind_group_layout(&renderer.device);
    let texture_bind_group = texture_set.create_bind_group(&renderer.device, &texture_bind_group_layout);
    
    // Create mesh
    let mesh_data = create_sphere(32);
    let mesh_buffer = MeshBuffer::new(&renderer.device, &mesh_data);
    
    // Camera setup (accounting for UI panel width)
    let orbit_camera = OrbitCamera::new(glam::Vec3::ZERO, 3.0);
    let pixels_per_point = window_ref.scale_factor() as f32;
    let panel_width_pixels = DEFAULT_UI_PANEL_WIDTH * pixels_per_point;
    let viewport_width = (renderer.size.width as f32 - panel_width_pixels).max(1.0);
    let aspect = viewport_width / renderer.size.height as f32;
    let camera = orbit_camera.to_camera_with_aspect(aspect);
    render_pipeline.update_camera(&renderer.queue, &camera);
    
    // Model transform
    let model_matrix = Mat4::IDENTITY;
    render_pipeline.update_model(&renderer.queue, model_matrix);
    
    // Material params
    let app_state = WgpuAppState::default();
    render_pipeline.update_material(
        &renderer.queue,
        &app_state.material_params,
        app_state.view_mode,
        &app_state.loaded_textures,
    );
    // Initialize light direction
    render_pipeline.update_light_direction(&renderer.queue, app_state.light_params.direction);
    
    let mut render_state = RenderState {
        render_pipeline,
        texture_bind_group,
        mesh_buffer,
        orbit_camera,
        app_state,
        camera,
        input_state: InputState::new(),
        egui_state,
    };
    
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent { event, .. } => {
                // Pass events to egui FIRST - this is critical for UI interaction
                let egui_consumed = render_state.egui_state.handle_event(&window, &event);
                
                // Update input state only if egui didn't consume the event
                if !egui_consumed {
                    render_state.input_state.update_from_event(&event);
                }
                
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(physical_size);
                        // Update camera aspect ratio (accounting for UI panel)
                        let pixels_per_point = window.scale_factor() as f32;
                        let panel_width_pixels = render_state.app_state.ui_panel_width * pixels_per_point;
                        let viewport_width = (physical_size.width as f32 - panel_width_pixels).max(1.0);
                        render_state.camera.aspect = viewport_width / physical_size.height as f32;
                        render_state.render_pipeline.update_camera(&renderer.queue, &render_state.camera);
                    }
                    WindowEvent::RedrawRequested => {
                        // Handle input for camera control (only if not over UI)
                        let over_ui = render_state.egui_state.context.wants_pointer_input() || 
                                     render_state.egui_state.context.is_pointer_over_area();
                        if !over_ui {
                            handle_camera_input(&mut render_state, &renderer.queue);
                        }
                        render_frame(&mut renderer, &mut render_state, &window);
                    }
                    _ => {}
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}

fn handle_camera_input(render_state: &mut RenderState, queue: &wgpu::Queue) {
    let input = &mut render_state.input_state;
    
    // Track if model was rotated this frame
    let mut model_rotated = false;
    
    // Model rotation (left mouse button) - rotate towards mouse direction
    // Light direction stays fixed in world space - only model rotates
    if input.left_mouse_pressed && !input.right_mouse_pressed && input.mouse_delta.length_squared() > 0.0 {
        let sensitivity = 0.01;
        // Rotate model so it follows the mouse drag direction
        // Horizontal drag (right) rotates around Y axis to bring right side forward
        // Vertical drag (down) rotates around X axis to bring bottom forward
        let rotation_y = glam::Quat::from_rotation_y(input.mouse_delta.x * sensitivity);
        let rotation_x = glam::Quat::from_rotation_x(input.mouse_delta.y * sensitivity);
        // Apply rotations in order: first Y (horizontal), then X (vertical)
        render_state.app_state.model_rotation = rotation_x * rotation_y * render_state.app_state.model_rotation;
        model_rotated = true;
        // Light direction is NOT updated - it stays fixed in world space
    }
    
    // Light direction control (right mouse button)
    // Model rotation stays fixed - only light direction changes
    // Use quaternion rotation for smooth movement, same as model rotation
    if input.right_mouse_pressed && !input.left_mouse_pressed && input.mouse_delta.length_squared() > 0.0 {
        let sensitivity = 0.01;
        // Rotate light direction using quaternions for smooth rotation
        // Horizontal movement rotates around Y axis, vertical around X axis
        let rotation_y = glam::Quat::from_rotation_y(-input.mouse_delta.x * sensitivity);
        let rotation_x = glam::Quat::from_rotation_x(-input.mouse_delta.y * sensitivity);
        // Apply rotations to light direction vector
        let mut light_dir = render_state.app_state.light_params.direction;
        light_dir = rotation_x * rotation_y * light_dir;
        // Normalize to ensure it stays a unit vector
        light_dir = light_dir.normalize();
        
        // Update light direction immediately - this takes precedence over any material_changed updates
        render_state.app_state.light_params.direction = light_dir;
        render_state.render_pipeline.update_light_direction(queue, light_dir);
        // Model rotation is NOT updated - it stays fixed
    }
    
    // Scroll zoom
    if input.scroll_delta.abs() > 0.0 {
        let zoom_speed = 0.1;
        render_state.orbit_camera.zoom(-input.scroll_delta * zoom_speed);
    }
    
    // Update camera
    render_state.camera = render_state.orbit_camera.to_camera_with_aspect(render_state.camera.aspect);
    render_state.render_pipeline.update_camera(queue, &render_state.camera);
    
    // Update model matrix from rotation (only if model was rotated this frame)
    if model_rotated {
        let model_matrix = Mat4::from_quat(render_state.app_state.model_rotation);
        render_state.render_pipeline.update_model(queue, model_matrix);
    }
    
    // Reset frame input
    input.reset_frame();
}

fn render_frame(renderer: &mut Renderer, render_state: &mut RenderState, window: &Window) {
    match renderer.get_current_texture() {
        Ok(frame) => {
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Begin egui frame
            render_state.egui_state.begin_frame(window);
            
            // Build UI and get current panel width
            let panel_width = build_ui(&render_state.egui_state.context, &mut render_state.app_state);
            
            // Update camera aspect ratio if panel width changed
            let pixels_per_point = window.scale_factor() as f32;
            let panel_width_pixels = panel_width * pixels_per_point;
            let viewport_width = (renderer.size.width as f32 - panel_width_pixels).max(1.0);
            let new_aspect = viewport_width / renderer.size.height as f32;
            if (render_state.camera.aspect - new_aspect).abs() > 0.001 {
                render_state.camera.aspect = new_aspect;
                render_state.render_pipeline.update_camera(&renderer.queue, &render_state.camera);
            }
            
            // Handle texture loading if needed
            if render_state.app_state.textures_need_reload {
                use crate::texture_loader::{TextureLoader, TexturePaths, detect_textures_in_directory};
                
                // Build texture paths from folder detection and individual selections
                let mut texture_paths = TexturePaths::default();
                
                // First, detect textures from folder if provided
                if let Some(ref folder_path) = render_state.app_state.texture_folder {
                    if let Ok(detected) = detect_textures_in_directory(std::path::Path::new(folder_path)) {
                        // Use detected paths, but individual selections override folder detection
                        texture_paths = detected;
                    }
                }
                
                // Override with individually selected textures
                if let Some(ref path) = render_state.app_state.texture_handles.base_color {
                    texture_paths.base_color = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.normal {
                    texture_paths.normal = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.metallic {
                    texture_paths.metallic = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.roughness {
                    texture_paths.roughness = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.orm {
                    texture_paths.orm = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.ao {
                    texture_paths.ao = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.emissive {
                    texture_paths.emissive = Some(std::path::PathBuf::from(path));
                }
                if let Some(ref path) = render_state.app_state.texture_handles.height {
                    texture_paths.height = Some(std::path::PathBuf::from(path));
                }
                
                // Load textures from paths
                match TextureLoader::load_from_paths(
                    &renderer.device,
                    &renderer.queue,
                    &texture_paths,
                ) {
                    Ok(new_texture_set) => {
                        // Update texture bind group
                        let texture_bind_group_layout = TextureSet::bind_group_layout(&renderer.device);
                        render_state.texture_bind_group = new_texture_set.create_bind_group(
                            &renderer.device,
                            &texture_bind_group_layout,
                        );
                        
                        // Update loaded texture status based on what we actually loaded
                        render_state.app_state.loaded_textures.reset();
                        render_state.app_state.loaded_textures.base_color = texture_paths.base_color.is_some();
                        render_state.app_state.loaded_textures.normal = texture_paths.normal.is_some();
                        render_state.app_state.loaded_textures.metallic = texture_paths.metallic.is_some();
                        render_state.app_state.loaded_textures.roughness = texture_paths.roughness.is_some();
                        render_state.app_state.loaded_textures.orm = texture_paths.orm.is_some();
                        render_state.app_state.loaded_textures.ao = texture_paths.ao.is_some();
                        render_state.app_state.loaded_textures.emissive = texture_paths.emissive.is_some();
                        render_state.app_state.loaded_textures.height = texture_paths.height.is_some();
                        
                        // Also update texture_handles with detected paths from folder
                        if let Some(ref folder_path) = render_state.app_state.texture_folder {
                            if let Ok(detected) = detect_textures_in_directory(std::path::Path::new(folder_path)) {
                                if detected.base_color.is_some() && render_state.app_state.texture_handles.base_color.is_none() {
                                    render_state.app_state.texture_handles.base_color = detected.base_color.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.normal.is_some() && render_state.app_state.texture_handles.normal.is_none() {
                                    render_state.app_state.texture_handles.normal = detected.normal.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.metallic.is_some() && render_state.app_state.texture_handles.metallic.is_none() {
                                    render_state.app_state.texture_handles.metallic = detected.metallic.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.roughness.is_some() && render_state.app_state.texture_handles.roughness.is_none() {
                                    render_state.app_state.texture_handles.roughness = detected.roughness.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.orm.is_some() && render_state.app_state.texture_handles.orm.is_none() {
                                    render_state.app_state.texture_handles.orm = detected.orm.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.ao.is_some() && render_state.app_state.texture_handles.ao.is_none() {
                                    render_state.app_state.texture_handles.ao = detected.ao.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.emissive.is_some() && render_state.app_state.texture_handles.emissive.is_none() {
                                    render_state.app_state.texture_handles.emissive = detected.emissive.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                                if detected.height.is_some() && render_state.app_state.texture_handles.height.is_none() {
                                    render_state.app_state.texture_handles.height = detected.height.as_ref().and_then(|p| p.to_str().map(|s| s.to_string()));
                                }
                            }
                        }
                        
                        // Trigger material update to refresh view mode
                        render_state.app_state.material_changed = true;
                        
                        log::info!("Textures loaded successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to load textures: {}", e);
                    }
                }
                render_state.app_state.textures_need_reload = false;
            }
            
            // Handle mesh switching if needed
            if render_state.app_state.mesh_changed {
                let mesh_data = match render_state.app_state.current_mesh {
                    mesh_wgpu::MeshType::Sphere => create_sphere(render_state.app_state.tessellation_level),
                    mesh_wgpu::MeshType::Cube => create_cube(),
                    _ => create_sphere(32), // Fallback to sphere
                };
                render_state.mesh_buffer = MeshBuffer::new(&renderer.device, &mesh_data);
                render_state.app_state.mesh_changed = false;
            }
            
            // Update material if changed
            if render_state.app_state.material_changed {
                render_state.render_pipeline.update_material(
                    &renderer.queue,
                    &render_state.app_state.material_params,
                    render_state.app_state.view_mode,
                    &render_state.app_state.loaded_textures,
                );
                // Also update light direction when material changes (in case it was changed via UI sliders)
                render_state.render_pipeline.update_light_direction(
                    &renderer.queue,
                    render_state.app_state.light_params.direction,
                );
                render_state.app_state.material_changed = false;
            }
            
            // Ensure model matrix is always current (in case handle_camera_input wasn't called)
            // This is safe because model_rotation only changes when left mouse is pressed
            let model_matrix = Mat4::from_quat(render_state.app_state.model_rotation);
            render_state.render_pipeline.update_model(&renderer.queue, model_matrix);
            
            // End egui frame and get output
            let egui_output = render_state.egui_state.end_frame(window);
            let textures_delta = &egui_output.textures_delta;
            
            // Screen descriptor for egui rendering
            let pixels_per_point = window.scale_factor() as f32;
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [renderer.size.width, renderer.size.height],
                pixels_per_point,
            };
            
            // Tessellate shapes into primitives (this is the key conversion step)
            let egui_primitives = render_state.egui_state.tessellate(egui_output.shapes, pixels_per_point);
            
            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
            
            if !textures_delta.is_empty() {
                render_state.egui_state.update_texture(
                    &renderer.device,
                    &renderer.queue,
                    textures_delta,
                );
            }
            
            // Render 3D scene
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &renderer.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                
                // Set viewport to exclude UI panel area (render 3D to the right of the panel)
                // Use the dynamic panel width from the UI
                let viewport_x = panel_width_pixels;
                let viewport_height = renderer.size.height as f32;
                render_pass.set_viewport(viewport_x, 0.0, viewport_width, viewport_height, 0.0, 1.0);
                
                // Set render pipeline
                render_pass.set_pipeline(&render_state.render_pipeline.pipeline);
                
                // Set bind groups
                render_pass.set_bind_group(0, &render_state.render_pipeline.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &render_state.texture_bind_group, &[]);
                render_pass.set_bind_group(2, &render_state.render_pipeline.material_bind_group, &[]);
                
                // Set vertex buffer
                render_pass.set_vertex_buffer(0, render_state.mesh_buffer.vertex_buffer.slice(..));
                
                // Set index buffer and draw
                render_pass.set_index_buffer(render_state.mesh_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..render_state.mesh_buffer.index_count, 0, 0..1);
            }
            
            // Update egui buffers
            render_state.egui_state.update_buffers(
                &renderer.device,
                &renderer.queue,
                &mut encoder,
                &screen_descriptor,
                &egui_primitives,
            );
            
            // Render egui UI
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("UI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                
                render_state.egui_state.render(
                    &mut render_pass,
                    &egui_primitives,
                    &screen_descriptor,
                );
            }
            
            renderer.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
        }
        Err(wgpu::SurfaceError::Lost) => {
            renderer.resize(renderer.size);
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
            std::process::exit(1);
        }
        Err(e) => eprintln!("Surface error: {:?}", e),
    }
}
