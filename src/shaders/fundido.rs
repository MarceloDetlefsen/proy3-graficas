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
    let south_melt = noise::smoothstep(0.25, -0.75, p.y);
    // Estirado ~1:10 en vertical para que las chorreaduras cuelguen hacia el sur.
    let stretched = Vector3::new(
        p.x * frequency * 3.5,
        p.y * frequency * 0.35,
        p.z * frequency * 3.5,
    );
    let streak = noise.fbm(stretched, 3, 2.0, 0.5);
    let finger = noise::smoothstep(0.50, 0.76, streak);
    (south_melt * finger).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let melt = noise::smoothstep(0.25, -0.65, p.y);
    let stretched = Vector3::new(
        p.x * frequency * 3.5,
        p.y * frequency * 0.35,
        p.z * frequency * 3.5,
    );
    let raw_streak = noise.fbm(stretched, 3, 2.0, 0.5);
    let finger = noise::smoothstep(0.54, 0.74, raw_streak);
    let vein = noise::smoothstep(0.68, 0.80, raw_streak);
    let drip_phase = noise.noise3(Vector3::new(p.x * 6.5, 0.0, p.z * 6.5));
    [melt, finger, vein, drip_phase]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [melt, finger, vein, drip_phase] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.fundido.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, 3.0)
    };

    // Borde de fusión irregular (sin bandas horizontales).
    let melt_edge = (melt + (finger - 0.5) * 0.45).clamp(0.0, 1.0);
    let melted_zone = noise::smoothstep(0.35, 0.65, melt_edge);

    // Gota animada que desciende a lo largo de cada dedo vertical.
    let flow_y = 0.25 - (uniforms.time * 0.22 + drip_phase).rem_euclid(1.0) * 1.15;
    let tip_dist = (p.y - flow_y).abs();
    let drip_tip = (1.0 - noise::smoothstep(0.0, 0.14, tip_dist)) * finger * melted_zone;

    // Mitad superior sólida: hueso #E6D9C3 con vetas finas naranja #D9541E (sin rosa).
    let upper_solid = noise::mix_color(BONE, BURNT_ORANGE, vein * 0.72);
    // Mitad inferior fundida: rojo óxido #8E2A12 a negro brea #0D0B0A, con chorreaduras naranjas.
    let lower_base = noise::mix_color(RUST_RED, TAR_BLACK, (1.0 - finger) * 0.65);
    let lower_melted = noise::mix_color(lower_base, BURNT_ORANGE, (finger * 0.65 + drip_tip * 0.85).clamp(0.0, 1.0));
    let mut base = noise::mix_color(upper_solid, lower_melted, melted_zone);

    // Charcos oscuros de brea en el polo sur con borde naranja brillante.
    let tar_wave = p.y + (finger - 0.5) * 0.18;
    let tar_pool = noise::smoothstep(-0.56, -0.82, tar_wave);
    let tar_rim = noise::smoothstep(-0.48, -0.58, tar_wave) * (1.0 - noise::smoothstep(-0.58, -0.68, tar_wave));
    base = noise::mix_color(base, TAR_BLACK, tar_pool * 0.94);
    base = noise::mix_color(base, BURNT_ORANGE, tar_rim * 0.90);

    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let mu = surface_mu(fragment, uniforms);
    let heat_haze = (1.0 - mu).powi(3) * melted_zone * (0.65 + 0.35 * (uniforms.time * 7.0 + p.y * 22.0).sin());
    let glow = BURNT_ORANGE * (drip_tip * 0.65 + tar_rim * 0.50 + heat_haze * 0.55);

    let mut out = base * (0.18 + 0.82 * diffuse) + glow;
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
