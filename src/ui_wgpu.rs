//! egui UI implementation for wgpu

use egui::*;
use crate::state_wgpu::{AppState, ViewMode, TessellationDebugMode};
use crate::mesh_wgpu::MeshType;

/// Build the egui UI
pub fn build_ui(ctx: &Context, state: &mut AppState) {
    // Style the UI with a darker theme
    let mut style = (*ctx.style()).clone();
    style.visuals.window_fill = Color32::from_rgba_unmultiplied(18, 18, 24, 245);
    style.visuals.panel_fill = Color32::from_rgba_unmultiplied(18, 18, 24, 245);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(32, 32, 40);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 58);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 75);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(75, 75, 95);
    style.visuals.override_text_color = Some(Color32::from_rgb(220, 220, 220));
    ctx.set_style(style);

    // Main control panel
    Window::new("PBR Viewer Controls")
        .default_pos([10.0, 10.0])
        .default_size([300.0, 600.0])
        .collapsible(true)
        .show(ctx, |ui| {
            ui.set_min_width(280.0);
            
            // Make the content scrollable
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
            
            // Mesh Selection
            ui.add_space(8.0);
            ui.heading(RichText::new("📦 Mesh").size(16.0));
            ui.horizontal(|ui| {
                for mesh_type in MeshType::primitives() {
                    if ui.selectable_label(state.current_mesh == *mesh_type, mesh_type.name()).clicked() {
                        state.current_mesh = *mesh_type;
                        state.mesh_changed = true;
                    }
                }
            });
            
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            
            // View Mode
            ui.heading(RichText::new("👁️ View Mode").size(16.0));
            ui.horizontal_wrapped(|ui| {
                for mode in ViewMode::all() {
                    if ui.selectable_label(state.view_mode == *mode, mode.name()).clicked() {
                        state.view_mode = *mode;
                        state.material_changed = true;  // This will trigger update_material with new view_mode
                    }
                }
            });
            
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            
            // Material Parameters
            ui.heading(RichText::new("🎨 Material").size(16.0));
            
            ui.label("Base Color Tint");
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut state.material_params.base_color_tint);
                if ui.button("Reset").clicked() {
                    state.material_params.base_color_tint = [0.8, 0.8, 0.8];
                    state.material_changed = true;
                }
            });
            
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
            
            ui.label("UV Tile Size");
            ui.label(RichText::new("Smaller = more repeats (finer pattern)").weak().small());
            if ui.add(Slider::new(&mut state.material_params.uv_scale, 0.1..=5.0).logarithmic(true)).changed() {
                state.material_changed = true;
            }
            
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            
            // GPU Tessellation Section
            ui.heading(RichText::new("🔷 GPU Tessellation").size(16.0));
            
            if ui.checkbox(&mut state.gpu_tessellation.enabled, "Enable GPU Tessellation").changed() {
                state.material_changed = true;
            }
            ui.label(RichText::new("Note: Requires DX12/Vulkan with tessellation support").weak().small());
            
            if state.gpu_tessellation.enabled {
                ui.add_space(8.0);
                
                // Tessellation factors
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
                
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                
                // Displacement parameters
                ui.heading(RichText::new("Displacement").size(14.0));
                
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
                
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                
                // Tessellation quality settings
                ui.heading(RichText::new("Quality Settings").size(14.0));
                
                ui.label("Screen Space Scale");
                if ui.add(Slider::new(&mut state.gpu_tessellation.screen_space_scale, 10.0..=500.0).logarithmic(true)).changed() {
                    state.material_changed = true;
                }
                
                ui.label("Distance Scale");
                if ui.add(Slider::new(&mut state.gpu_tessellation.distance_scale, 0.01..=1.0).logarithmic(true)).changed() {
                    state.material_changed = true;
                }
                
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                
                // Debug visualization
                ui.heading(RichText::new("Debug Visualization").size(14.0));
                ui.horizontal(|ui| {
                    if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::None, "None").changed() {
                        state.material_changed = true;
                    }
                    if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::TessellationDensity, "Tess Density").changed() {
                        state.material_changed = true;
                    }
                    if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::Wireframe, "Wireframe").changed() {
                        state.material_changed = true;
                    }
                    if ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::DisplacementOnly, "Displacement").changed() {
                        state.material_changed = true;
                    }
                });
            }
            
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            
            // Light Parameters
            ui.heading(RichText::new("💡 Light").size(16.0));
            
            ui.label("Direction");
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut state.light_params.direction.x, -1.0..=1.0).text("X"));
                ui.add(Slider::new(&mut state.light_params.direction.y, -1.0..=1.0).text("Y"));
                ui.add(Slider::new(&mut state.light_params.direction.z, -1.0..=1.0).text("Z"));
            });
            
            ui.label("Intensity");
            if ui.add(Slider::new(&mut state.light_params.intensity, 0.0..=50.0)).changed() {
                state.material_changed = true;
            }
            
            ui.label("Ambient Intensity");
            if ui.add(Slider::new(&mut state.light_params.ambient_intensity, 0.0..=2.0)).changed() {
                state.material_changed = true;
            }
            
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            
            // Texture Loading
            ui.heading(RichText::new("📁 Textures").size(16.0));
            
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
            
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            
            // Individual texture loading
            ui.heading(RichText::new("Individual Textures").size(14.0));
            
            // Helper macro to create texture row
            macro_rules! texture_row {
                ($ui:expr, $state:expr, $label:expr, $checked:expr, $handle:ident) => {
                    $ui.horizontal(|ui| {
                        // Show checkbox state (read-only indicator using symbol)
                        let checkbox_symbol = if $checked { "✓" } else { "☐" };
                        ui.label(RichText::new(format!("{} {}", checkbox_symbol, $label)).size(14.0));
                        
                        // File name display
                        if let Some(name) = $state.texture_handles.get_file_name(stringify!($handle)) {
                            ui.label(RichText::new(format!("({})", name)).weak().small());
                        } else {
                            ui.label(RichText::new("(none)").weak().small());
                        }
                        
                        // Load button
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
                }); // End ScrollArea
        });
}

