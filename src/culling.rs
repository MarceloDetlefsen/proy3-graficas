#![allow(dead_code)]
//! Extracción de planos del frustum y descarte por esferas envolventes.
//!
//! Extrae los 6 planos del frustum (izquierdo, derecho, inferior, superior,
//! cercano y lejano) desde la matriz `P * V` bajo la convención de vectores
//! columna y `z` NDC en `[-1, 1]`, normalizados para medir distancias reales.

use raylib::prelude::Vector3;

use crate::math::Mat4;

#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

impl Plane {
    fn from_coeffs(a: f32, b: f32, c: f32, d: f32) -> Self {
        let len = (a * a + b * b + c * c).sqrt().max(1e-12);
        Self {
            normal: Vector3::new(a / len, b / len, c / len),
            d: d / len,
        }
    }

    pub fn signed_distance(&self, point: Vector3) -> f32 {
        self.normal.dot(point) + self.d
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extrae los 6 planos del frustum desde `vp = projection * view` (`z` NDC en `[-1, 1]`).
    pub fn from_vp(vp: &Mat4) -> Self {
        let row = |r: usize| -> [f32; 4] {
            [vp.cols[0][r], vp.cols[1][r], vp.cols[2][r], vp.cols[3][r]]
        };
        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        let left = Plane::from_coeffs(r3[0] + r0[0], r3[1] + r0[1], r3[2] + r0[2], r3[3] + r0[3]);
        let right = Plane::from_coeffs(r3[0] - r0[0], r3[1] - r0[1], r3[2] - r0[2], r3[3] - r0[3]);
        let bottom = Plane::from_coeffs(r3[0] + r1[0], r3[1] + r1[1], r3[2] + r1[2], r3[3] + r1[3]);
        let top = Plane::from_coeffs(r3[0] - r1[0], r3[1] - r1[1], r3[2] - r1[2], r3[3] - r1[3]);
        let near = Plane::from_coeffs(r3[0] + r2[0], r3[1] + r2[1], r3[2] + r2[2], r3[3] + r2[3]);
        let far = Plane::from_coeffs(r3[0] - r2[0], r3[1] - r2[1], r3[2] - r2[2], r3[3] - r2[3]);

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }
}
