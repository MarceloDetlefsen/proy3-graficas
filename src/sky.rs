#![allow(dead_code)]
//! Fondo estelar procedural sin geometría para píxeles vacíos del framebuffer.
//!
//! Reconstruye la dirección de la vista por píxel mediante la inversa de la
//! matriz `P * V` y evalúa un campo de estrellas procedural basado en hash de
//! celdas 3D sobre la esfera unitaria, con temperatura de color y parpadeo suave.

use raylib::prelude::Vector3;

use crate::math::{Mat4, Vec4};

pub const SPACE_BG: Vector3 = Vector3 {
    x: 0.015,
    y: 0.020,
    z: 0.045,
};

#[inline]
fn hash_cell(ix: i32, iy: i32, iz: i32, seed: u32) -> u32 {
    let mut h = (ix as u32).wrapping_mul(374_761_393)
        ^ (iy as u32).wrapping_mul(668_265_263)
        ^ (iz as u32).wrapping_mul(2_147_483_647)
        ^ seed.wrapping_mul(1_597_334_677);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

#[inline]
fn hash_to_f32(h: u32) -> f32 {
    (h & 0x00FF_FFFF) as f32 / 16_777_215.0
}

#[derive(Clone, Copy, Debug)]
pub struct StarSample {
    pub intensity: f32,
    pub hash: u32,
}

/// Evalúa una capa de estrellas procedurales en `dir` usando hash de celdas 3D.
pub fn star_layer_sample(
    dir: Vector3,
    seed: u32,
    grid_scale: f32,
    density: f32,
    max_radius: f32,
) -> StarSample {
    let p = dir.normalize() * grid_scale;
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let iz = p.z.floor() as i32;
    let fx = p.x - ix as f32;
    let fy = p.y - iy as f32;
    let fz = p.z - iz as f32;

    let h = hash_cell(ix, iy, iz, seed);
    let prob = hash_to_f32(h);
    if prob > density {
        return StarSample {
            intensity: 0.0,
            hash: h,
        };
    }

    let h1 = h.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let h2 = h1.wrapping_mul(277_803_737).wrapping_add(1_442_695_041);
    let h3 = h2.wrapping_mul(1_103_515_245).wrapping_add(12_345);

    let cx = 0.25 + 0.50 * hash_to_f32(h1);
    let cy = 0.25 + 0.50 * hash_to_f32(h2);
    let cz = 0.25 + 0.50 * hash_to_f32(h3);

    let dx = fx - cx;
    let dy = fy - cy;
    let dz = fz - cz;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist >= max_radius {
        return StarSample {
            intensity: 0.0,
            hash: h,
        };
    }

    let core = (1.0 - dist / max_radius).powi(3);
    let brightness = 0.30 + 0.70 * hash_to_f32(h ^ (h >> 7));
    StarSample {
        intensity: core * brightness,
        hash: h,
    }
}

/// Evalúa el brillo combinado de estrellas (capa lejana densa + capa brillante).
pub fn sky_star_intensity(dir: Vector3, seed: u32) -> f32 {
    let dim = star_layer_sample(dir, seed, 110.0, 0.045, 0.22);
    let bright = star_layer_sample(dir, seed.wrapping_add(17), 55.0, 0.025, 0.20);
    (dim.intensity * 0.65 + bright.intensity * 1.15).clamp(0.0, 1.0)
}

/// Reconstruye la dirección unitaria del rayo de vista para el centro del píxel `(x, y)`.
pub fn view_direction_for_pixel(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    inv_vp: &Mat4,
    camera_pos: Vector3,
) -> Vector3 {
    let ndc_x = ((x + 0.5) / width.max(1.0)) * 2.0 - 1.0;
    let ndc_y = 1.0 - ((y + 0.5) / height.max(1.0)) * 2.0;
    let world_h = *inv_vp * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    let inv_w = if world_h.w.abs() > 1e-8 {
        1.0 / world_h.w
    } else {
        1.0
    };
    let world_pt = world_h.xyz() * inv_w;
    (world_pt - camera_pos).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;

    #[test]
    fn center_pixel_unprojects_to_camera_forward_direction() {
        let cam = Camera::new(
            Vector3::new(3.0, 2.0, 12.0),
            Vector3::new(-1.0, 0.5, 0.0),
            45.0_f32.to_radians(),
            0.1,
            300.0,
        );
        let vp = cam.projection_matrix(4.0 / 3.0) * cam.view_matrix();
        let inv_vp = vp.inverse().expect("VP debe ser invertible");
        let dir = view_direction_for_pixel(399.5, 299.5, 800.0, 600.0, &inv_vp, cam.position);
        let expected = (cam.target - cam.position).normalize();
        assert!((dir - expected).length() < 1e-4);
    }
}
