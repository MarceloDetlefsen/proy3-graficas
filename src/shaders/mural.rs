#![allow(dead_code)]
//! Shader Mural (pared de pintura descascarada): estratigrafía de 3 capas por umbral fbm,
//! normales derivadas de altura, grietas tipo ridge, marcas de tinta y escamas animadas.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::icosphere::spherical_tangent_basis;
use crate::noise::{self, Noise};
use crate::shaders::diffuse_factor;

const BLUE_GRAY: Vector3 = Vector3 { x: 0.486, y: 0.529, z: 0.580 }; // #7C8794 (~55%)
const WALL_CREAM: Vector3 = Vector3 { x: 0.914, y: 0.875, z: 0.784 }; // #E9DFC8 (~30%)
const WORN_RED: Vector3 = Vector3 { x: 0.722, y: 0.255, z: 0.184 }; // #B8412F (~12%)
const INK_BLACK: Vector3 = Vector3 { x: 0.071, y: 0.071, z: 0.071 }; // #121212 (~3%)

#[inline]
fn layer_field(noise: &Noise, p: Vector3, frequency: f32) -> f32 {
    let raw = noise.fbm(p * (frequency * 0.42), 3, 2.0, 0.5);
    ((raw - 0.5) * 2.1 + 0.5).clamp(0.0, 1.0)
}

#[inline]
fn paint_height_from_field(lf: f32) -> f32 {
    let mid_step = noise::smoothstep(0.22, 0.27, lf);
    let top_step = noise::smoothstep(0.43, 0.49, lf);
    // Los bordes de parche se levantan ligeramente como pintura descascarada.
    let curl_mid = (1.0 - ((lf - 0.25).abs() / 0.035).min(1.0)) * 0.12;
    let curl_top = (1.0 - ((lf - 0.46).abs() / 0.035).min(1.0)) * 0.14;
    (0.14 + 0.38 * mid_step + 0.48 * top_step + curl_mid + curl_top).clamp(0.0, 1.0)
}

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let lf = layer_field(noise, p, frequency);
    paint_height_from_field(lf)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let lf = layer_field(noise, p, frequency);
    let h0 = paint_height_from_field(lf);

    let (t, b) = spherical_tangent_basis(p, p);
    let eps = 0.012;
    let ht = paint_height_from_field(layer_field(noise, (p + t * eps).normalize(), frequency));
    let hb = paint_height_from_field(layer_field(noise, (p + b * eps).normalize(), frequency));
    let slope_t = ((ht - h0) / eps).clamp(-4.0, 4.0);
    let slope_b = ((hb - h0) / eps).clamp(-4.0, 4.0);

    let spray = noise::smoothstep(0.76, 0.88, noise.fbm(p * 3.2 + Vector3::new(4.1, -2.0, 1.3), 2, 2.0, 0.5));
    [lf, slope_t, slope_b, spray]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let freq = uniforms.noise_frequency.max(1.0);
    let [_baked_lf, _baked_st, _baked_sb, baked_spray] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.mural.sample_dir(p)
    } else {
        [0.5, 0.0, 0.0, 0.0]
    };

    // Evaluar el campo de capas en 3D para evitar bordes dentados al acercarse.
    let lf = layer_field(&uniforms.noise, p, freq);

    // Ciclo largo (~28 s) de escamas que se despegan, se curvan y reaparecen en bordes de parche.
    let t = uniforms.time;
    let peel_zone = uniforms.noise.noise3(p * 4.5 + Vector3::new(-2.3, 1.1, 3.4));
    let cycle = ((t * 0.22 + peel_zone * std::f32::consts::TAU).sin() * 0.5 + 0.5).powf(2.0);
    let peel_shift = if peel_zone > 0.52 { (cycle - 0.5) * 0.065 } else { 0.0 };
    let lf_anim = (lf + peel_shift).clamp(0.0, 1.0);

    // Ancho de suavizado (~1 a 1.5 px) para bordes limpios a cualquier distancia.
    let vh = uniforms.viewport_height.max(1) as f32;
    let edge_w = (5.5 / vh).clamp(0.010, 0.026);
    let mid_mask = noise::smoothstep(0.24 - edge_w, 0.24 + edge_w, lf_anim);
    let top_mask = noise::smoothstep(0.455 - edge_w, 0.455 + edge_w, lf_anim);

    // Moteado de mugre en 3D suave y redondo (sin píxeles cuadrados ni ruido de TV), intensidad ~50%.
    let grime_n = uniforms.noise.fbm(p * 7.5 + Vector3::new(1.7, -0.9, 2.3), 2, 2.0, 0.5);
    let grime_spots = noise::smoothstep(0.56, 0.72, grime_n);
    let deep_mottle = uniforms.noise.noise3(p * 6.5);
    let deep_col = noise::mix_color(WORN_RED * 0.88, WORN_RED, deep_mottle);
    let mid_col = noise::mix_color(WALL_CREAM * 0.94, WALL_CREAM, deep_mottle * 0.6);
    let top_col = noise::mix_color(BLUE_GRAY * 0.94, BLUE_GRAY, 1.0 - grime_spots * 0.22);

    let mut base = noise::mix_color(deep_col, mid_col, mid_mask);
    base = noise::mix_color(base, top_col, top_mask);

    // Grietas ridge finas y rayones de tinta evaluados en 3D (sin pixelado).
    let crack_raw = (uniforms.noise.noise3(p * (freq * 1.55) + Vector3::new(2.1, -1.7, 0.8)) - 0.5).abs() * 2.0;
    let ridge_crack = 1.0 - noise::smoothstep(0.0, 0.085, crack_raw);
    let streak_n = (uniforms.noise.noise3(Vector3::new(p.x * 16.0, p.y * 4.5, p.z * 16.0)) - 0.5).abs() * 2.0;
    let scratch = (1.0 - noise::smoothstep(0.0, 0.032, streak_n))
        * noise::smoothstep(0.62, 0.78, uniforms.noise.noise3(p * 3.8 + Vector3::new(-1.4, 2.3, 0.5)));
    let ink_and_crack = (ridge_crack * 0.50 + scratch * 0.85 + baked_spray * 0.55).clamp(0.0, 1.0);

    // Mugre sutil en los bordes de parche + marcas de tinta.
    let edge_grime = (1.0 - ((lf_anim - 0.455).abs() / 0.034).min(1.0))
        .max(1.0 - ((lf_anim - 0.24).abs() / 0.032).min(1.0));
    base = noise::mix_color(
        base,
        INK_BLACK,
        (edge_grime * 0.22 + grime_spots * 0.10 + ink_and_crack * 0.86).clamp(0.0, 1.0),
    );

    // Normales de relieve derivadas en 3D cerca de los bordes levantados (sin escalones).
    let n_base = fragment.normal.normalize();
    let near_edge = (lf_anim - 0.24).abs() < 0.075 || (lf_anim - 0.455).abs() < 0.075;
    let peel_n = if near_edge {
        let (pt, pb) = spherical_tangent_basis(p, p);
        let (wt, wb) = spherical_tangent_basis(p, n_base);
        let eps = 0.012;
        let h0 = paint_height_from_field(lf_anim);
        let ht = paint_height_from_field(
            (layer_field(&uniforms.noise, (p + pt * eps).normalize(), freq) + peel_shift).clamp(0.0, 1.0),
        );
        let hb = paint_height_from_field(
            (layer_field(&uniforms.noise, (p + pb * eps).normalize(), freq) + peel_shift).clamp(0.0, 1.0),
        );
        let slope_t = ((ht - h0) / eps).clamp(-4.0, 4.0);
        let slope_b = ((hb - h0) / eps).clamp(-4.0, 4.0);
        (n_base - wt * (slope_t * 0.42) - wb * (slope_b * 0.42)).normalize()
    } else {
        n_base
    };
    let diffuse = diffuse_factor(peel_n, uniforms);

    // Leve variación de luz tipo pared bajo farola.
    let streetlamp = 0.93 + 0.07 * (t * 0.75 + p.y * 2.2).sin();
    let mut out = base * ((0.22 + 0.78 * diffuse) * streetlamp);
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
