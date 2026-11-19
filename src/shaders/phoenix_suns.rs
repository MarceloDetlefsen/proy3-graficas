#![allow(dead_code)]
//! Shader PhoenixSuns: esfera emisiva con el logo de los Phoenix Suns proyectado
//! azimutalmente alrededor del eje Z en ambos polos y halo dorado.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::smoothstep;
use crate::shaders::surface_mu;

const SUNS_ORANGE: Vector3 = Vector3 { x: 0.898, y: 0.376, z: 0.125 }; // #E56020
const SUNS_ORANGE_DARK: Vector3 = Vector3 { x: 0.690, y: 0.263, z: 0.078 }; // #B04314
const SUNS_GOLD: Vector3 = Vector3 { x: 0.976, y: 0.627, z: 0.106 }; // #F9A01B
const SUNS_PURPLE: Vector3 = Vector3 { x: 0.114, y: 0.067, z: 0.376 }; // #1D1160
const SEAM_BLACK: Vector3 = Vector3 { x: 0.05, y: 0.04, z: 0.04 };

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    // Proyección azimutal alrededor de Z en ambos polos (+Z y -Z).
    let u = p.x;
    let v = p.y;
    let r = (u * u + v * v).sqrt();
    let angle = v.atan2(u);

    let mu = surface_mu(fragment, uniforms);
    let halo = (1.0 - mu).powi(3) * 0.55;

    // Puntas solares alrededor del balón y rayos largos hacia abajo-izquierda (refs/suns_logo.png).
    let spike = 1.0 - ((angle * 4.5).sin().abs());
    let streak_dir = (-2.45_f32 - angle).abs();
    let tail_boost = (1.0 - smoothstep(0.0, 0.55, streak_dir)) * 0.18;
    let sun_outer = 0.64 + 0.18 * spike + tail_boost;

    let mut color = if r < 0.50 {
        // Balón de baloncesto naranja con sombra creciente abajo a la izquierda.
        let shadow_mask = smoothstep(-0.18, -0.34, u + v);
        let mut ball = SUNS_ORANGE + (SUNS_ORANGE_DARK - SUNS_ORANGE) * shadow_mask;

        // Costuras negras del balón (meridiano, ecuador y dos arcos laterales).
        let d_vert = (u - 0.16).abs();
        let d_horiz = (v - 0.04 - 0.14 * u).abs();
        let d_arc1 = (((u - 0.44).powi(2) + (v - 0.02).powi(2)).sqrt() - 0.28).abs();
        let d_arc2 = (((u + 0.06).powi(2) + (v - 0.44).powi(2)).sqrt() - 0.30).abs();
        let seam_d = d_vert.min(d_horiz).min(d_arc1).min(d_arc2);
        let seam = 1.0 - smoothstep(0.014, 0.026, seam_d);
        ball = ball + (SEAM_BLACK - ball) * seam;
        ball
    } else if r < sun_outer {
        SUNS_GOLD
    } else {
        SUNS_PURPLE
    };

    color += SUNS_GOLD * halo;
    color.x = color.x.clamp(0.0, 1.0);
    color.y = color.y.clamp(0.0, 1.0);
    color.z = color.z.clamp(0.0, 1.0);
    color
}
