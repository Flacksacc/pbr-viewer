//! Camera system for wgpu renderer

use glam::{Mat4, Vec3};

/// Camera controller with orbit behavior
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 45.0f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        }
    }
    
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }
    
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
    
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// Orbit camera controller
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance,
            yaw: 0.0,
            pitch: 0.3,
            min_distance: 0.5,
            max_distance: 50.0,
            min_pitch: -std::f32::consts::FRAC_PI_2 + 0.1,
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.1,
        }
    }
    
    pub fn calculate_position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }
    
    #[allow(dead_code)]
    pub fn to_camera(&self) -> Camera {
        let position = self.calculate_position();
        Camera::new(position, self.target)
    }
    
    pub fn to_camera_with_aspect(&self, aspect: f32) -> Camera {
        let position = self.calculate_position();
        let mut cam = Camera::new(position, self.target);
        cam.aspect = aspect;
        cam
    }
    
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch)
            .clamp(self.min_pitch, self.max_pitch);
    }
    
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta)
            .clamp(self.min_distance, self.max_distance);
    }
}

