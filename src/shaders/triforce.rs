#![allow(dead_code)]
//! Shader Triforce: dorado metálico con destello animado que recorre la superficie.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::smoothstep;
use crate::shaders::{diffuse_factor, surface_mu, surface_view};

const GOLD_DEEP: Vector3 = Vector3 { x: 0.48, y: 0.31, z: 0.04 };
const GOLD_MID: Vector3 = Vector3 { x: 0.94, y: 0.74, z: 0.14 };
const GOLD_SPEC: Vector3 = Vector3 { x: 1.00, y: 0.96, z: 0.72 };

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let n = fragment.normal.normalize();
    let diff = diffuse_factor(n, uniforms);
    let view = surface_view(fragment, uniforms);
    let half_vec = (uniforms.light_dir + view).normalize();
    let spec = n.dot(half_vec).max(0.0).powi(28);
    let mu = surface_mu(fragment, uniforms);
    let fresnel = (1.0 - mu).powi(3);

    // Destello animado que barre en diagonal sobre la superficie.
    let speed = 1.6 * uniforms.shader_param;
    let phase = (fragment.object_pos.x * 0.9 + fragment.object_pos.y * 0.7 - uniforms.time * speed)
        .rem_euclid(3.2);
    let sweep = (1.0 - smoothstep(0.0, 0.35, (phase - 1.2).abs())) * 0.85;

    let base = GOLD_DEEP + (GOLD_MID - GOLD_DEEP) * (0.28 + 0.72 * diff);
    let mut out = base + GOLD_SPEC * (spec * 0.75 + fresnel * 0.25 + sweep * (0.45 + 0.55 * diff));
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
