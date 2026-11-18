#![allow(dead_code)]
//! Shader Abismo (mundo submarino): ondas direccionales, cáusticas, maelstrom y bioluminiscencia.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::icosphere::spherical_tangent_basis;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_view};

const DEEP_BLUE: Vector3 = Vector3 { x: 0.043, y: 0.235, z: 0.365 };
const TEAL: Vector3 = Vector3 { x: 0.122, y: 0.478, z: 0.549 };
const AQUA: Vector3 = Vector3 { x: 0.498, y: 0.839, z: 0.816 };
const AMBER_LAMP: Vector3 = Vector3 { x: 0.949, y: 0.722, z: 0.294 };

pub fn height_at(_noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let w1 = (p.x * frequency + p.z * frequency * 0.7).sin();
    let w2 = (p.y * frequency * 0.8 - p.x * frequency * 0.5).cos();
    (0.5 + 0.25 * w1 + 0.25 * w2).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3) -> [f32; 4] {
    let p = dir.normalize();
    let lat_depth = 1.0 - p.y.abs();
    let fbm_depth = noise.fbm(p * 2.4, 3, 2.0, 0.5);
    let bio_noise = noise.fbm(p * 9.5 + Vector3::new(3.1, -2.2, 1.7), 2, 2.0, 0.5);
    let biolum = noise::smoothstep(0.67, 0.76, bio_noise);
    let bio_phase = noise.noise3(p * 14.0);
    [lat_depth, fbm_depth, biolum, bio_phase]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [lat_depth, fbm_depth, biolum, bio_phase] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.abismo.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p)
    };

    let t = uniforms.time;
    // Suma de ondas direccionales sobre las normales en espacio tangente.
    let (tan, bitan) = spherical_tangent_basis(p, fragment.normal);
    let w1 = (p.x * 16.0 + p.z * 11.0 + t * 2.2).cos();
    let w2 = (p.z * 19.0 - p.y * 13.0 - t * 1.8).cos();
    let w3 = (p.y * 23.0 + p.x * 9.0 + t * 2.7).cos();
    let wave_n = (fragment.normal.normalize()
        + tan * ((w1 * 0.09 + w3 * 0.06) * 0.9)
        + bitan * ((w2 * 0.09 - w3 * 0.05) * 0.9))
    .normalize();

    let diffuse = diffuse_factor(wave_n, uniforms);
    let base_water = noise::mix_color(DEEP_BLUE, TEAL, (lat_depth * 0.55 + fbm_depth * 0.45).clamp(0.0, 1.0));

    // Cáusticas procedurales por interferencia de ondas.
    let c1 = (p.x * 22.0 + p.z * 17.0 + t * 1.5).sin();
    let c2 = (p.z * 21.0 - p.y * 19.0 - t * 1.3).sin();
    let caustic = (1.0 - (c1 + c2).abs().min(1.0)).powi(3) * 0.55;

    // Remolino gigante (maelstrom) en el polo norte.
    let polar = noise::smoothstep(0.55, 0.98, p.y);
    let angle = p.z.atan2(p.x);
    let spiral = (0.5 + 0.5 * (angle * 4.0 + (1.0 - p.y) * 28.0 - t * 2.2).sin()) * polar;
    let maelstrom_core = noise::smoothstep(0.86, 0.99, p.y);

    let mut color = noise::mix_color(base_water, AQUA, (caustic + spiral * 0.45).clamp(0.0, 1.0));
    color = noise::mix_color(color, DEEP_BLUE * 0.45, maelstrom_core * 0.85);
    color *= 0.16 + 0.84 * diffuse;

    let view = surface_view(fragment, uniforms);
    let half_v = (uniforms.light_dir + view).normalize();
    let spec = wave_n.dot(half_v).max(0.0).powf(64.0) * 0.75;
    color += AQUA * spec;

    // Bioluminiscencia cálida parpadeante en el lado nocturno.
    let night = (1.0 - diffuse).powi(2);
    let twinkle = 0.55 + 0.45 * (t * 2.6 + bio_phase * std::f32::consts::TAU).sin();
    color += AMBER_LAMP * (biolum * night * twinkle * 1.1);

    Vector3::new(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
}
