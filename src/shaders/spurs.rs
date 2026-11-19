#![allow(dead_code)]
//! Shader Spurs (Cresta del Tottenham):
//! - Material 0 (gallo): sombreado chunky en 3 bandas usando el azul marino medido.
//! - Material 1 (balón): azul marino con líneas blancas de costura y parche de cordones
//!   en espacio objeto siguiendo `refs/spurs_crest.jpg`.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::smoothstep;
use crate::shaders::{diffuse_factor, surface_mu, surface_view};

const WHITE: Vector3 = Vector3 { x: 0.97, y: 0.98, z: 1.00 };

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
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
    let spec = if n.dot(half_vec).max(0.0) > 0.94 { 0.35 } else { 0.0 };
    let mu = surface_mu(fragment, uniforms);
    let rim = (1.0 - mu).powi(3) * 0.22;

    let mut albedo = fragment.color;
    if fragment.material_id >= 0.5 {
        // Balón en espacio objeto: líneas blancas de paneles y parche de cordones.
        let p = fragment.object_pos.normalize();
        // Costura ecuatorial curva y dos costuras hacia el vértice izquierdo (x ~ -0.92).
        let d_eq = (p.y + 0.14 * p.x).abs();
        let d_up = (p.y - 0.55 * (p.x + 0.85)).abs();
        let d_dn = (p.y + 0.55 * (p.x + 0.85)).abs();
        let d_ring = ((p.x + 0.86).powi(2) + p.y.powi(2)).sqrt() - 0.14;
        // Parche ovalado de cordones en el lado derecho del frente.
        let lx = (p.x - 0.36) / 0.30;
        let ly = (p.y + 0.12) / 0.15;
        let oval_d = ((lx * lx + ly * ly).sqrt() - 1.0).abs();
        let in_oval = (lx * lx + ly * ly) < 1.0;
        let lace_bar = if in_oval {
            let main_lace = (p.y + 0.12).abs();
            let cross_step = ((p.x - 0.22) * 18.0).sin().abs();
            let cross = if p.x > 0.20 && p.x < 0.52 && (p.y + 0.12).abs() < 0.07 {
                cross_step
            } else {
                1.0
            };
            main_lace.min(cross * 0.025)
        } else {
            1.0
        };

        let line_d = d_eq
            .min(d_up)
            .min(d_dn)
            .min(d_ring.abs())
            .min(oval_d * 0.18)
            .min(lace_bar);
        let white_mask = 1.0 - smoothstep(0.018, 0.036, line_d);
        albedo = albedo + (WHITE - albedo) * white_mask;
    }

    let mut out = albedo * band + WHITE * (spec + rim);
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
