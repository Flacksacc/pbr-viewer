//! Orbit camera controller for Bevy

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

/// Plugin for orbit camera functionality
pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        // Run camera system after UI to ensure egui has processed pointer state
        app.add_systems(Update, orbit_camera_system.after(crate::ui::ui_system));
    }
}

/// Component for orbit camera control
#[derive(Component)]
pub struct OrbitCameraController {
    /// Target point to orbit around
    pub target: Vec3,
    /// Distance from target
    pub distance: f32,
    /// Horizontal rotation (yaw)
    pub yaw: f32,
    /// Vertical rotation (pitch)
    pub pitch: f32,
    /// Rotation sensitivity
    pub sensitivity: f32,
    /// Zoom sensitivity
    pub zoom_sensitivity: f32,
    /// Minimum distance
    pub min_distance: f32,
    /// Maximum distance
    pub max_distance: f32,
    /// Minimum pitch (prevents flipping)
    pub min_pitch: f32,
    /// Maximum pitch (prevents flipping)
    pub max_pitch: f32,
    /// Whether right mouse button is pressed
    pub is_rotating: bool,
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 3.0,
            yaw: 0.0,
            pitch: 0.3,
            sensitivity: 0.005,
            zoom_sensitivity: 0.5,
            min_distance: 0.5,
            max_distance: 50.0,
            min_pitch: -std::f32::consts::FRAC_PI_2 + 0.1,
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.1,
            is_rotating: false,
        }
    }
}

impl OrbitCameraController {
    /// Calculate camera position from orbital parameters
    pub fn calculate_position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        
        self.target + Vec3::new(x, y, z)
    }
}

/// System to handle orbit camera input
pub fn orbit_camera_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCameraController, &mut Transform)>,
    mut contexts: bevy_egui::EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    
    // Check ALL conditions that indicate UI is being interacted with
    let pointer_over_ui = ctx.is_pointer_over_area();
    let using_pointer = ctx.is_using_pointer();
    let wants_pointer = ctx.wants_pointer_input();
    
    // Block camera input if any UI interaction is happening
    if pointer_over_ui || using_pointer || wants_pointer {
        mouse_motion.clear();
        mouse_wheel.clear();
        return;
    }

    for (mut controller, mut transform) in query.iter_mut() {
        // Track right mouse button state
        controller.is_rotating = mouse_button.pressed(MouseButton::Right);

        // Handle rotation with right mouse button
        if controller.is_rotating {
            for event in mouse_motion.read() {
                controller.yaw += event.delta.x * controller.sensitivity;
                controller.pitch = (controller.pitch - event.delta.y * controller.sensitivity)
                    .clamp(controller.min_pitch, controller.max_pitch);
            }
        } else {
            mouse_motion.clear();
        }

        // Handle zoom with scroll wheel
        for event in mouse_wheel.read() {
            controller.distance = (controller.distance - event.y * controller.zoom_sensitivity)
                .clamp(controller.min_distance, controller.max_distance);
        }

        // Update camera transform
        let position = controller.calculate_position();
        *transform = Transform::from_translation(position)
            .looking_at(controller.target, Vec3::Y);
    }
}
