//! Input handling for camera and interaction

use winit::event::{ElementState, MouseButton, WindowEvent};
use glam::Vec2;

/// Input state tracking
pub struct InputState {
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
    pub left_mouse_pressed: bool,
    pub right_mouse_pressed: bool,
    pub middle_mouse_pressed: bool,
    pub scroll_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            left_mouse_pressed: false,
            right_mouse_pressed: false,
            middle_mouse_pressed: false,
            scroll_delta: 0.0,
        }
    }

    pub fn update_from_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_delta = new_pos - self.mouse_position;
                self.mouse_position = new_pos;
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match button {
                    MouseButton::Left => {
                        self.left_mouse_pressed = *state == ElementState::Pressed;
                    }
                    MouseButton::Right => {
                        self.right_mouse_pressed = *state == ElementState::Pressed;
                    }
                    MouseButton::Middle => {
                        self.middle_mouse_pressed = *state == ElementState::Pressed;
                    }
                    _ => {}
                }
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.scroll_delta = *y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.scroll_delta = pos.y as f32 * 0.01;
                    }
                }
                true
            }
            WindowEvent::KeyboardInput { .. } => {
                // Keyboard input handling can be added here if needed
                false
            }
            _ => false,
        }
    }

    pub fn reset_frame(&mut self) {
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }
}

