#![allow(dead_code)]
//! Shader Vestigio (película vieja / VHS): dunas anisotrópicas, remolinos de polvo y grano.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_mu};

const SEPIA: Vector3 = Vector3 { x: 0.710, y: 0.545, z: 0.353 };
const OCHRE: Vector3 = Vector3 { x: 0.851, y: 0.718, z: 0.494 };
const OLIVE: Vector3 = Vector3 { x: 0.420, y: 0.478, z: 0.294 };
const DUST_ORANGE: Vector3 = Vector3 { x: 0.761, y: 0.439, z: 0.239 };
const CREAM: Vector3 = Vector3 { x: 0.922, y: 0.867, z: 0.749 };

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let aniso = Vector3::new(p.x * 0.45, p.y * 2.1, p.z * 0.45) * frequency;
    noise.ridged(aniso, 3, 2.0, 0.5).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let dunes = height_at(noise, p, frequency);
    let erosion = noise.fbm(p * 2.8 + Vector3::new(1.9, -3.2, 0.7), 3, 2.0, 0.5);
    let crater_n = noise.fbm(p * 6.0 + Vector3::new(-4.1, 2.0, 3.3), 2, 2.0, 0.5);
    let crater = 1.0 - noise::smoothstep(0.62, 0.74, crater_n);
    [dunes, erosion, crater, dunes]
}

#[inline]
fn film_grain(x: i32, y: i32, tick: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ tick.wrapping_mul(2_147_483_647);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0xFF) as f32 / 255.0 - 0.5
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [dunes, erosion, crater, _] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.vestigio.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, 5.0)
    };

    let sand = noise::mix_color(SEPIA, OCHRE, dunes);
    let patch = noise::mix_color(sand, OLIVE, noise::smoothstep(0.56, 0.72, erosion) * 0.65);
    let mut base = noise::mix_color(patch, SEPIA * 0.75, crater * 0.45);

    // Frente de tormenta advectado y remolinos de polvo (dust devils).
    let t = uniforms.time;
    let storm_wave = (p.x * 2.5 - t * 0.35 + p.y * 1.4).sin() * 0.5 + 0.5;
    let storm = noise::smoothstep(0.62, 0.92, storm_wave);
    let devil_lon = (t * 0.45).cos();
    let devil_lat = (t * 0.45).sin();
    let devil_center = Vector3::new(devil_lon, 0.15 * (t * 0.3).sin(), devil_lat).normalize();
    let devil_dist = (p - devil_center).length();
    let devil = (1.0 - (devil_dist / 0.28).min(1.0)).powi(2);

    base = noise::mix_color(base, DUST_ORANGE, (storm * 0.45 + devil * 0.8).clamp(0.0, 1.0));
    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let mut color = base * (0.20 + 0.80 * diffuse);

    // Estilización VHS: scanlines suaves, grano parpadeante, viñeta y cuantización leve.
    let scanline = 1.0 - 0.06 * (0.5 + 0.5 * (fragment.position.y * 1.4 + t * 8.0).sin());
    let tick = (t * 24.0) as u32;
    let grain = film_grain(fragment.position.x as i32, fragment.position.y as i32, tick) * 0.08;
    let vignette = 0.72 + 0.28 * surface_mu(fragment, uniforms);
    color = noise::mix_color(color * scanline * vignette, CREAM, 0.08);
    color += Vector3::new(grain, grain * 0.9, grain * 0.7);

    let levels = 18.0_f32;
    Vector3::new(
        (color.x.clamp(0.0, 1.0) * levels).round() / levels,
        (color.y.clamp(0.0, 1.0) * levels).round() / levels,
        (color.z.clamp(0.0, 1.0) * levels).round() / levels,
    )
}
