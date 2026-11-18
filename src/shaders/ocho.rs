#![allow(dead_code)]
//! Shader Ocho (mural plano / tinta): cel shading, semitono y continentes en lemniscata.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::diffuse_factor;

const WALL_CREAM: Vector3 = Vector3 { x: 0.914, y: 0.875, z: 0.784 };
const FADED_RED: Vector3 = Vector3 { x: 0.722, y: 0.255, z: 0.184 };
const INK_BLACK: Vector3 = Vector3 { x: 0.071, y: 0.071, z: 0.071 };
const BLUE_GRAY: Vector3 = Vector3 { x: 0.486, y: 0.529, z: 0.580 };

#[inline]
fn lemniscate_field(p: Vector3) -> f32 {
    let r2 = p.x * p.x + p.y * p.y;
    r2 * r2 - 0.58 * (p.x * p.x - p.y * p.y)
}

pub fn height_at(_noise: &Noise, dir: Vector3, _frequency: f32) -> f32 {
    let p = dir.normalize();
    let lem = lemniscate_field(p);
    (1.0 - noise::smoothstep(-0.02, 0.02, lem)).clamp(0.0, 1.0)
}

pub fn sample_static(_noise: &Noise, dir: Vector3) -> [f32; 4] {
    let p = dir.normalize();
    let lem = lemniscate_field(p);
    let continent = 1.0 - noise::smoothstep(-0.02, 0.02, lem);
    let contour = 1.0 - noise::smoothstep(0.0, 0.026, lem.abs());
    [continent, contour, lem, 0.0]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [continent, _contour, lem, _] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.ocho.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p)
    };

    // El contorno "respira" suavemente con el tiempo.
    let breathe = 0.026 + 0.008 * (uniforms.time * 1.6).sin();
    let contour = 1.0 - noise::smoothstep(0.0, breathe, lem.abs());

    // Cel shading (posterización de la luz en 3 bandas).
    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let toon = if diffuse > 0.65 {
        1.0
    } else if diffuse > 0.25 {
        0.65
    } else {
        0.32
    };

    let base = noise::mix_color(BLUE_GRAY, FADED_RED, continent);
    let mut color = noise::mix_color(base * toon, WALL_CREAM, if diffuse > 0.65 { 0.18 } else { 0.0 });

    // Trama de semitono / hatching animado en las sombras.
    if diffuse < 0.45 {
        let phase = uniforms.time * 18.0;
        let hatch = ((fragment.position.x + fragment.position.y + phase) * 0.65).sin();
        let dot_mask = ((hatch > 0.45) as i32) as f32;
        let shadow_strength = noise::smoothstep(0.45, 0.10, diffuse);
        color = noise::mix_color(color, INK_BLACK, dot_mask * shadow_strength * 0.72);
    }

    noise::mix_color(color, INK_BLACK, contour)
}
