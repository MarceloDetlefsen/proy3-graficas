#![allow(dead_code)]
//! Mapa equirectangular para hornear capas estáticas de shaders planetarios.
//!
//! Mapea direcciones 3D normalizadas en espacio objeto a coordenadas `(u, v)`
//! mediante `u = atan2(z, x) / (2π) + 0.5` y `v = asin(y) / π + 0.5`.
//! Realiza muestreo bilineal con envoltura horizontal en `u` y recorte en los
//! polos (`v`), permitiendo precalcular ruido costoso al cargar la escena.

use raylib::prelude::Vector3;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

#[derive(Clone, Debug)]
pub struct EquirectMap<const N: usize> {
    pub width: usize,
    pub height: usize,
    data: Vec<[f32; N]>,
}

/// Convierte una dirección 3D normalizada en coordenadas equirectangulares `(u, v)` en `[0, 1]`.
pub fn direction_to_uv(dir: Vector3) -> (f32, f32) {
    let p = dir.normalize();
    let u = p.z.atan2(p.x) / TAU + 0.5;
    let v = p.y.clamp(-1.0, 1.0).asin() / PI + 0.5;
    (u.rem_euclid(1.0), v.clamp(0.0, 1.0))
}

/// Convierte coordenadas `(u, v)` en `[0, 1]` a una dirección 3D unitaria.
pub fn uv_to_direction(u: f32, v: f32) -> Vector3 {
    let theta = (u - 0.5) * TAU;
    let phi = ((v - 0.5) * PI).clamp(-FRAC_PI_2, FRAC_PI_2);
    let cos_phi = phi.cos();
    Vector3::new(cos_phi * theta.cos(), phi.sin(), cos_phi * theta.sin())
}

impl<const N: usize> EquirectMap<N> {
    pub fn from_fn<F>(width: usize, height: usize, mut generator: F) -> Self
    where
        F: FnMut(Vector3) -> [f32; N],
    {
        let width = width.max(1);
        let height = height.max(1);
        let mut data = Vec::with_capacity(width * height);
        for y in 0..height {
            let v = (y as f32 + 0.5) / height as f32;
            for x in 0..width {
                let u = (x as f32 + 0.5) / width as f32;
                let dir = uv_to_direction(u, v);
                data.push(generator(dir));
            }
        }
        Self {
            width,
            height,
            data,
        }
    }

    pub fn sample_uv(&self, u: f32, v: f32) -> [f32; N] {
        let fx = u.rem_euclid(1.0) * self.width as f32 - 0.5;
        let fy = (v.clamp(0.0, 1.0) * self.height as f32 - 0.5)
            .clamp(0.0, (self.height - 1) as f32);

        let x0 = (fx.floor() as isize).rem_euclid(self.width as isize) as usize;
        let x1 = (x0 + 1) % self.width;
        let tx = fx - fx.floor();

        let y0 = (fy.floor() as usize).min(self.height - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let ty = fy - fy.floor();

        let c00 = self.data[y0 * self.width + x0];
        let c10 = self.data[y0 * self.width + x1];
        let c01 = self.data[y1 * self.width + x0];
        let c11 = self.data[y1 * self.width + x1];

        let mut out = [0.0; N];
        for i in 0..N {
            let top = c00[i] + (c10[i] - c00[i]) * tx;
            let bot = c01[i] + (c11[i] - c01[i]) * tx;
            out[i] = top + (bot - top) * ty;
        }
        out
    }

    pub fn sample_dir(&self, dir: Vector3) -> [f32; N] {
        let (u, v) = direction_to_uv(dir);
        self.sample_uv(u, v)
    }
}
