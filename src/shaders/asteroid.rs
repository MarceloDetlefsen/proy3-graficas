#![allow(dead_code)]
//! Shader Asteroid (Asteroide de FF7): roca oscura con grietas de bordes de celdas
//! Voronoi horneadas en la capa estática y pulso rojo incandescente en vivo.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise, voronoi_3d};
use crate::shaders::diffuse_factor;

const ROCK_DARK: Vector3 = Vector3 { x: 0.09, y: 0.08, z: 0.10 };
const ROCK_MID: Vector3 = Vector3 { x: 0.22, y: 0.19, z: 0.22 };
const CRACK_CRIMSON: Vector3 = Vector3 { x: 0.78, y: 0.06, z: 0.04 };
const CRACK_CORE: Vector3 = Vector3 { x: 1.00, y: 0.28, z: 0.08 };

pub fn sample_static(noise: &Noise, dir: Vector3) -> [f32; 4] {
    let p = dir.normalize();
    let v = voronoi_3d(p * 3.8, noise.seed().wrapping_add(77));
    let crack_core = 1.0 - noise::smoothstep(0.02, 0.11, v.edge_dist);
    let crack_glow = 1.0 - noise::smoothstep(0.05, 0.24, v.edge_dist);
    let cell_phase = (v.cell_id & 0xFF) as f32 / 255.0;
    let rock_relief = noise.fbm(p * 5.0, 3, 2.0, 0.5);
    [crack_core, crack_glow, cell_phase, rock_relief]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [crack_core, crack_glow, cell_phase, rock_relief] =
        if let Some(baked) = uniforms.baked.as_ref() {
            baked.asteroid.sample_dir(p)
        } else {
            sample_static(&uniforms.noise, p)
        };

    let diff = diffuse_factor(fragment.normal, uniforms);
    let rock = noise::mix_color(ROCK_DARK, ROCK_MID, rock_relief) * (0.16 + 0.84 * diff);

    let pulse = 0.65 + 0.35 * (uniforms.time * 3.2 + cell_phase * std::f32::consts::TAU).sin();
    let intensity = uniforms.shader_param;
    let glow = CRACK_CRIMSON * (crack_glow * 0.85 * pulse * intensity)
        + CRACK_CORE * (crack_core * 1.15 * pulse * intensity);

    let mut out = rock * (1.0 - crack_core * 0.7) + glow;
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
