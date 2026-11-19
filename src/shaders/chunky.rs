#![allow(dead_code)]
//! Shader Chunky: iluminación en 3 bandas, brillo especular marcado y borde Fresnel
//! usando el color por vértice, sin evaluaciones de ruido.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::shaders::{diffuse_factor, surface_mu, surface_view};

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    if fragment.material_id >= 1.5 {
        // Visor emisivo de Samus con paleta medida, borde inferior verde oscuro y pulso suave.
        let [cr, cg, cb] = crate::samus_bombonera_data::SAMUS_VISOR_CENTER_RGB;
        let [er, eg, eb] = crate::samus_bombonera_data::SAMUS_VISOR_EDGE_RGB;
        let center_col = Vector3::new(cr, cg, cb);
        let edge_col = Vector3::new(er, eg, eb);
        let v_grad = (fragment.object_pos.y * 0.9 + 0.55).clamp(0.0, 1.0);
        let lower_rim = ((0.18 - fragment.object_pos.y) * 2.2).clamp(0.0, 1.0);
        let rim_glow = ((1.0 - surface_mu(fragment, uniforms)).powi(2) * 0.65 + lower_rim * 0.45)
            .clamp(0.0, 1.0);
        let pulse = 0.92 + 0.08 * (uniforms.time * 2.2 * uniforms.shader_param).sin();
        let base = center_col * (1.0 - rim_glow) + edge_col * rim_glow;
        let out = base * (0.90 + 0.14 * v_grad) * pulse;
        return Vector3::new(
            out.x.clamp(0.0, 1.0),
            out.y.clamp(0.0, 1.0),
            out.z.clamp(0.0, 1.0),
        );
    }

    let n = fragment.normal.normalize();
    let ndl = diffuse_factor(n, uniforms);
    let band = if ndl > 0.68 {
        1.0
    } else if ndl > 0.25 {
        0.66
    } else {
        0.34
    };

    let view = surface_view(fragment, uniforms);
    let half_vec = (uniforms.light_dir + view).normalize();
    let spec_dot = n.dot(half_vec).max(0.0);
    let spec = if fragment.material_id >= 0.5 {
        // Barniz metálico con especular fuerte (e.g. marco de raqueta Wilson).
        if spec_dot > 0.88 {
            0.78
        } else if spec_dot > 0.72 {
            0.24
        } else {
            0.0
        }
    } else if spec_dot > 0.94 {
        0.42
    } else {
        0.0
    };

    let mu = surface_mu(fragment, uniforms);
    let rim = (1.0 - mu).powi(3) * 0.26;

    let mut out = fragment.color * band + Vector3::new(1.0, 1.0, 1.0) * (spec + rim);
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Mat4;
    use crate::noise::Noise;
    use raylib::prelude::Vector2;

    fn test_uniforms() -> Uniforms {
        Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            time: 0.0,
            light_dir: Vector3::new(0.0, 0.0, 1.0),
            camera_pos: Vector3::new(0.0, 0.0, 5.0),
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: 3,
            normal_mapping: false,
            debug_normals: false,
            viewport_width: 400,
            viewport_height: 300,
            noise: Noise::new(0),
            baked: None,
            shader_param: 1.0,
        }
    }

    #[test]
    fn chunky_uses_vertex_color_and_three_bands() {
        let u = test_uniforms();
        let mk_frag = |normal: Vector3| Fragment {
            position: Vector2::zero(),
            object_pos: Vector3::zero(),
            world_pos: Vector3::zero(),
            normal,
            color: Vector3::new(0.8, 0.2, 0.1),
            material_id: 0.0,
            depth: 0.5,
        };
        let dark = shade(&mk_frag(Vector3::new(1.0, 0.0, 0.0)), &u);
        let mid = shade(&mk_frag(Vector3::new(0.0, 0.866, 0.5)), &u);
        let bright = shade(&mk_frag(Vector3::new(0.0, 0.1, 0.995)), &u);
        assert!(dark.x < mid.x && mid.x < bright.x);
    }
}
