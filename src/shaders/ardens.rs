#![allow(dead_code)]
//! Shader Ardens (atardecer saturado): placas tectónicas Voronoi con lava pulsante.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::icosphere::spherical_tangent_basis;
use crate::noise::{self, Noise, voronoi_3d};
use crate::shaders::diffuse_factor;

const MAGENTA: Vector3 = Vector3 { x: 0.878, y: 0.157, z: 0.478 };
const ORANGE: Vector3 = Vector3 { x: 1.000, y: 0.416, z: 0.122 };
const GOLD: Vector3 = Vector3 { x: 1.000, y: 0.761, z: 0.239 };
const BASALT: Vector3 = Vector3 { x: 0.165, y: 0.059, z: 0.180 };

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let v = voronoi_3d(p * (frequency * 0.55), noise.seed().wrapping_add(11));
    let crack = 1.0 - noise::smoothstep(0.03, 0.14, v.edge_dist);
    let plate_relief = noise.fbm(p * frequency, 3, 2.0, 0.5);
    ((0.40 + 0.60 * plate_relief) * (1.0 - crack * 0.88)).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let (t, b) = spherical_tangent_basis(p, p);
    let plate_relief = noise.fbm(p * frequency, 3, 2.0, 0.5);
    let eps = 0.015;
    let r_t = noise.fbm((p + t * eps).normalize() * frequency, 3, 2.0, 0.5);
    let r_b = noise.fbm((p + b * eps).normalize() * frequency, 3, 2.0, 0.5);
    let slope_t = ((r_t - plate_relief) / eps).clamp(-3.5, 3.5);
    let slope_b = ((r_b - plate_relief) / eps).clamp(-3.5, 3.5);
    [plate_relief, slope_t, slope_b, 0.0]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [plate_relief, slope_t, slope_b, _] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.ardens.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, 6.0)
    };

    // Voronoi 3D directo por píxel para grietas suaves sin pixelado.
    let v = voronoi_3d(p * (uniforms.noise_frequency.max(1.0) * 0.55), uniforms.noise.seed().wrapping_add(11));
    let crack_core = 1.0 - noise::smoothstep(0.02, 0.11, v.edge_dist);
    let crack_glow = 1.0 - noise::smoothstep(0.05, 0.22, v.edge_dist);
    let cell_phase = (v.cell_id & 0xFF) as f32 / 255.0;

    // Perturbar la normal con el relieve de la placa para que capture luz direccional.
    let n_base = fragment.normal.normalize();
    let (wt, wb) = spherical_tangent_basis(p, n_base);
    let plate_n = (n_base - wt * (slope_t * 0.45) - wb * (slope_b * 0.45)).normalize();
    let diffuse = diffuse_factor(plate_n, uniforms);

    // Placa de basalto con relieve iluminado y ligero bisel en el borde.
    let rim_catch = noise::smoothstep(0.08, 0.20, v.edge_dist) * (1.0 - noise::smoothstep(0.20, 0.34, v.edge_dist));
    let plate_lit = BASALT * (0.28 + 0.85 * plate_relief + 0.30 * rim_catch) * (0.14 + 0.86 * diffuse);

    let pulse = 0.5 + 0.5 * (uniforms.time * 2.4 + cell_phase * std::f32::consts::TAU).sin();
    let geyser = if crack_core > 0.55 && cell_phase > 0.65 {
        (uniforms.time * 1.4 + cell_phase * 23.0).sin().max(0.0).powi(4)
    } else {
        0.0
    };

    let lava_base = noise::mix_color(MAGENTA, ORANGE, pulse);
    let lava_hot = noise::mix_color(lava_base, GOLD, (pulse * 0.45 + crack_core * 0.40 + geyser * 0.85).clamp(0.0, 1.0));
    let night_boost = 0.85 + 0.55 * (1.0 - diffuse);

    let with_glow = noise::mix_color(plate_lit, lava_base, crack_glow * 0.42);
    let mut out = noise::mix_color(with_glow, lava_hot, crack_core * 0.85)
        + lava_hot * ((crack_core * 0.85 + crack_glow * 0.25) * night_boost + geyser * 0.65);
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
