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
    let trench = noise.ridged(p * 1.8 + Vector3::new(-1.5, 2.1, 0.4), 2, 2.0, 0.5);
    [lat_depth, fbm_depth, trench, 0.0]
}

#[inline]
fn caustic_layer(p: Vector3, seed: u32) -> f32 {
    let v = noise::voronoi_3d(p, seed);
    let edge = 1.0 - noise::smoothstep(0.0, 0.16, v.edge_dist);
    edge * edge * edge
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [lat_depth, fbm_depth, trench, _] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.abismo.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p)
    };

    let t = uniforms.time;
    let n_base = fragment.normal.normalize();
    let (tan, bitan) = spherical_tangent_basis(p, n_base);

    // 4 ondas direccionales tipo Gerstner en 3D sobre la esfera.
    let wave_dirs: [([f32; 3], f32, f32, f32); 4] = [
        ([0.76, 0.42, 0.49], 15.0, 2.1, 0.055),
        ([-0.52, 0.68, 0.52], 21.0, -1.7, 0.042),
        ([0.28, -0.58, 0.77], 28.0, 2.6, 0.030),
        ([-0.81, -0.31, -0.50], 12.0, 1.3, 0.048),
    ];
    let mut grad_n = Vector3::zero();
    for (dir_arr, freq, speed, amp) in wave_dirs {
        let d = Vector3::new(dir_arr[0], dir_arr[1], dir_arr[2]).normalize();
        let phase = p.dot(d) * freq + t * speed;
        let slope = phase.cos() + 0.35 * (phase * 2.0).cos();
        let proj = d - p * p.dot(d);
        grad_n += proj * (slope * amp);
    }

    // Remolino gigante (maelstrom) en el polo norte con espiral logarítmica y 4 brazos de espuma.
    let polar = noise::smoothstep(0.45, 0.96, p.y);
    let r_polar = (1.0 - p.y).max(0.005);
    let angle = p.z.atan2(p.x);
    let log_r = r_polar.ln();
    let spiral_phase = angle * 4.0 - log_r * 7.5 - t * 2.8;
    let spiral_arms = (0.5 + 0.5 * spiral_phase.cos()).powf(2.2) * polar;
    let maelstrom_core = noise::smoothstep(0.90, 0.99, p.y);
    let vortex_swirl = tan * (polar * 0.16) - bitan * (polar * 0.10 * spiral_phase.sin());

    let wave_n = (n_base - grad_n + vortex_swirl).normalize();
    let diffuse = diffuse_factor(wave_n, uniforms);

    // Profundidad oceánica y fosas abisales.
    let depth_t = (lat_depth * 0.45 + fbm_depth * 0.40 + trench * 0.15).clamp(0.0, 1.0);
    let base_water = noise::mix_color(DEEP_BLUE, TEAL, depth_t);

    // Cáusticas en 3D con 2 capas de Voronoi en movimiento opuesto e intensidad moderada.
    let warp_a = Vector3::new((t * 0.25).sin() * 0.35, t * 0.18, (t * 0.22).cos() * 0.35);
    let warp_b = Vector3::new(-(t * 0.20).cos() * 0.30, -t * 0.15, (t * 0.28).sin() * 0.30);
    let c_a = caustic_layer(p * 6.5 + warp_a, 31);
    let c_b = caustic_layer(p * 8.0 + warp_b, 97);
    let caustic = (c_a * 0.55 + c_b * 0.45).clamp(0.0, 1.0) * 0.36 * (0.35 + 0.65 * diffuse);

    let mut color = noise::mix_color(base_water, AQUA, (caustic + spiral_arms * 0.68).clamp(0.0, 1.0));
    color = noise::mix_color(color, DEEP_BLUE * 0.28, maelstrom_core * 0.90);
    color *= 0.15 + 0.85 * diffuse;

    // Brillo solar especular pequeño y nítido.
    let view = surface_view(fragment, uniforms);
    let half_v = (uniforms.light_dir + view).normalize();
    let spec = wave_n.dot(half_v).max(0.0).powf(120.0) * 0.38;
    color += AQUA * spec;

    // Bioluminiscencia 3D suave y orgánica en el lado nocturno (sin píxeles cuadrados).
    let night = (1.0 - diffuse).powi(2);
    if night > 0.03 {
        let bio_n = uniforms
            .noise
            .fbm(p * 11.5 + Vector3::new(3.1, -2.2, 1.7), 2, 2.0, 0.5);
        let core_spots = noise::smoothstep(0.64, 0.74, bio_n);
        let soft_halo = noise::smoothstep(0.50, 0.64, bio_n) * 0.28;
        let phase_n = uniforms.noise.noise3(p * 7.0);
        let twinkle = 0.55 + 0.45 * (t * 1.8 + phase_n * std::f32::consts::TAU).sin();
        color += AMBER_LAMP * ((core_spots + soft_halo) * night * twinkle * 0.95);
    }

    Vector3::new(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
}
