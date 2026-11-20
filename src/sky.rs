#![allow(dead_code)]
//! Fondo estelar procedural sin geometría para píxeles vacíos del framebuffer.
//!
//! Reconstruye la dirección de la vista por píxel mediante la inversa de la
//! matriz `P * V` y evalúa un campo de estrellas procedural basado en hash de
//! celdas 3D sobre la esfera unitaria, con temperatura de color y parpadeo suave.

use raylib::prelude::Vector3;

use crate::math::{Mat4, Vec4};

pub const SPACE_BG: Vector3 = Vector3 {
    x: 0.014,
    y: 0.020,
    z: 0.036,
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

#[inline]
fn value_noise_3d(p: Vector3, seed: u32) -> f32 {
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let iz = p.z.floor() as i32;
    let fx = p.x - ix as f32;
    let fy = p.y - iy as f32;
    let fz = p.z - iz as f32;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let uz = fz * fz * (3.0 - 2.0 * fz);

    let c000 = hash_to_f32(hash_cell(ix, iy, iz, seed));
    let c100 = hash_to_f32(hash_cell(ix + 1, iy, iz, seed));
    let c010 = hash_to_f32(hash_cell(ix, iy + 1, iz, seed));
    let c110 = hash_to_f32(hash_cell(ix + 1, iy + 1, iz, seed));
    let c001 = hash_to_f32(hash_cell(ix, iy, iz + 1, seed));
    let c101 = hash_to_f32(hash_cell(ix + 1, iy, iz + 1, seed));
    let c011 = hash_to_f32(hash_cell(ix, iy + 1, iz + 1, seed));
    let c111 = hash_to_f32(hash_cell(ix + 1, iy + 1, iz + 1, seed));

    let x00 = c000 + (c100 - c000) * ux;
    let x10 = c010 + (c110 - c010) * ux;
    let x01 = c001 + (c101 - c001) * ux;
    let x11 = c011 + (c111 - c011) * ux;
    let y0 = x00 + (x10 - x00) * uy;
    let y1 = x01 + (x11 - x01) * uy;
    y0 + (y1 - y0) * uz
}

#[inline]
fn fbm3(mut p: Vector3, seed: u32) -> f32 {
    let mut sum = 0.0_f32;
    let mut amp = 0.55_f32;
    for oct in 0..3 {
        sum += amp * value_noise_3d(p, seed.wrapping_add(oct * 97));
        p *= 2.07;
        amp *= 0.48;
    }
    sum
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

/// Mapea un valor `t in [0, 1]` a color estelar por temperatura (naranja -> blanco -> azulado).
pub fn star_temperature_color(t: f32) -> Vector3 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        let k = t * 2.0;
        Vector3::new(
            1.0,
            0.72 + (0.98 - 0.72) * k,
            0.48 + (0.95 - 0.48) * k,
        )
    } else {
        let k = (t - 0.5) * 2.0;
        Vector3::new(
            1.0 + (0.68 - 1.0) * k,
            0.98 + (0.84 - 0.98) * k,
            0.95 + (1.0 - 0.95) * k,
        )
    }
}

/// Parpadeo suave dependiente del tiempo y del hash de la estrella.
#[inline]
pub fn star_twinkle(hash: u32, time: f32) -> f32 {
    let speed = 1.5 + 2.5 * hash_to_f32(hash ^ (hash >> 11));
    let phase = std::f32::consts::TAU * hash_to_f32(hash.wrapping_mul(31));
    0.78 + 0.22 * (time * speed + phase).sin()
}

/// Evalúa el color final del fondo espacial en `dir`: nebulosa volumétrica cálida/fría,
/// franja galáctica, filamentos oscuros de polvo y múltiples capas de estrellas.
pub fn sample_sky(dir: Vector3, seed: u32, time: f32) -> Vector3 {
    let d = dir.normalize();

    // Plano galáctico inclinado para concentrar las nubes interestelares.
    let band_n = Vector3::new(0.28, 0.88, -0.38).normalize();
    let band_dist = d.dot(band_n).abs();
    let galactic_band = (-band_dist * band_dist * 5.2).exp();

    // Domain warp 3D rápido para nubes orgánicas de polvo y gas.
    let wx = fbm3(d * 1.9 + Vector3::new(1.7, 3.1, -2.3), seed.wrapping_add(101));
    let wy = fbm3(d * 1.9 + Vector3::new(-4.2, 0.8, 2.9), seed.wrapping_add(211));
    let wz = fbm3(d * 1.9 + Vector3::new(2.5, -3.7, 1.1), seed.wrapping_add(307));
    let warp = Vector3::new(wx - 0.5, wy - 0.5, wz - 0.5) * 1.35;

    let n_warm = fbm3(d * 2.4 + warp, seed.wrapping_add(401));
    let n_cool = fbm3(d * 2.8 - warp * 0.85, seed.wrapping_add(503));
    let n_dust = fbm3(d * 5.2 + warp * 1.4, seed.wrapping_add(607));

    let warm_mask = ((n_warm - 0.34) * 2.1).clamp(0.0, 1.0).powf(1.5) * (0.35 + 0.65 * galactic_band);
    let cool_mask = ((n_cool - 0.36) * 2.0).clamp(0.0, 1.0).powf(1.6) * (0.40 + 0.60 * galactic_band);
    let dust_lane = ((n_dust - 0.46) * 2.6).clamp(0.0, 0.78);

    let warm_col = Vector3::new(0.56, 0.23, 0.07);
    let amber_core = Vector3::new(0.78, 0.38, 0.14);
    let cool_col = Vector3::new(0.07, 0.21, 0.34);
    let cyan_glow = Vector3::new(0.14, 0.34, 0.48);

    let mut color = SPACE_BG;
    color += Vector3::new(0.025, 0.035, 0.055) * galactic_band;
    color += warm_col * (warm_mask * 0.62) + amber_core * (warm_mask * warm_mask * 0.42);
    color += cool_col * (cool_mask * 0.65) + cyan_glow * (cool_mask * cool_mask * 0.35);
    color *= 1.0 - dust_lane * 0.72;

    let star_atten = 1.0 - dust_lane * 0.55;
    for (layer_seed, grid, density, radius, gain) in [
        (seed, 135.0, 0.060, 0.24, 0.68),
        (seed.wrapping_add(17), 72.0, 0.032, 0.22, 1.10),
        (seed.wrapping_add(53), 34.0, 0.016, 0.26, 1.55),
    ] {
        let star = star_layer_sample(d, layer_seed, grid, density, radius);
        if star.intensity > 0.0 {
            let temp = hash_to_f32(star.hash ^ (star.hash >> 5));
            let tint = star_temperature_color(temp);
            let twinkle = star_twinkle(star.hash, time);
            color += tint * (star.intensity * gain * twinkle * star_atten);
        }
    }
    Vector3::new(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
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
