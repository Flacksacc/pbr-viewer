//! PBR Texture Set Viewer - wgpu version

mod renderer;
mod camera_wgpu;
mod mesh_wgpu;
mod state_wgpu;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use renderer::Renderer;
use state_wgpu::AppState;
use camera_wgpu::OrbitCamera;

// Silence warnings for now
#[allow(unused_imports)]
#[allow(unused_variables)]

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    log::info!("PBR Texture Viewer started!");
    
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("PBR Texture Viewer")
        .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0))
        .build(&event_loop)?;
    
    let mut renderer = pollster::block_on(async {
        Renderer::new(&window).await
    })?;
    
    let mut app_state = AppState::default();
    let mut orbit_camera = OrbitCamera::new(glam::Vec3::ZERO, 3.0);
    
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    render_frame(&mut renderer);
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}

fn render_frame(renderer: &mut Renderer) {
    match renderer.get_current_texture() {
        Ok(frame) => {
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
            
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

