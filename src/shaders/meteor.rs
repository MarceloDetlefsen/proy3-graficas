#![allow(dead_code)]
//! Shader Meteoro (Meteoro de FF7):
//! - Material 0 (cabeza): esfera vítrea blanco hielo / azul cielo / azul verdoso
//!   con trazos blancos en espiral horneados y girando lentamente.
//! - Material 1 (cola): degradado por vértice de azul verdoso a verde oscuro con leve centelleo.
//! - Material 2 (orbe): esfera pequeña verde azulado oscuro con brillo especular.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_mu, surface_view};

pub const ICE_WHITE: Vector3 = Vector3 { x: 0.918, y: 0.957, z: 1.000 }; // #EAF4FF
pub const SKY_BLUE: Vector3 = Vector3 { x: 0.620, y: 0.773, z: 1.000 }; // #9EC5FF
pub const TEAL: Vector3 = Vector3 { x: 0.243, y: 0.561, z: 0.639 }; // #3E8FA3
pub const BLUE_GREEN: Vector3 = Vector3 { x: 0.122, y: 0.420, z: 0.345 }; // #1F6B58
pub const DARK_GREEN: Vector3 = Vector3 { x: 0.024, y: 0.149, z: 0.106 }; // #06261B

const SPIRAL_CENTER: Vector3 = Vector3 { x: -0.56, y: -0.46, z: 0.687 };

fn rotate_around_axis(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let a = axis.normalize();
    let (s, c) = angle.sin_cos();
    v * c + a.cross(v) * s + a * a.dot(v) * (1.0 - c)
}

pub fn sample_static(noise: &Noise, dir: Vector3) -> [f32; 4] {
    let p = dir.normalize();
    let center = SPIRAL_CENTER.normalize();
    let dist_to_center = (p - center).length();
    let swirl_angle = dist_to_center * 3.8;
    let warped = rotate_around_axis(p, center, swirl_angle);

    // Ruido anisótropo estirado para simular pinceladas/raspaduras curvas.
    let tangent = center.cross(warped);
    let aniso_pos = Vector3::new(
        warped.x * 3.2 + tangent.x * 5.5,
        warped.y * 3.2 + tangent.y * 5.5,
        warped.z * 3.2 + tangent.z * 5.5,
    );
    let n1 = noise.fbm(aniso_pos * 1.4, 3, 2.1, 0.55);
    let n2 = noise.fbm(warped * 5.5 + Vector3::new(2.3, -1.7, 0.9), 2, 2.0, 0.5);
    let arc_mask = noise::smoothstep(0.47, 0.62, n1);
    let fine_streak = noise::smoothstep(0.52, 0.68, n2);
    let concentration = noise::smoothstep(-0.15, 0.72, p.dot(center));
    [arc_mask, fine_streak, concentration, 0.0]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let mat = fragment.material_id;
    let n = fragment.normal.normalize();

    if mat >= 1.5 {
        // Orbe (material 2): verde azulado oscuro con brillo especular marcado.
        let diff = diffuse_factor(n, uniforms);
        let view = surface_view(fragment, uniforms);
        let half_vec = (uniforms.light_dir + view).normalize();
        let spec = n.dot(half_vec).max(0.0).powi(36);
        let base = noise::mix_color(DARK_GREEN, BLUE_GREEN, 0.25 + 0.75 * diff);
        let mut out = base + ICE_WHITE * (spec * 0.90);
        out.x = out.x.clamp(0.0, 1.0);
        out.y = out.y.clamp(0.0, 1.0);
        out.z = out.z.clamp(0.0, 1.0);
        return out;
    }

    if mat >= 0.5 {
        // Cola (material 1): color por vértice con leve centelleo animado.
        let p = fragment.object_pos;
        let shimmer =
            0.90 + 0.10 * (uniforms.time * 4.5 + p.x * 6.0 + p.y * 5.0).sin();
        let mut out = fragment.color * shimmer;
        out.x = out.x.clamp(0.0, 1.0);
        out.y = out.y.clamp(0.0, 1.0);
        out.z = out.z.clamp(0.0, 1.0);
        return out;
    }

    // Cabeza (material 0): esfera vítrea con degradado y trazos blancos en espiral.
    let p = fragment.object_pos.normalize();
    let center = SPIRAL_CENTER.normalize();
    let p_anim = rotate_around_axis(p, center, uniforms.time * 0.16);
    let [arc_mask, fine_streak, concentration, _] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.meteor.sample_dir(p_anim)
    } else {
        sample_static(&uniforms.noise, p_anim)
    };

    let diff = diffuse_factor(n, uniforms);
    let view = surface_view(fragment, uniforms);
    let half_vec = (uniforms.light_dir + view).normalize();
    let spec = n.dot(half_vec).max(0.0).powi(64);
    let mu = surface_mu(fragment, uniforms);
    let rim = (1.0 - mu).powi(3);

    // Degradado base: azul verdoso -> azul cielo -> blanco hielo en el lado iluminado.
    let mid = noise::mix_color(TEAL, SKY_BLUE, noise::smoothstep(0.08, 0.62, diff));
    let lit = noise::mix_color(mid, ICE_WHITE, noise::smoothstep(0.60, 0.96, diff));

    // Trazos blancos raspados en espiral centrados abajo a la izquierda.
    let stroke_raw = (arc_mask * 0.85 + fine_streak * 0.35) * concentration;
    let stroke = (stroke_raw * uniforms.shader_param).clamp(0.0, 1.0);
    let leading_edge = rim * noise::smoothstep(0.15, 0.85, diff);

    let mut out = noise::mix_color(lit, ICE_WHITE, (stroke * 0.88 + leading_edge * 0.75).clamp(0.0, 1.0));
    out += ICE_WHITE * (spec * 0.75 + rim * 0.22);
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
