//! Shader Suns: placas emisivas de la explosión solar con estela de rayos y
//! balón de basquetbol 3D con costuras giratorias y sombra inferior izquierda.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::smoothstep;
use crate::shaders::{diffuse_factor, surface_mu};

fn rotate_y(v: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
}

fn rotate_x(v: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    // Placas extruidas de la explosión solar y estela de rayos: completamente emisivas.
    if fragment.material_id < 0.5 {
        let [gr, gg, gb] = crate::logo_data::SUNS_GOLD_RGB;
        let base = Vector3::new(gr, gg, gb);
        let mu = surface_mu(fragment, uniforms);
        let halo = (1.0 - mu).powi(3) * 0.35;
        let out = base + Vector3::new(1.0, 0.80, 0.25) * halo;
        return Vector3::new(out.x.clamp(0.0, 1.0), out.y.clamp(0.0, 1.0), out.z.clamp(0.0, 1.0));
    }

    // Balón de basquetbol 3D centrado en (0, 0, 0.16).
    let local = fragment.object_pos - Vector3::new(0.0, 0.0, 0.16);
    let len = local.length();
    let n_obj = if len > 1e-5 {
        local * (1.0 / len)
    } else {
        Vector3::new(0.0, 0.0, 1.0)
    };

    let [or, og, ob] = crate::logo_data::SUNS_ORANGE_RGB;
    let [sr, sg, sb] = crate::logo_data::SUNS_SHADOW_RGB;
    let orange = Vector3::new(or, og, ob);
    let shadow_orange = Vector3::new(sr, sg, sb);

    // Sombra creciente en la parte inferior izquierda (como en refs/suns_logo.png).
    let shadow_dir = Vector3::new(-0.68, -0.70, 0.20).normalize();
    let shadow_t = smoothstep(0.28, 0.52, n_obj.dot(shadow_dir));
    let mut base = orange * (1.0 - shadow_t) + shadow_orange * shadow_t;

    // Costuras negras girando sobre el eje propio del balón.
    let spin_angle = uniforms.time * uniforms.shader_param * 0.9;
    let q = rotate_x(rotate_y(n_obj, spin_angle), 0.22);

    let seam_w = 0.050;
    let d_equator = q.y.abs();
    let d_meridian = q.x.abs();
    let d_side_arcs = (q.x.abs() - (0.32 + 0.38 * q.y * q.y)).abs();
    let d_seam = d_equator.min(d_meridian).min(d_side_arcs);
    let seam_mask = 1.0 - smoothstep(seam_w * 0.55, seam_w, d_seam);

    let diff = diffuse_factor(fragment.normal, uniforms);
    let lit = 0.62 + 0.38 * diff;
    base *= lit;

    let seam_color = Vector3::new(0.05, 0.04, 0.04);
    let out = base * (1.0 - seam_mask) + seam_color * seam_mask;
    Vector3::new(out.x.clamp(0.0, 1.0), out.y.clamp(0.0, 1.0), out.z.clamp(0.0, 1.0))
}
