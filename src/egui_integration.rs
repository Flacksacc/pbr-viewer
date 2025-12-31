//! egui-wgpu integration

use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiWinitState;
use winit::window::Window;

/// egui integration state
pub struct EguiState {
    pub context: egui::Context,
    pub winit_state: EguiWinitState,
    pub renderer: EguiRenderer,
}

impl EguiState {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        window: &Window,
    ) -> Self {
        let context = egui::Context::default();
        let winit_state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
        );
        
        // Convert wgpu types to egui-wgpu types
        // Since egui-wgpu 0.28 uses wgpu 0.20 internally, we need to use its re-exported types
        let egui_device: &egui_wgpu::wgpu::Device = unsafe { std::mem::transmute(device) };
        let renderer = EguiRenderer::new(
            egui_device,
            surface_config.format,
            None,
            1,
        );
        
        Self {
            context,
            winit_state,
            renderer,
        }
    }
    
    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        self.winit_state.on_window_event(window, event).consumed
    }
    
    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.winit_state.take_egui_input(window);
        self.context.begin_frame(raw_input);
    }
    
    pub fn end_frame(&mut self, _window: &Window) -> egui::FullOutput {
        self.context.end_frame()
    }
    
    /// Tessellate shapes into primitives that can be rendered
    pub fn tessellate(&self, shapes: Vec<epaint::ClippedShape>, pixels_per_point: f32) -> Vec<epaint::ClippedPrimitive> {
        self.context.tessellate(shapes, pixels_per_point)
    }
    
    pub fn update_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: &egui::TexturesDelta,
    ) {
        // Convert to egui-wgpu types
        let egui_device: &egui_wgpu::wgpu::Device = unsafe { std::mem::transmute(device) };
        let egui_queue: &egui_wgpu::wgpu::Queue = unsafe { std::mem::transmute(queue) };
        
        // Now that versions match, we can properly update textures
        for (id, image_delta) in &delta.set {
            self.renderer.update_texture(egui_device, egui_queue, *id, image_delta);
        }
        for id in &delta.free {
            self.renderer.free_texture(id);
        }
    }
    
    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        primitives: &[epaint::ClippedPrimitive],
    ) {
        // Convert to egui-wgpu types
        let egui_device: &egui_wgpu::wgpu::Device = unsafe { std::mem::transmute(device) };
        let egui_queue: &egui_wgpu::wgpu::Queue = unsafe { std::mem::transmute(queue) };
        let egui_encoder: &mut egui_wgpu::wgpu::CommandEncoder = unsafe { std::mem::transmute(encoder) };
        
        // epaint::ClippedPrimitive should be compatible with egui-wgpu's expected type
        // They're from the same egui/epaint version, so we can use them directly
        self.renderer.update_buffers(egui_device, egui_queue, egui_encoder, primitives, screen_descriptor);
    }
    
    pub fn render<'rp>(
        &'rp mut self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        primitives: &'rp [epaint::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        // Convert to egui-wgpu type
        let egui_render_pass: &mut egui_wgpu::wgpu::RenderPass<'rp> = unsafe { std::mem::transmute(render_pass) };
        
        // Render the primitives
        self.renderer.render(egui_render_pass, primitives, screen_descriptor);
    }
}
