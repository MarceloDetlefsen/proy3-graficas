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
    pub max_accel: f32,
    pub boost_multiplier: f32,
    pub max_speed: f32,
    pub damping: f32,
    pub brake_damping: f32,
    pub angular_velocity: Vector3,
    pub angular_accel: f32,
    pub angular_damping: f32,
    pub max_angular_speed: f32,
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
            max_accel: 12.0,
            boost_multiplier: 2.2,
            max_speed: 18.0,
            damping: 0.45,
            brake_damping: 4.5,
            angular_velocity: Vector3::zero(),
            angular_accel: 7.5,
            angular_damping: 5.5,
            max_angular_speed: 2.4,
        }
    }

    /// Aplica empuje progresivo hacia adelante (`throttle > 0`) o atrás (`throttle < 0`)
    /// limitando la aceleración y la velocidad máxima resultante.
    pub fn apply_thrust(&mut self, throttle: f32, boost: bool, dt: f32) {
        let t = throttle.clamp(-1.0, 1.0);
        if t.abs() <= 1e-6 || dt <= 0.0 {
            return;
        }
        let mult = if boost { self.boost_multiplier } else { 1.0 };
        self.velocity += self.forward * (t * self.max_accel * mult * dt);
        let speed_limit = self.max_speed * mult;
        let speed_sq = self.velocity.length_sqr();
        if speed_sq > speed_limit * speed_limit {
            self.velocity = self.velocity.normalize() * speed_limit;
        }
    }

    /// Aplica amortiguación progresiva a la velocidad lineal, reforzada al frenar.
    pub fn apply_damping(&mut self, braking: bool, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let rate = if braking { self.brake_damping } else { self.damping }.max(0.0);
        let factor = (-rate * dt).exp();
        self.velocity *= factor;
        if braking && self.velocity.length_sqr() < 1e-6 {
            self.velocity = Vector3::zero();
        }
    }

    /// Acumula velocidad angular en cabeceo, guiñada y alabeo con amortiguación y tope.
    pub fn apply_angular_input(&mut self, pitch_in: f32, yaw_in: f32, roll_in: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        self.angular_velocity.x += pitch_in.clamp(-1.0, 1.0) * self.angular_accel * dt;
        self.angular_velocity.y += yaw_in.clamp(-1.0, 1.0) * self.angular_accel * dt;
        self.angular_velocity.z += roll_in.clamp(-1.0, 1.0) * self.angular_accel * dt;

        let factor = (-self.angular_damping.max(0.0) * dt).exp();
        self.angular_velocity *= factor;

        let cap = self.max_angular_speed;
        self.angular_velocity.x = self.angular_velocity.x.clamp(-cap, cap);
        self.angular_velocity.y = self.angular_velocity.y.clamp(-cap, cap);
        self.angular_velocity.z = self.angular_velocity.z.clamp(-cap, cap);
    }

    /// Integra posición y orientación (Euler semi-implícito) y re-ortonormaliza la base.
    pub fn integrate_pose(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        self.position += self.velocity * dt;
        self.rotate_local(
            self.angular_velocity.y * dt,
            self.angular_velocity.x * dt,
            self.angular_velocity.z * dt,
        );
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
