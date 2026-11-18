#![allow(dead_code)]

use raylib::prelude::Vector3;

use crate::math::Mat4;
use crate::ship::Ship;

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

#[derive(Clone, Copy, Debug)]
pub struct FollowCamera {
    pub distance: f32,
    pub height: f32,
    pub look_ahead: f32,
    pub current_pos: Vector3,
    pub current_target: Vector3,
    pub current_up: Vector3,
}

impl FollowCamera {
    pub fn new(ship: &Ship, distance: f32, height: f32) -> Self {
        let (pos, target, up) = Self::desired_pose_raw(ship, distance, height, 4.0);
        Self {
            distance,
            height,
            look_ahead: 4.0,
            current_pos: pos,
            current_target: target,
            current_up: up,
        }
    }

    fn desired_pose_raw(
        ship: &Ship,
        distance: f32,
        height: f32,
        look_ahead: f32,
    ) -> (Vector3, Vector3, Vector3) {
        let pos = ship.position - ship.forward * distance + ship.up * height;
        let target = ship.position + ship.forward * look_ahead;
        (pos, target, ship.up)
    }

    pub fn snap(&mut self, ship: &Ship) {
        let (pos, target, up) =
            Self::desired_pose_raw(ship, self.distance, self.height, self.look_ahead);
        self.current_pos = pos;
        self.current_target = target;
        self.current_up = up;
    }

    pub fn to_camera(&self, fov_y: f32, near: f32, far: f32) -> Camera {
        Camera {
            position: self.current_pos,
            target: self.current_target,
            up: self.current_up,
            fov_y,
            near,
            far,
        }
    }
}
