#![allow(dead_code)]

use raylib::prelude::Vector3;

use crate::math::Mat4;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vector3, target: Vector3, fov_y: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            target,
            up: Vector3::new(0.0, 1.0, 0.0),
            fov_y,
            near,
            far,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective(self.fov_y, aspect, self.near, self.far)
    }
}
