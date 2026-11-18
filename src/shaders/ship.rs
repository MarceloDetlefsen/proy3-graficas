//! Shader de casco metálico con líneas de panel y brillo de motor animado.
//!
//! Diseñado específicamente para la nave espacial: combina iluminación metálica
//! Blinn-Phong con líneas de panel procedurales en espacio objeto, borde de
//! Fresnel y emisión pulsante cian/blanca en la parte trasera del motor.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::shaders::{diffuse_factor, surface_view};

const HULL_DARK: [f32; 3] = [0.22, 0.25, 0.30];
const HULL_LIGHT: [f32; 3] = [0.64, 0.68, 0.74];
const SEAM_COLOR: [f32; 3] = [0.08, 0.10, 0.14];
const ENGINE_GLOW: [f32; 3] = [0.25, 0.85, 1.00];

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos;
    let gx = (p.x * 6.0).fract().abs();
    let gz = (p.z * 6.0).fract().abs();
    let seam = if gx < 0.06 || gx > 0.94 || gz < 0.06 || gz > 0.94 {
        1.0
    } else {
        0.0
    };

    let base_hull = if (p.x.abs() > 0.35) ^ (p.y > 0.05) {
        Vector3::new(HULL_LIGHT[0], HULL_LIGHT[1], HULL_LIGHT[2])
    } else {
        Vector3::new(HULL_DARK[0], HULL_DARK[1], HULL_DARK[2])
    };
    let seam_col = Vector3::new(SEAM_COLOR[0], SEAM_COLOR[1], SEAM_COLOR[2]);
    let albedo = base_hull * (1.0 - seam * 0.65) + seam_col * (seam * 0.65);

    let normal = fragment.normal.normalize();
    let diffuse = diffuse_factor(normal, uniforms);
    let view = surface_view(fragment, uniforms);
    let half_vec = (uniforms.light_dir + view).normalize();
    let spec = normal.dot(half_vec).max(0.0).powf(32.0) * 0.75;
    let rim = (1.0 - normal.dot(view).abs().clamp(0.0, 1.0)).powf(3.0) * 0.22;

    let mut color = albedo * (0.22 + 0.78 * diffuse)
        + Vector3::new(0.85, 0.90, 1.0) * spec
        + Vector3::new(0.35, 0.55, 0.80) * rim;

    // Brillo animado del motor en la parte trasera (+Z local en el modelo).
    if p.z > 0.30 && p.x.abs() < 0.35 {
        let pulse = 0.75 + 0.25 * (uniforms.time * 9.0).sin();
        let mask = ((p.z - 0.30) * 2.5).clamp(0.0, 1.0);
        color += Vector3::new(ENGINE_GLOW[0], ENGINE_GLOW[1], ENGINE_GLOW[2]) * (mask * pulse * 1.1);
    }

    color
}
