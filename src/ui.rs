//! egui UI implementation

use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_egui::{egui, EguiContexts};
use crate::mesh::MeshType;
use crate::state::{AppState, ViewMode, TessellationDebugMode};

/// Build the egui UI
pub fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    let ctx = contexts.ctx_mut();
    
    // Check if mouse is over UI area
    let pointer_over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();
    
    // Handle model rotation with left mouse button (only when not over UI)
    if !pointer_over_ui {
        if mouse_button.just_pressed(MouseButton::Left) {
            state.is_rotating_model = true;
        }
        if mouse_button.just_released(MouseButton::Left) {
            state.is_rotating_model = false;
        }
        
        if state.is_rotating_model {
            for event in mouse_motion.read() {
                let sensitivity = 0.005;
                let rotation_y = Quat::from_rotation_y(event.delta.x * sensitivity);
                let rotation_x = Quat::from_rotation_x(event.delta.y * sensitivity);
                state.model_rotation = rotation_y * state.model_rotation * rotation_x;
            }
        }
    } else {
        state.is_rotating_model = false;
        mouse_motion.clear();
    }

    // Style the UI with a darker theme
    let mut style = (*ctx.style()).clone();
    style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(18, 18, 24, 245);
    style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(18, 18, 24, 245);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(32, 32, 40);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 58);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 65, 85);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(85, 105, 150);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(70, 90, 130);
    ctx.set_style(style);

    // Left panel - Main controls
    egui::SidePanel::left("controls_panel")
        .default_width(320.0)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(8.0);
                ui.heading(egui::RichText::new("🎨 PBR Texture Viewer").size(22.0).strong());
                ui.separator();

                // ─────────────────────────────────────────────────────────────
                // Texture Loading Section
                // ─────────────────────────────────────────────────────────────
                ui.add_space(12.0);
                ui.heading(egui::RichText::new("📁 Texture Set").size(16.0));

                if let Some(ref folder) = state.texture_folder {
                    ui.horizontal(|ui| {
                        ui.label("📂");
                        ui.label(egui::RichText::new(folder).weak().italics());
                    });
                } else {
                    ui.label(egui::RichText::new("No texture set loaded").weak());
                }

                ui.add_space(4.0);
                if ui.button("📂 Open Folder...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_title("Select Texture Folder")
                        .pick_folder()
                    {
                        state.texture_folder = Some(folder.to_string_lossy().to_string());
                        state.textures_need_reload = true;
                    }
                }

                ui.label(egui::RichText::new("Tip: Drag & drop a texture file to load its folder").weak().small());

                // Show loaded textures status
                if state.texture_folder.is_some() {
                    ui.add_space(4.0);
                    ui.group(|ui| {
                        ui.label("Detected textures:");
                        ui.horizontal_wrapped(|ui| {
                            let tex = &state.loaded_textures;
                            if tex.base_color {
                                ui.label(egui::RichText::new("✓ Color").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                            if tex.normal {
                                ui.label(egui::RichText::new("✓ Normal").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                            if tex.roughness || tex.orm {
                                ui.label(egui::RichText::new("✓ Rough").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                            if tex.metallic || tex.orm {
                                ui.label(egui::RichText::new("✓ Metal").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                            if tex.ao || tex.orm {
                                ui.label(egui::RichText::new("✓ AO").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                            if tex.emissive {
                                ui.label(egui::RichText::new("✓ Emissive").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                            if tex.height {
                                ui.label(egui::RichText::new("✓ Height").color(egui::Color32::from_rgb(100, 200, 100)).small());
                            }
                        });
                    });
                }

                ui.add_space(16.0);
                ui.separator();

                // ─────────────────────────────────────────────────────────────
                // Mesh Selection
                // ─────────────────────────────────────────────────────────────
                ui.add_space(12.0);
                ui.heading(egui::RichText::new("🔷 Mesh").size(16.0));

                ui.horizontal(|ui| {
                    for mesh_type in MeshType::primitives() {
                        let selected = state.current_mesh == *mesh_type && !state.using_custom_model;
                        if ui.selectable_label(selected, mesh_type.name()).clicked() {
                            state.current_mesh = *mesh_type;
                            state.using_custom_model = false;
                            state.mesh_changed = true;
                        }
                    }
                });
                
                // Custom model section
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let custom_selected = state.using_custom_model;
                    if ui.selectable_label(custom_selected, "📦 Custom").clicked() && state.custom_model_path.is_some() {
                        state.using_custom_model = true;
                        state.mesh_changed = true;
                    }
                    
                    if ui.button("Load Model...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select 3D Model")
                            .add_filter("3D Models", &["gltf", "glb", "obj"])
                            .pick_file()
                        {
                            state.custom_model_path = Some(path.to_string_lossy().to_string());
                            state.custom_model_needs_load = true;
                            state.using_custom_model = true;
                            state.current_mesh = MeshType::Custom;
                            state.mesh_changed = true;
                        }
                    }
                });
                
                if let Some(ref path) = state.custom_model_path {
                    let file_name = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    ui.label(egui::RichText::new(format!("Model: {}", file_name)).weak().small());
                }

                // Tessellation level
                ui.add_space(8.0);
                let mut tess = state.tessellation_level as i32;
                if ui.add(egui::Slider::new(&mut tess, 4..=128).text("Tessellation")).changed() {
                    state.tessellation_level = tess as u32;
                    state.mesh_changed = true;
                }

                ui.add_space(8.0);
                if ui.button("🔄 Reset Rotation").clicked() {
                    state.model_rotation = Quat::IDENTITY;
                }

                ui.add_space(16.0);
                ui.separator();

                // ─────────────────────────────────────────────────────────────
                // View Mode
                // ─────────────────────────────────────────────────────────────
                ui.add_space(12.0);
                ui.heading(egui::RichText::new("👁 View Mode").size(16.0));

                egui::Grid::new("view_mode_grid")
                    .num_columns(4)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        for (i, mode) in ViewMode::all().iter().enumerate() {
                            let selected = state.view_mode == *mode;
                            if ui.selectable_label(selected, mode.name()).clicked() {
                                state.view_mode = *mode;
                                state.material_changed = true;
                            }
                            if (i + 1) % 4 == 0 {
                                ui.end_row();
                            }
                        }
                    });

                ui.add_space(16.0);
                ui.separator();

                // ─────────────────────────────────────────────────────────────
                // Material Parameters (only shown in Lit mode)
                // ─────────────────────────────────────────────────────────────
                ui.add_space(12.0);
                ui.heading(egui::RichText::new("⚙ Material Parameters").size(16.0));
                
                if state.view_mode != ViewMode::Lit {
                    ui.label(egui::RichText::new("(Only active in Lit mode)").weak().italics());
                }

                // Base Color Tint
                ui.horizontal(|ui| {
                    ui.label("Base Color Tint");
                    let mut color = state.material_params.base_color_tint;
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        state.material_params.base_color_tint = color;
                        state.material_changed = true;
                    }
                });

                ui.add_space(8.0);

                // Metallic
                ui.label("Metallic");
                if ui.add(egui::Slider::new(&mut state.material_params.metallic_multiplier, 0.0..=1.0)).changed() {
                    state.material_changed = true;
                }

                // Roughness
                ui.label("Roughness");
                if ui.add(egui::Slider::new(&mut state.material_params.roughness_multiplier, 0.0..=1.0)).changed() {
                    state.material_changed = true;
                }

                // Normal Strength (note: Bevy doesn't directly support this, but we keep it for future use)
                ui.label("Normal Strength");
                ui.add(egui::Slider::new(&mut state.material_params.normal_strength, 0.0..=2.0));

                // AO Strength (note: Bevy doesn't directly support this, but we keep it for future use)
                ui.label("AO Strength");
                ui.add(egui::Slider::new(&mut state.material_params.ao_strength, 0.0..=2.0));

                // Emissive Strength
                ui.label("Emissive Strength");
                if ui.add(egui::Slider::new(&mut state.material_params.emissive_strength, 0.0..=5.0)).changed() {
                    state.material_changed = true;
                }

                // Displacement Strength (Parallax)
                ui.label("Parallax Depth");
                if ui.add(egui::Slider::new(&mut state.material_params.displacement_strength, 0.0..=0.2)).changed() {
                    state.material_changed = true;
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                // UV Scale
                ui.heading(egui::RichText::new("📐 UV Mapping").size(16.0));
                ui.label("UV Scale");
                if ui.add(egui::Slider::new(&mut state.material_params.uv_scale, 0.1..=10.0).logarithmic(true)).changed() {
                    state.material_changed = true;
                }
                ui.label(egui::RichText::new("Tip: Adjust to tile/scale textures on the model").weak().small());

                ui.add_space(16.0);
                ui.separator();

                // ─────────────────────────────────────────────────────────────
                // GPU Tessellation & Displacement
                // ─────────────────────────────────────────────────────────────
                ui.add_space(12.0);
                ui.heading(egui::RichText::new("🔷 GPU Tessellation").size(16.0));
                
                if ui.checkbox(&mut state.gpu_tessellation.enabled, "Enable GPU Tessellation").changed() {
                    state.material_changed = true;
                }
                ui.label(egui::RichText::new("Note: Requires DX12/Vulkan with tessellation support").weak().small());
                
                if state.gpu_tessellation.enabled {
                    ui.add_space(8.0);
                    
                    // Tessellation factors
                    ui.label("Min Tessellation Factor");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.min_tess_factor, 1.0..=16.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Max Tessellation Factor");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.max_tess_factor, 1.0..=128.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Quality Cap");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.quality_cap, 1.0..=128.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Displacement parameters
                    ui.heading(egui::RichText::new("Displacement").size(14.0));
                    
                    ui.label("Displacement Scale");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.displacement_scale, 0.0..=1.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Displacement Midpoint");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.displacement_midpoint, 0.0..=1.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Displacement Bias");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.displacement_bias, -1.0..=1.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Clamp Min");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.displacement_clamp_min, -2.0..=0.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Clamp Max");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.displacement_clamp_max, 0.0..=2.0)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Tessellation quality settings
                    ui.heading(egui::RichText::new("Quality Settings").size(14.0));
                    
                    ui.label("Screen Space Scale");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.screen_space_scale, 10.0..=500.0).logarithmic(true)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.label("Distance Scale");
                    if ui.add(egui::Slider::new(&mut state.gpu_tessellation.distance_scale, 0.01..=1.0).logarithmic(true)).changed() {
                        state.material_changed = true;
                    }
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Debug visualization
                    ui.heading(egui::RichText::new("Debug Visualization").size(14.0));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::None, "None");
                        ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::TessellationDensity, "Tess Density");
                        ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::Wireframe, "Wireframe");
                        ui.selectable_value(&mut state.gpu_tessellation.debug_visualization, TessellationDebugMode::DisplacementOnly, "Displacement");
                    });
                }

                ui.add_space(16.0);
                ui.separator();

                // ─────────────────────────────────────────────────────────────
                // Lighting
                // ─────────────────────────────────────────────────────────────
                ui.add_space(12.0);
                ui.heading(egui::RichText::new("💡 Lighting").size(16.0));

                // Light color
                ui.horizontal(|ui| {
                    ui.label("Light Color");
                    ui.color_edit_button_rgb(&mut state.light_params.color);
                });

                // Light intensity
                ui.label("Light Intensity");
                ui.add(egui::Slider::new(&mut state.light_params.intensity, 0.0..=30.0));

                // Ambient intensity
                ui.label("Ambient Intensity");
                ui.add(egui::Slider::new(&mut state.light_params.ambient_intensity, 0.0..=1.0));

                // Light direction
                ui.label("Light Direction");
                
                let dir = state.light_params.direction;
                let mut yaw = dir.x.atan2(dir.z);
                let mut pitch = dir.y.asin();

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Yaw:");
                    if ui.add(egui::Slider::new(&mut yaw, -std::f32::consts::PI..=std::f32::consts::PI).show_value(false)).changed() {
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Pitch:");
                    if ui.add(egui::Slider::new(&mut pitch, -std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2).show_value(false)).changed() {
                        changed = true;
                    }
                });

                if changed {
                    state.light_params.direction = Vec3::new(
                        pitch.cos() * yaw.sin(),
                        pitch.sin(),
                        pitch.cos() * yaw.cos(),
                    ).normalize();
                }

                ui.add_space(20.0);
            });
        });

    // Bottom panel - Help
    egui::TopBottomPanel::bottom("help_panel")
        .exact_height(32.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("Controls:").strong());
                ui.separator();
                ui.label("🖱 Left drag = Rotate model");
                ui.separator();
                ui.label("🖱 Right drag = Orbit camera");
                ui.separator();
                ui.label("🖱 Scroll = Zoom");
                ui.separator();
                ui.label("📁 Drop texture = Load set");
            });
        });
    
    // Show full-screen drag-and-drop overlay when hovering with a file
    if let Some(ref path) = state.drag_hover_path {
        // Get the folder that will be loaded
        let display_path = std::path::Path::new(path);
        let folder_name = if display_path.is_file() {
            display_path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("folder")
        } else {
            display_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("folder")
        };
        
        let file_name = display_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        
        // Full screen semi-transparent overlay
        let screen_rect = ctx.screen_rect();
        
        egui::Area::new(egui::Id::new("drop_overlay_bg"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                // Draw full-screen background
                let painter = ui.painter();
                painter.rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(15, 40, 80, 180),
                );
                
                // Draw dashed border around viewport
                let border_rect = screen_rect.shrink(20.0);
                painter.rect_stroke(
                    border_rect,
                    12.0,
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(80, 160, 255)),
                );
            });
        
        // Centered drop indicator
        egui::Area::new(egui::Id::new("drop_overlay_content"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(25, 70, 140, 245))
                    .rounding(20.0)
                    .inner_margin(48.0)
                    .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 200, 255)))
                    .shadow(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
                    })
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            // Drop/Plus icon
                            ui.label(egui::RichText::new("⊕").size(72.0).color(egui::Color32::from_rgb(120, 220, 255)));
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new("Drop to Load Textures").size(24.0).strong().color(egui::Color32::WHITE));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(format!("📄 {}", file_name)).size(16.0).color(egui::Color32::from_rgb(180, 210, 255)));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(format!("📁 {}", folder_name)).size(14.0).color(egui::Color32::from_rgb(150, 180, 220)));
                        });
                    });
            });
    }
}
