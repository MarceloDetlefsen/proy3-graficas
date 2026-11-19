#![allow(dead_code)]
//! Shader Fundido: domain warp direccional hacia el polo sur y deformación asimétrica.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_mu};

const BURNT_ORANGE: Vector3 = Vector3 { x: 0.851, y: 0.329, z: 0.118 };
const RUST_RED: Vector3 = Vector3 { x: 0.557, y: 0.165, z: 0.071 };
const TAR_BLACK: Vector3 = Vector3 { x: 0.051, y: 0.043, z: 0.039 };
const BONE: Vector3 = Vector3 { x: 0.902, y: 0.851, z: 0.765 };

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let south_melt = noise::smoothstep(0.30, -0.75, p.y);
    // Ruido estirado verticalmente para que las gotas cuelguen hacia el sur.
    let stretched = Vector3::new(p.x * frequency * 2.4, p.y * frequency * 0.40, p.z * frequency * 2.4);
    let drip = noise.fbm(stretched, 3, 2.0, 0.5);
    (south_melt * drip.powf(1.25)).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let melt = noise::smoothstep(0.35, -0.70, p.y);
    let stretched = Vector3::new(p.x * frequency * 2.4, p.y * frequency * 0.38, p.z * frequency * 2.4);
    let drip_streak = noise.fbm(stretched, 3, 2.0, 0.5);
    let tar_pool = noise::smoothstep(-0.55, -0.92, p.y);
    let drip_phase = noise.noise3(Vector3::new(p.x * 5.0, 0.0, p.z * 5.0));
    [melt, drip_streak, tar_pool, drip_phase]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [melt, drip_streak, tar_pool, drip_phase] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.fundido.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, 3.0)
    };

    // Las gotas descienden lentamente hacia -Y y se reinician.
    let drop_flow = (p.y * 4.5 + uniforms.time * 0.55 + drip_phase * 6.0).rem_euclid(1.0);
    let drop_head = noise::smoothstep(0.75, 0.98, drop_flow) * melt * drip_streak;

    let solid_part = noise::mix_color(BONE, RUST_RED, drip_streak * 0.65);
    let melted_part = noise::mix_color(RUST_RED, BURNT_ORANGE, (drip_streak + drop_head * 0.6).clamp(0.0, 1.0));
    let mut base = noise::mix_color(solid_part, melted_part, melt);
    base = noise::mix_color(base, TAR_BLACK, tar_pool * 0.85);

    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let mu = surface_mu(fragment, uniforms);
    let heat_haze = (1.0 - mu).powi(3) * melt * (0.65 + 0.35 * (uniforms.time * 7.0 + p.y * 24.0).sin());
    let glow = BURNT_ORANGE * (melt * drip_streak * 0.45 + drop_head * 0.5 + heat_haze * 0.55);

    let mut out = base * (0.18 + 0.82 * diffuse) + glow;
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
