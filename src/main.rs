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
use state_wgpu::AppState as WgpuAppState;
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
    let mut egui_state = EguiState::new(
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
    
    // Camera setup
    let mut orbit_camera = OrbitCamera::new(glam::Vec3::ZERO, 3.0);
    let aspect = renderer.size.width as f32 / renderer.size.height as f32;
    let camera = orbit_camera.to_camera_with_aspect(aspect);
    render_pipeline.update_camera(&renderer.queue, &camera);
    
    // Model transform
    let model_matrix = Mat4::IDENTITY;
    render_pipeline.update_model(&renderer.queue, model_matrix);
    
    // Material params
    let mut app_state = WgpuAppState::default();
    render_pipeline.update_material(&renderer.queue, &app_state.material_params);
    
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
                        // Update camera aspect
                        render_state.camera.aspect = physical_size.width as f32 / physical_size.height as f32;
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
    
    // Model rotation (left mouse button)
    if input.left_mouse_pressed && input.mouse_delta.length_squared() > 0.0 {
        let sensitivity = 0.005;
        let rotation_y = glam::Quat::from_rotation_y(-input.mouse_delta.x * sensitivity);
        let rotation_x = glam::Quat::from_rotation_x(-input.mouse_delta.y * sensitivity);
        render_state.app_state.model_rotation = rotation_y * render_state.app_state.model_rotation * rotation_x;
    }
    
    // Camera rotation (right mouse button)
    if input.right_mouse_pressed && input.mouse_delta.length_squared() > 0.0 {
        let sensitivity = 0.005;
        let delta_yaw = -input.mouse_delta.x * sensitivity;
        let delta_pitch = -input.mouse_delta.y * sensitivity;
        render_state.orbit_camera.rotate(delta_yaw, delta_pitch);
    }
    
    // Scroll zoom
    if input.scroll_delta.abs() > 0.0 {
        let zoom_speed = 0.1;
        render_state.orbit_camera.zoom(-input.scroll_delta * zoom_speed);
    }
    
    // Update camera
    render_state.camera = render_state.orbit_camera.to_camera_with_aspect(render_state.camera.aspect);
    render_state.render_pipeline.update_camera(queue, &render_state.camera);
    
    // Update model matrix from rotation
    let model_matrix = Mat4::from_quat(render_state.app_state.model_rotation);
    render_state.render_pipeline.update_model(queue, model_matrix);
    
    // Reset frame input
    input.reset_frame();
}

fn render_frame(renderer: &mut Renderer, render_state: &mut RenderState, window: &Window) {
    match renderer.get_current_texture() {
        Ok(frame) => {
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Begin egui frame
            render_state.egui_state.begin_frame(window);
            
            // Build UI
            build_ui(&render_state.egui_state.context, &mut render_state.app_state);
            
            // Handle texture loading if needed
            if render_state.app_state.textures_need_reload {
                if let Some(ref folder_path) = render_state.app_state.texture_folder {
                    match crate::texture_loader::TextureLoader::load_from_directory(
                        &renderer.device,
                        &renderer.queue,
                        std::path::Path::new(folder_path),
                    ) {
                        Ok(new_texture_set) => {
                            // Update texture bind group
                            let texture_bind_group_layout = TextureSet::bind_group_layout(&renderer.device);
                            render_state.texture_bind_group = new_texture_set.create_bind_group(
                                &renderer.device,
                                &texture_bind_group_layout,
                            );
                            
                            // Update loaded texture status
                            render_state.app_state.loaded_textures.reset();
                            // Check which textures were actually loaded
                            // For now, we'll assume all are loaded if the folder was selected
                            render_state.app_state.loaded_textures.base_color = true;
                            render_state.app_state.loaded_textures.normal = true;
                            render_state.app_state.loaded_textures.roughness = true;
                            render_state.app_state.loaded_textures.metallic = true;
                            
                            log::info!("Textures loaded from: {}", folder_path);
                        }
                        Err(e) => {
                            log::error!("Failed to load textures from {}: {}", folder_path, e);
                        }
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
                );
                render_state.app_state.material_changed = false;
            }
            
            // Always update model matrix from rotation (ensures it's current even if handle_camera_input wasn't called)
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
