#![allow(dead_code)]
//! Mapa equirectangular para hornear capas estáticas de shaders planetarios.
//!
//! Mapea direcciones 3D normalizadas en espacio objeto a coordenadas `(u, v)`
//! mediante `u = atan2(z, x) / (2π) + 0.5` y `v = asin(y) / π + 0.5`.
//! Realiza muestreo bilineal con envoltura horizontal en `u` y recorte en los
//! polos (`v`), permitiendo precalcular ruido costoso al cargar la escena.

use raylib::prelude::Vector3;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::noise::Noise;
use crate::shaders::{abismo, ardens, basquia, gas_giant, rocky, vestigio, Shader};

pub const DEFAULT_BAKE_WIDTH: usize = 512;
pub const DEFAULT_BAKE_HEIGHT: usize = 256;

#[derive(Clone, Debug)]
pub struct BakedMaps {
    pub rocky: EquirectMap<7>,
    pub rocky_normals: EquirectMap<3>,
    pub gas_giant: EquirectMap<5>,
    pub ardens: EquirectMap<4>,
    pub abismo: EquirectMap<4>,
    pub basquia: EquirectMap<4>,
    pub vestigio: EquirectMap<4>,
}

impl BakedMaps {
    pub fn bake(noise: &Noise, width: usize, height: usize) -> Self {
        Self::bake_with_frequency(noise, width, height, 4.0)
    }

    pub fn bake_with_frequency(
        noise: &Noise,
        width: usize,
        height: usize,
        frequency: f32,
    ) -> Self {
        let rocky = EquirectMap::from_fn(width, height, |dir| {
            let h = planet_height(noise, dir, frequency);
            rocky::sample_static_with_height(noise, dir, h)
        });
        let rocky_normals = build_normal_map_from_heights(&rocky, 3.5);
        let pw = (width / 2).max(128);
        let ph = (height / 2).max(64);
        let ardens = EquirectMap::from_fn(pw, ph, |dir| ardens::sample_static(noise, dir, 6.0));
        let abismo = EquirectMap::from_fn(pw, ph, |dir| abismo::sample_static(noise, dir));
        let basquia = EquirectMap::from_fn(pw, ph, |dir| basquia::sample_static(noise, dir, 4.0));
        let vestigio = EquirectMap::from_fn(pw, ph, |dir| vestigio::sample_static(noise, dir, 5.0));
        Self {
            rocky,
            rocky_normals,
            gas_giant: EquirectMap::from_fn(width, height, |dir| gas_giant::sample_static(noise, dir)),
            ardens,
            abismo,
            basquia,
            vestigio,
        }
    }
}

/// Altura normalizada `[0, 1]` despachada por tipo de shader de planeta.
pub fn shader_height(shader: Shader, noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    match shader {
        Shader::Ardens => ardens::height_at(noise, dir, frequency),
        Shader::Abismo => abismo::height_at(noise, dir, frequency),
        Shader::Basquia => basquia::height_at(noise, dir, frequency),
        Shader::Vestigio => vestigio::height_at(noise, dir, frequency),
        _ => planet_height(noise, dir, frequency),
    }
}

/// Genera un mapa de normales en espacio tangente `(nx, ny, nz)` a partir
/// del canal de altura (índice 3) de `height_map`.
pub fn build_normal_map_from_heights(height_map: &EquirectMap<7>, strength: f32) -> EquirectMap<3> {
    let w = height_map.width.max(1);
    let h = height_map.height.max(1);
    let mut data = Vec::with_capacity(w * h);
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        let lat = (v - 0.5) * PI;
        let cos_lat = lat.cos().abs().max(0.15);
        let y_dn = y.saturating_sub(1);
        let y_up = (y + 1).min(h - 1);
        for x in 0..w {
            let x_lt = (x + w - 1) % w;
            let x_rt = (x + 1) % w;
            let h_l = height_map.data[y * w + x_lt][3];
            let h_r = height_map.data[y * w + x_rt][3];
            let h_d = height_map.data[y_dn * w + x][3];
            let h_u = height_map.data[y_up * w + x][3];

            let dh_du = (h_r - h_l) * strength / cos_lat;
            let dh_dv = (h_u - h_d) * strength;
            // Base tangente: T (+u), B = N x T (-v), N.
            let n = Vector3::new(-dh_du, dh_dv, 1.0).normalize();
            data.push([n.x, n.y, n.z]);
        }
    }
    EquirectMap {
        width: w,
        height: h,
        data,
    }
}

/// Fuente única de verdad para la altura normalizada `[0, 1]` en una dirección
/// unitaria `dir`, compartida por deformación de malla, horneado y normales.
pub fn planet_height(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let scale = (frequency / 4.0).max(0.1);
    rocky::height_field(noise, dir.normalize() * scale)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cardinal_directions_round_trip_and_sample_accurately() {
        let map = EquirectMap::from_fn(128, 64, |dir| [dir.x, dir.y, dir.z]);
        let cardinals = [
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, -1.0),
        ];

        for dir in cardinals {
            let (u, v) = direction_to_uv(dir);
            let back = uv_to_direction(u, v);
            assert!(
                (back - dir).length() < 1e-5,
                "Round-trip falló para {dir:?}: {back:?}"
            );

            let s = map.sample_dir(dir);
            let sampled = Vector3::new(s[0], s[1], s[2]);
            assert!(
                (sampled - dir).length() < 0.08,
                "Muestreo cardinal falló en {dir:?}: {sampled:?}"
            );
        }
    }

    #[test]
    fn seam_and_poles_are_continuous_without_jumps() {
        let map = EquirectMap::from_fn(256, 128, |dir| [dir.x * 0.5 + 0.5, dir.y * 0.5 + 0.5]);

        // Costura en -X (u cruza entre ~1.0 y ~0.0)
        let a = map.sample_dir(Vector3::new(-1.0, 0.2, 0.001));
        let b = map.sample_dir(Vector3::new(-1.0, 0.2, -0.001));
        assert!((a[0] - b[0]).abs() < 0.01 && (a[1] - b[1]).abs() < 0.01);

        // Polos (+Y y -Y)
        let north_a = map.sample_dir(Vector3::new(0.01, 1.0, 0.0));
        let north_b = map.sample_dir(Vector3::new(-0.01, 1.0, 0.0));
        assert!(north_a[0].is_finite() && (north_a[1] - north_b[1]).abs() < 0.01);

        let south = map.sample_dir(Vector3::new(0.0, -1.0, 0.0));
        assert!(south[0].is_finite() && south[1].is_finite());
    }
}
