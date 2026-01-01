//! egui UI implementation for wgpu

use egui::*;
use crate::state_wgpu::{AppState, ViewMode, TessellationDebugMode, UiTab, MIN_UI_PANEL_WIDTH, MAX_UI_PANEL_WIDTH};
use crate::mesh_wgpu::MeshType;

/// Build the egui UI with tabs and top bar
/// Returns the current panel width for viewport calculations
pub fn build_ui(ctx: &Context, state: &mut AppState) -> f32 {
    // Style the UI with a darker theme
    let mut style = (*ctx.style()).clone();
    style.visuals.window_fill = Color32::from_rgba_unmultiplied(18, 18, 24, 255);
    style.visuals.panel_fill = Color32::from_rgba_unmultiplied(18, 18, 24, 255);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(32, 32, 40);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 58);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 75);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(75, 75, 95);
    style.visuals.override_text_color = Some(Color32::from_rgb(220, 220, 220));
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    ctx.set_style(style);

    // Top bar for View Mode (always visible above the 3D viewport)
    TopBottomPanel::top("view_mode_bar")
        .frame(Frame::none().fill(Color32::from_rgba_unmultiplied(25, 25, 32, 240)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(state.ui_panel_width + 8.0); // Offset by panel width
                ui.label(RichText::new("View:").strong());
                ui.add_space(4.0);
                for mode in ViewMode::all() {
                    let selected = state.view_mode == *mode;
                    let text = if selected {
                        RichText::new(mode.name()).strong().color(Color32::from_rgb(100, 200, 255))
                    } else {
                        RichText::new(mode.name())
                    };
                    if ui.selectable_label(selected, text).clicked() {
                        state.view_mode = *mode;
                        state.material_changed = true;
                    }
                }
            });
        });

    // Side panel with tabs
    let panel_response = SidePanel::left("controls_panel")
        .resizable(true)
        .default_width(state.ui_panel_width)
        .width_range(MIN_UI_PANEL_WIDTH..=MAX_UI_PANEL_WIDTH)
        .show(ctx, |ui| {
            // Title
            ui.add_space(8.0);
            ui.heading(RichText::new("PBR Viewer").size(20.0).strong());
            ui.add_space(8.0);
            
            // Tab buttons
            ui.horizontal(|ui| {
                if ui.selectable_label(state.ui_tab == UiTab::Mesh, "📦 Mesh").clicked() {
                    state.ui_tab = UiTab::Mesh;
                }
                if ui.selectable_label(state.ui_tab == UiTab::Material, "🎨 Material").clicked() {
                    state.ui_tab = UiTab::Material;
                }
                if ui.selectable_label(state.ui_tab == UiTab::Light, "💡 Light").clicked() {
                    state.ui_tab = UiTab::Light;
                }
                if ui.selectable_label(state.ui_tab == UiTab::Textures, "📁 Textures").clicked() {
                    state.ui_tab = UiTab::Textures;
                }
            });
            
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            
            // Tab content with scroll area
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    match state.ui_tab {
                        UiTab::Mesh => build_mesh_tab(ui, state),
                        UiTab::Material => build_material_tab(ui, state),
                        UiTab::Light => build_light_tab(ui, state),
                        UiTab::Textures => build_textures_tab(ui, state),
                    }
                });
        });
    
    // Get the actual panel width and store it
    let panel_width = panel_response.response.rect.width();
    state.ui_panel_width = panel_width;
    panel_width
}

/// Build the Mesh tab content
fn build_mesh_tab(ui: &mut Ui, state: &mut AppState) {
    ui.heading(RichText::new("Mesh Selection").size(16.0));
    ui.add_space(8.0);
    
    ui.vertical(|ui| {
        for mesh_type in MeshType::primitives() {
            if ui.selectable_label(state.current_mesh == *mesh_type, mesh_type.name()).clicked() {
                state.current_mesh = *mesh_type;
                state.mesh_changed = true;
            }
        }
    });
    
    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);
    
    // Tessellation level for mesh
    ui.heading(RichText::new("Mesh Detail").size(14.0));
    ui.add_space(4.0);
    ui.label("Tessellation Level");
    if ui.add(Slider::new(&mut state.tessellation_level, 4..=128)).changed() {
        state.mesh_changed = true;
    }
    ui.label(RichText::new("Higher = more polygons").weak().small());
}

/// Build the Material tab content (includes tessellation settings)
fn build_material_tab(ui: &mut Ui, state: &mut AppState) {
    // Base Material Settings
    ui.heading(RichText::new("Material Properties").size(16.0));
    ui.add_space(8.0);
    
    ui.label("Base Color Tint");
    ui.horizontal(|ui| {
        if ui.color_edit_button_rgb(&mut state.material_params.base_color_tint).changed() {
            state.material_changed = true;
        }
        if ui.button("Reset").clicked() {
            state.material_params.base_color_tint = [0.8, 0.8, 0.8];
            state.material_changed = true;
        }
    });
    
    ui.add_space(4.0);
    ui.label("Metallic");
    if ui.add(Slider::new(&mut state.material_params.metallic_multiplier, 0.0..=1.0)).changed() {
        state.material_changed = true;
    }
    
    ui.label("Roughness");
    if ui.add(Slider::new(&mut state.material_params.roughness_multiplier, 0.0..=1.0)).changed() {
        state.material_changed = true;
    }
    
    ui.label("Normal Strength");
    if ui.add(Slider::new(&mut state.material_params.normal_strength, 0.0..=2.0)).changed() {
        state.material_changed = true;
    }
    
    ui.label("AO Strength");
    if ui.add(Slider::new(&mut state.material_params.ao_strength, 0.0..=2.0)).changed() {
        state.material_changed = true;
    }
    
    ui.label("Emissive Strength");
    if ui.add(Slider::new(&mut state.material_params.emissive_strength, 0.0..=5.0)).changed() {
        state.material_changed = true;
    }
    
    ui.add_space(8.0);
    ui.label("UV Tile Size");
    ui.label(RichText::new("Smaller = more repeats").weak().small());
    if ui.add(Slider::new(&mut state.material_params.uv_scale, 0.1..=5.0).logarithmic(true)).changed() {
        state.material_changed = true;
    }
    
    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);
    
    // GPU Tessellation Section
    ui.heading(RichText::new("GPU Tessellation").size(16.0));
    ui.add_space(4.0);
    
    if ui.checkbox(&mut state.gpu_tessellation.enabled, "Enable GPU Tessellation").changed() {
        state.material_changed = true;
    }
    ui.label(RichText::new("Requires DX12/Vulkan with tessellation support").weak().small());
    
    if state.gpu_tessellation.enabled {
        ui.add_space(8.0);
        
        ui.label("Min Tessellation Factor");
        if ui.add(Slider::new(&mut state.gpu_tessellation.min_tess_factor, 1.0..=16.0)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Max Tessellation Factor");
        if ui.add(Slider::new(&mut state.gpu_tessellation.max_tess_factor, 1.0..=128.0)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Quality Cap");
        if ui.add(Slider::new(&mut state.gpu_tessellation.quality_cap, 1.0..=128.0)).changed() {
            state.material_changed = true;
        }
        
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        
        // Displacement parameters
        ui.heading(RichText::new("Displacement").size(14.0));
        ui.add_space(4.0);
        
        ui.label("Displacement Scale");
        if ui.add(Slider::new(&mut state.gpu_tessellation.displacement_scale, 0.0..=1.0)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Displacement Midpoint");
        if ui.add(Slider::new(&mut state.gpu_tessellation.displacement_midpoint, 0.0..=1.0)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Displacement Bias");
        if ui.add(Slider::new(&mut state.gpu_tessellation.displacement_bias, -1.0..=1.0)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Clamp Min");
        if ui.add(Slider::new(&mut state.gpu_tessellation.displacement_clamp_min, -2.0..=0.0)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Clamp Max");
        if ui.add(Slider::new(&mut state.gpu_tessellation.displacement_clamp_max, 0.0..=2.0)).changed() {
            state.material_changed = true;
        }
        
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        
        // Tessellation quality settings
        ui.heading(RichText::new("Quality Settings").size(14.0));
        ui.add_space(4.0);
        
        ui.label("Screen Space Scale");
        if ui.add(Slider::new(&mut state.gpu_tessellation.screen_space_scale, 10.0..=500.0).logarithmic(true)).changed() {
            state.material_changed = true;
        }
        
        ui.label("Distance Scale");
        if ui.add(Slider::new(&mut state.gpu_tessellation.distance_scale, 0.01..=1.0).logarithmic(true)).changed() {
            state.material_changed = true;
        }
        
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        
        // Debug visualization
        ui.heading(RichText::new("Debug Visualization").size(14.0));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::None, "None").changed() {
                state.material_changed = true;
            }
            if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::TessellationDensity, "Density").changed() {
                state.material_changed = true;
            }
        });
        ui.horizontal(|ui| {
            if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::Wireframe, "Wireframe").changed() {
                state.material_changed = true;
            }
            if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::DisplacementOnly, "Displacement").changed() {
                state.material_changed = true;
            }
        });
    }
}

/// Build the Light tab content
fn build_light_tab(ui: &mut Ui, state: &mut AppState) {
    ui.heading(RichText::new("Light Settings").size(16.0));
    ui.add_space(8.0);
    
    // Light direction visualization
    ui.group(|ui| {
        let size = egui::vec2(120.0, 120.0);
        ui.set_min_size(size);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let light_dir = state.light_params.direction;
        
        let center = rect.center();
        let radius = (size.x.min(size.y) * 0.4).min(40.0);
        
        let painter = ui.painter();
        painter.circle_filled(center, radius, Color32::from_rgba_unmultiplied(40, 40, 50, 255));
        painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_rgb(80, 80, 100)));
        
        let theta = light_dir.x.atan2(light_dir.z);
        let phi = (light_dir.y / light_dir.length()).asin();
        
        let elevation_factor = phi.cos();
        let icon_radius = radius * 0.7 * elevation_factor;
        let icon_x = center.x + theta.cos() * icon_radius;
        let icon_y = center.y + theta.sin() * icon_radius;
        let icon_pos = egui::pos2(icon_x, icon_y);
        
        let icon_size = 12.0;
        painter.circle_filled(icon_pos, icon_size, Color32::from_rgb(255, 220, 100));
        painter.circle_stroke(icon_pos, icon_size, Stroke::new(2.0, Color32::from_rgb(255, 180, 50)));
        
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
            let ray_start = icon_pos + egui::Vec2::angled(angle) * icon_size;
            let ray_end = icon_pos + egui::Vec2::angled(angle) * (icon_size + 4.0);
            painter.line_segment([ray_start, ray_end], Stroke::new(1.5, Color32::from_rgb(255, 200, 80)));
        }
        
        let arrow_start = icon_pos;
        let arrow_end = center;
        let arrow_dir = (arrow_end - arrow_start).normalized();
        let arrow_length = (arrow_end - arrow_start).length();
        
        if arrow_length < radius * 1.5 {
            painter.line_segment([arrow_start, arrow_end], Stroke::new(2.0, Color32::from_rgb(100, 200, 255)));
            
            let arrow_head_size = 8.0;
            let perp = egui::Vec2::new(-arrow_dir.y, arrow_dir.x);
            let arrow_tip = arrow_end;
            let arrow_base = arrow_end - arrow_dir * arrow_head_size;
            let arrow_left = arrow_base + perp * (arrow_head_size * 0.5);
            let arrow_right = arrow_base - perp * (arrow_head_size * 0.5);
            painter.line_segment([arrow_tip, arrow_left], Stroke::new(2.0, Color32::from_rgb(100, 200, 255)));
            painter.line_segment([arrow_tip, arrow_right], Stroke::new(2.0, Color32::from_rgb(100, 200, 255)));
            painter.line_segment([arrow_left, arrow_right], Stroke::new(2.0, Color32::from_rgb(100, 200, 255)));
        }
        
        ui.add_space(radius * 2.0 + 20.0);
        ui.label(RichText::new("Light Direction (Right-click to rotate)").small().weak());
    });
    
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    
    ui.label("Direction (Manual)");
    let mut dir_changed = false;
    ui.horizontal(|ui| {
        if ui.add(Slider::new(&mut state.light_params.direction.x, -1.0..=1.0).text("X")).changed() {
            dir_changed = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.add(Slider::new(&mut state.light_params.direction.y, -1.0..=1.0).text("Y")).changed() {
            dir_changed = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.add(Slider::new(&mut state.light_params.direction.z, -1.0..=1.0).text("Z")).changed() {
            dir_changed = true;
        }
    });
    
    if dir_changed {
        let dir = state.light_params.direction;
        if dir.length_squared() > 0.0001 {
            state.light_params.direction = dir.normalize();
            state.material_changed = true;
        }
    }
    
    ui.add_space(8.0);
    ui.label("Intensity");
    if ui.add(Slider::new(&mut state.light_params.intensity, 0.0..=50.0)).changed() {
        state.material_changed = true;
    }
    
    ui.label("Ambient Intensity");
    if ui.add(Slider::new(&mut state.light_params.ambient_intensity, 0.0..=2.0)).changed() {
        state.material_changed = true;
    }
}

/// Build the Textures tab content
fn build_textures_tab(ui: &mut Ui, state: &mut AppState) {
    ui.heading(RichText::new("Texture Loading").size(16.0));
    ui.add_space(8.0);
    
    // Load folder button
    if ui.button("📂 Load Texture Folder").clicked() {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select Texture Folder")
            .pick_folder()
        {
            state.texture_folder = Some(folder.to_string_lossy().to_string());
            state.textures_need_reload = true;
        }
    }
    
    if let Some(ref folder) = state.texture_folder {
        ui.label(RichText::new(format!("📂 {}", folder)).small());
    } else {
        ui.label(RichText::new("No texture folder loaded").weak().small());
    }
    
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    
    // Individual texture loading
    ui.heading(RichText::new("Individual Textures").size(14.0));
    ui.add_space(4.0);
    
    // Helper macro for texture rows
    macro_rules! texture_row {
        ($ui:expr, $state:expr, $label:expr, $checked:expr, $handle:ident) => {
            $ui.horizontal(|ui| {
                let checkbox_symbol = if $checked { "✓" } else { "☐" };
                ui.label(RichText::new(format!("{} {}", checkbox_symbol, $label)).size(14.0));
                
                if let Some(name) = $state.texture_handles.get_file_name(stringify!($handle)) {
                    ui.label(RichText::new(format!("({})", name)).weak().small());
                } else {
                    ui.label(RichText::new("(none)").weak().small());
                }
                
                if ui.small_button("📄").clicked() {
                    if let Some(file) = rfd::FileDialog::new()
                        .set_title(&format!("Select {} Texture", $label))
                        .add_filter("Image", &["png", "jpg", "jpeg", "tga", "bmp", "dds"])
                        .pick_file()
                    {
                        $state.texture_handles.$handle = Some(file.to_string_lossy().to_string());
                        $state.textures_need_reload = true;
                    }
                }
            });
        };
    }
    
    texture_row!(ui, state, "Base Color", state.loaded_textures.base_color, base_color);
    texture_row!(ui, state, "Normal", state.loaded_textures.normal, normal);
    texture_row!(ui, state, "Metallic", state.loaded_textures.metallic || state.loaded_textures.orm, metallic);
    texture_row!(ui, state, "Roughness", state.loaded_textures.roughness || state.loaded_textures.orm, roughness);
    texture_row!(ui, state, "ORM", state.loaded_textures.orm, orm);
    texture_row!(ui, state, "AO", state.loaded_textures.ao || state.loaded_textures.orm, ao);
    texture_row!(ui, state, "Emissive", state.loaded_textures.emissive, emissive);
    texture_row!(ui, state, "Height", state.loaded_textures.height, height);
    
    ui.add_space(12.0);
    
    // Clear all textures button
    if ui.button("🗑 Clear All Textures").clicked() {
        state.texture_folder = None;
        state.texture_handles = Default::default();
        state.loaded_textures.reset();
        state.textures_need_reload = true;
    }
}
