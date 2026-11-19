#![allow(dead_code)]
//! Shader Basquia (expresionista): bloques primarios Voronoi, trazo negro y animación stop-motion.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise, voronoi_3d};
use crate::shaders::diffuse_factor;

const YELLOW: Vector3 = Vector3 { x: 0.949, y: 0.773, z: 0.000 };
const RED: Vector3 = Vector3 { x: 0.839, y: 0.157, z: 0.157 };
const BLUE: Vector3 = Vector3 { x: 0.114, y: 0.306, z: 0.620 };
const BLACK: Vector3 = Vector3 { x: 0.039, y: 0.039, z: 0.039 };
const BONE: Vector3 = Vector3 { x: 0.937, y: 0.902, z: 0.824 };

fn warped_dir(noise: &Noise, dir: Vector3, frequency: f32) -> Vector3 {
    let p = dir.normalize();
    let w = noise.fbm(p * (frequency * 0.6), 2, 2.0, 0.5) * 2.0 - 1.0;
    Vector3::new(p.x + w * 0.18, p.y - w * 0.14, p.z + w * 0.16) * (frequency * 0.7)
}

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let wp = warped_dir(noise, dir, frequency);
    let v = voronoi_3d(wp, noise.seed().wrapping_add(33));
    ((v.cell_id % 4) as f32 / 3.0).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let wp = warped_dir(noise, p, frequency);
    let v = voronoi_3d(wp, noise.seed().wrapping_add(33));
    let color_idx = ((v.cell_id % 4) as f32 + 0.5) / 4.0;
    let stroke_w = 0.06 + 0.06 * noise.noise3(p * 6.5);
    let scribble_seed = ((v.cell_id >> 8) & 0xFF) as f32 / 255.0;
    [color_idx, v.edge_dist, stroke_w, scribble_seed]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let [color_idx, edge_dist, stroke_w, scribble_seed] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.basquia.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, 4.0)
    };

    let base_color = if color_idx < 0.25 {
        YELLOW
    } else if color_idx < 0.50 {
        RED
    } else if color_idx < 0.75 {
        BLUE
    } else {
        BONE
    };

    // Animación stop-motion (~6 FPS) para el temblor del trazo negro y tachones.
    let t_q = (uniforms.time * 6.0).floor();
    let jitter = (uniforms.noise.noise3(p * 11.0 + Vector3::new(t_q * 0.73, t_q * 0.41, 0.0)) - 0.5)
        * 0.045;
    let outline = 1.0 - noise::smoothstep(stroke_w - 0.015 + jitter, stroke_w + 0.015 + jitter, edge_dist);

    // Tachones expresionistas animados tipo corona y rayones en celdas seleccionadas (~6 FPS).
    let active_scribble = ((t_q * 0.45 + scribble_seed * 17.0).sin() > -0.20) as i32 as f32;
    let mut crown_tachon = 0.0_f32;
    if scribble_seed > 0.36 && active_scribble > 0.5 && outline < 0.85 {
        // Coordenadas locales repetidas con ligero desplazamiento stop-motion por cuadro.
        let jx = (t_q * 1.7 + scribble_seed * 31.0).sin() * 0.06;
        let jy = (t_q * 2.3 + scribble_seed * 19.0).cos() * 0.06;
        let u = ((p.x + p.z * 0.65) * 4.2 + scribble_seed * 5.0 + jx).rem_euclid(1.0) * 2.0 - 1.0;
        let v = (p.y * 4.2 + scribble_seed * 3.0 + jy).rem_euclid(1.0) * 2.0 - 1.0;

        // Motivo corona de 3 picos: barra base + zig-zag superior.
        let in_x = u.abs() < 0.58;
        let base_bar = in_x && (v + 0.32).abs() < 0.075;
        let crown_peaks = (u * std::f32::consts::PI * 2.6).cos().abs() * 0.42 - 0.12;
        let crown_top = in_x && v > -0.32 && (v - crown_peaks).abs() < 0.085;
        let left_wall = (u + 0.54).abs() < 0.075 && v > -0.34 && v < 0.25;
        let right_wall = (u - 0.54).abs() < 0.075 && v > -0.34 && v < 0.25;

        // Tachón diagonal cruzado en celdas alternas.
        let cross = ((u - v).abs() < 0.070 || (u + v).abs() < 0.070) && u.abs() < 0.48 && v.abs() < 0.48;
        let use_crown = scribble_seed > 0.58;
        if (use_crown && (base_bar || crown_top || left_wall || right_wall)) || (!use_crown && cross) {
            crown_tachon = 0.92;
        }
    }

    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let lit = base_color * (0.45 + 0.55 * diffuse);
    noise::mix_color(lit, BLACK, (outline + crown_tachon).clamp(0.0, 1.0))
}
