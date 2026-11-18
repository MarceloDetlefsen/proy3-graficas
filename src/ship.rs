#![allow(dead_code)]
//! Estado de la nave espacial y base ortonormal de orientación.
//!
//! Mantiene la posición, velocidad y los tres ejes ortonormales (`forward`,
//! `right`, `up`) en espacio mundo (mano derecha, frente local en `-Z`).
//! Permite rotar la nave alrededor de sus ejes locales y construir la matriz
//! de modelo a partir de su orientación, escala y traslación.

use raylib::prelude::Vector3;

use crate::math::Mat4;

#[derive(Clone, Copy, Debug)]
pub struct Ship {
    pub position: Vector3,
    pub velocity: Vector3,
    pub forward: Vector3,
    pub right: Vector3,
    pub up: Vector3,
    pub scale: f32,
    pub radius: f32,
}

fn rotate_around_axis(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    let k = axis.normalize();
    v * c + k.cross(v) * s + k * k.dot(v) * (1.0 - c)
}

impl Ship {
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            velocity: Vector3::zero(),
            forward: Vector3::new(0.0, 0.0, -1.0),
            right: Vector3::new(1.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            scale: 0.7,
            radius: 0.7,
        }
    }

    pub fn orthonormalize(&mut self) {
        self.forward = self.forward.normalize();
        self.right = self.forward.cross(self.up).normalize();
        self.up = self.right.cross(self.forward).normalize();
    }

    pub fn rotate_local(&mut self, yaw: f32, pitch: f32, roll: f32) {
        if yaw.abs() > 1e-6 {
            let up = self.up;
            self.forward = rotate_around_axis(self.forward, up, yaw);
        }
        if pitch.abs() > 1e-6 {
            let right = self.right;
            self.forward = rotate_around_axis(self.forward, right, pitch);
            self.up = rotate_around_axis(self.up, right, pitch);
        }
        if roll.abs() > 1e-6 {
            let forward = self.forward;
            self.up = rotate_around_axis(self.up, forward, roll);
        }
        self.orthonormalize();
    }

    /// Matriz de modelo: alinea `-Z` local con `forward`, `+X` con `right` y `+Y` con `up`.
    pub fn model_matrix(&self, mesh_center: Vector3, unit_scale: f32) -> Mat4 {
        let s = self.scale * unit_scale;
        let r = self.right * s;
        let u = self.up * s;
        let b = -self.forward * s;
        let rot_scale = Mat4::from_cols(
            [r.x, r.y, r.z, 0.0],
            [u.x, u.y, u.z, 0.0],
            [b.x, b.y, b.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        );
        // El modelo ship.obj apunta en sentido contrario; se rota 180° en Y local antes de orientar la nave.
        Mat4::translation(self.position)
            * rot_scale
            * Mat4::rotation_y(std::f32::consts::PI)
            * Mat4::translation(Vector3::new(-mesh_center.x, -mesh_center.y, -mesh_center.z))
    }
}
