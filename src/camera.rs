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
    pub stiffness: f32,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub current_pos: Vector3,
    pub current_target: Vector3,
    pub current_up: Vector3,
}

fn rotate_axis(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    let k = axis.normalize();
    v * c + k.cross(v) * s + k * k.dot(v) * (1.0 - c)
}

impl FollowCamera {
    pub fn new(ship: &Ship, distance: f32, height: f32) -> Self {
        let mut cam = Self {
            distance,
            height,
            look_ahead: 4.0,
            stiffness: 8.0,
            look_yaw: 0.0,
            look_pitch: 0.0,
            current_pos: Vector3::zero(),
            current_target: Vector3::zero(),
            current_up: ship.up,
        };
        cam.snap(ship);
        cam
    }

    pub fn apply_mouse_look(&mut self, dx: f32, dy: f32, sensitivity: f32) {
        self.look_yaw -= dx * sensitivity;
        self.look_pitch = (self.look_pitch - dy * sensitivity).clamp(-1.2, 1.2);
    }

    pub fn recenter(&mut self) {
        self.look_yaw = 0.0;
        self.look_pitch = 0.0;
    }

    fn desired_pose(&self, ship: &Ship) -> (Vector3, Vector3, Vector3) {
        let mut look_dir = rotate_axis(ship.forward, ship.up, self.look_yaw);
        let look_right = look_dir.cross(ship.up).normalize();
        look_dir = rotate_axis(look_dir, look_right, self.look_pitch).normalize();
        let look_up = look_right.cross(look_dir).normalize();

        let pos = ship.position - look_dir * self.distance + look_up * self.height;
        let target = ship.position + look_dir * self.look_ahead;
        (pos, target, look_up)
    }

    pub fn update(&mut self, ship: &Ship, dt: f32) {
        let (desired_pos, desired_target, desired_up) = self.desired_pose(ship);
        let alpha = 1.0 - (-self.stiffness * dt.max(0.0)).exp();
        self.current_pos += (desired_pos - self.current_pos) * alpha;
        self.current_target += (desired_target - self.current_target) * alpha;
        self.current_up = (self.current_up + (desired_up - self.current_up) * alpha).normalize();
    }

    pub fn snap(&mut self, ship: &Ship) {
        let (pos, target, up) = self.desired_pose(ship);
        self.current_pos = pos;
        self.current_target = target;
        self.current_up = up;
    }

    pub fn to_camera(self, fov_y: f32, near: f32, far: f32) -> Camera {
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
