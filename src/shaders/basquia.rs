#![allow(dead_code)]
//! Shader Basquia: lienzo negro con trazos gruesos de oilstick/crayón, coronas,
//! marcas de conteo (||||) y flechas con animación stop-motion (~6 FPS).

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::icosphere::spherical_tangent_basis;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_view};

const CANVAS_DARK: Vector3 = Vector3 { x: 0.039, y: 0.039, z: 0.039 }; // #0A0A0A
const CANVAS_WARM: Vector3 = Vector3 { x: 0.102, y: 0.086, z: 0.078 }; // #1A1614
const YELLOW: Vector3 = Vector3 { x: 0.949, y: 0.773, z: 0.000 }; // #F2C500
const RED: Vector3 = Vector3 { x: 0.839, y: 0.157, z: 0.157 }; // #D62828
const BLUE: Vector3 = Vector3 { x: 0.114, y: 0.306, z: 0.620 }; // #1D4E9E
const BONE: Vector3 = Vector3 { x: 0.937, y: 0.902, z: 0.824 }; // #EFE6D2

#[inline]
fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

#[inline]
fn hash3(ix: i32, iy: i32, iz: i32, seed: u32) -> u32 {
    let h = (ix as u32).wrapping_mul(374_761_393)
        ^ (iy as u32).wrapping_mul(668_265_263)
        ^ (iz as u32).wrapping_mul(2_147_483_647)
        ^ seed.wrapping_mul(1_274_126_177);
    hash_u32(h)
}

#[inline]
fn hash_f01(h: u32) -> f32 {
    (h & 0xFFFF) as f32 / 65535.0
}

#[inline]
fn color_from_idx(idx: u32) -> Vector3 {
    match idx % 4 {
        0 => YELLOW,
        1 => RED,
        2 => BLUE,
        _ => BONE,
    }
}

/// Evalúa trazos de oilstick en celdas de rejilla 3D (2x2x2 vecinas, sin Voronoi).
/// Devuelve `(coverage, color_id_norm, thickness)`.
fn eval_stroke_layer(
    p: Vector3,
    scale: f32,
    seed: u32,
    t_q: f32,
    animated: bool,
) -> (f32, f32, f32) {
    let q = p * scale;
    let fx = q.x.floor();
    let fy = q.y.floor();
    let fz = q.z.floor();
    let ix0 = fx as i32;
    let iy0 = fy as i32;
    let iz0 = fz as i32;
    let ox = if q.x - fx < 0.5 { -1 } else { 0 };
    let oy = if q.y - fy < 0.5 { -1 } else { 0 };
    let oz = if q.z - fz < 0.5 { -1 } else { 0 };

    let (tan, bitan) = spherical_tangent_basis(p, p);
    let frame_bucket = (t_q / 3.0).floor() as u32;

    let mut best_cov = 0.0_f32;
    let mut best_col = 0.0_f32;
    let mut best_thick = 0.0_f32;

    for dz in oz..=oz + 1 {
        for dy in oy..=oy + 1 {
            for dx in ox..=ox + 1 {
                let cx = ix0 + dx;
                let cy = iy0 + dy;
                let cz = iz0 + dz;
                let h_cell = hash3(cx, cy, cz, seed);
                // ~58% de celdas tienen 1..=2 trazos para dejar ~60% de lienzo negro visible.
                if (h_cell & 0xFF) > 148 {
                    continue;
                }
                let count = 1 + ((h_cell >> 8) & 1);
                for s in 0..count {
                    let base_hs = hash_u32(h_cell ^ s.wrapping_mul(0x9E37_79B9));
                    let can_redraw = (base_hs & 0xFF) < 46; // ~18% se redibuja cada pocos cuadros
                    let hs = if animated && can_redraw {
                        hash_u32(base_hs ^ frame_bucket.wrapping_mul(0x85EB_CA6B))
                    } else {
                        base_hs
                    };

                    let h1 = hash_u32(hs ^ 0xA24B_1F3D);
                    let h2 = hash_u32(hs ^ 0x517C_C1B7);
                    let h3 = hash_u32(hs ^ 0xC2B2_AE35);

                    let jitter_u = if animated {
                        (hash_f01(hash_u32(hs ^ (t_q as u32).wrapping_mul(7919))) - 0.5) * 0.045
                    } else {
                        0.0
                    };
                    let jitter_v = if animated {
                        (hash_f01(hash_u32(hs ^ (t_q as u32).wrapping_mul(104_729))) - 0.5) * 0.045
                    } else {
                        0.0
                    };

                    let center = Vector3::new(
                        cx as f32 + 0.20 + 0.60 * hash_f01(hs),
                        cy as f32 + 0.20 + 0.60 * hash_f01(h1),
                        cz as f32 + 0.20 + 0.60 * hash_f01(h2),
                    );

                    let rel = q - center;
                    // Proyectar al plano tangente para que el trazo abrace la esfera.
                    let rel_tan = rel - p * rel.dot(p);
                    let ang = hash_f01(h3) * std::f32::consts::TAU;
                    let dir_t = (tan * ang.cos() + bitan * ang.sin()).normalize();
                    let rel_j = rel_tan + tan * jitter_u + bitan * jitter_v;

                    let half_len = 0.22 + 0.30 * hash_f01(h1 >> 4);
                    let base_w = 0.055 + 0.055 * hash_f01(h2 >> 4);

                    let t_proj = (rel_j.dot(dir_t) / half_len).clamp(-1.0, 1.0);
                    let perp = rel_j - dir_t * (t_proj * half_len);
                    let dist = perp.length();

                    let phase = hash_f01(h3 >> 6) * std::f32::consts::TAU;
                    let local_w = base_w
                        * (1.0 - 0.32 * t_proj * t_proj)
                        * (0.82 + 0.18 * (t_proj * 7.0 + phase).sin());

                    if dist < local_w {
                        let cov = 1.0 - noise::smoothstep(local_w * 0.45, local_w, dist);
                        if cov > best_cov {
                            best_cov = cov;
                            best_col = (((hs >> 12) % 4) as f32 + 0.5) / 4.0;
                            best_thick = base_w;
                        }
                    }
                }
            }
        }
    }

    (best_cov, best_col, best_thick)
}

/// Evalúa motivos icónicos de Basquiat (coronas de 3 puntas, conteo ||||, flechas) en rejilla 3D.
fn eval_motifs(p: Vector3, scale: f32, seed: u32, t_q: f32, animated: bool) -> (f32, f32) {
    let q = p * scale;
    let ix = q.x.floor() as i32;
    let iy = q.y.floor() as i32;
    let iz = q.z.floor() as i32;
    let h = hash3(ix, iy, iz, seed);
    // ~24% de celdas contienen un motivo especial.
    if (h & 0xFF) > 62 {
        return (0.0, 0.0);
    }
    let motif_seed = hash_f01(h >> 8);
    if animated && (t_q * 0.35 + motif_seed * 13.0).sin() < -0.15 {
        return (0.0, 0.0);
    }

    let (tan, bitan) = spherical_tangent_basis(p, p);
    let center = Vector3::new(ix as f32 + 0.5, iy as f32 + 0.5, iz as f32 + 0.5);
    let rel = q - center;
    let jx = if animated {
        (t_q * 1.7 + motif_seed * 29.0).sin() * 0.04
    } else {
        0.0
    };
    let jy = if animated {
        (t_q * 2.1 + motif_seed * 17.0).cos() * 0.04
    } else {
        0.0
    };
    let u = (rel.dot(tan) * 2.2 + jx).clamp(-1.0, 1.0);
    let v = (rel.dot(bitan) * 2.2 + jy).clamp(-1.0, 1.0);

    let kind = (h >> 16) % 3;
    let hit = match kind {
        0 => {
            // Corona de 3 puntas.
            let in_x = u.abs() < 0.62;
            let base_bar = in_x && (v + 0.34).abs() < 0.085;
            let peaks = (u * std::f32::consts::PI * 2.45).cos().abs() * 0.44 - 0.12;
            let top = in_x && v > -0.34 && (v - peaks).abs() < 0.09;
            let lw = (u + 0.58).abs() < 0.085 && v > -0.36 && v < 0.28;
            let rw = (u - 0.58).abs() < 0.085 && v > -0.36 && v < 0.28;
            base_bar || top || lw || rw
        }
        1 => {
            // Marcas de conteo (|||| + diagonal).
            let bars = (v.abs() < 0.52)
                && (((u + 0.36).abs() < 0.065)
                    || ((u + 0.12).abs() < 0.065)
                    || ((u - 0.12).abs() < 0.065)
                    || ((u - 0.36).abs() < 0.065));
            let slash = (u - v * 0.9).abs() < 0.075 && u.abs() < 0.50;
            bars || slash
        }
        _ => {
            // Flecha gruesa.
            let shaft = v.abs() < 0.08 && u > -0.58 && u < 0.52;
            let head_a = (u - 0.52 + v.abs() * 0.95).abs() < 0.085 && u > 0.10 && u < 0.56;
            shaft || head_a
        }
    };

    if hit {
        let col_idx = (((h >> 20) % 4) as f32 + 0.5) / 4.0;
        (1.0, col_idx)
    } else {
        (0.0, 0.0)
    }
}

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let (c1, _, _) = eval_stroke_layer(p, frequency * 1.15, noise.seed().wrapping_add(33), 0.0, false);
    let (c2, _, _) = eval_stroke_layer(p, frequency * 2.15, noise.seed().wrapping_add(97), 0.0, false);
    let (m, _) = eval_motifs(p, frequency * 0.95, noise.seed().wrapping_add(141), 0.0, false);
    c1.max(c2).max(m * 0.9).clamp(0.0, 1.0)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let (c1, col1, _) = eval_stroke_layer(p, frequency * 1.15, noise.seed().wrapping_add(33), 0.0, false);
    let (c2, col2, _) = eval_stroke_layer(p, frequency * 2.15, noise.seed().wrapping_add(97), 0.0, false);
    let (cov, col) = if c2 > 0.35 { (c2, col2) } else { (c1, col1) };
    let grain = (noise.noise3(p * 42.0) * 0.65 + noise.noise3(p * 88.0) * 0.35).clamp(0.0, 1.0);
    let canvas_var = noise.fbm(p * 2.2, 2, 2.0, 0.5);
    [cov, col, grain, canvas_var]
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let freq = uniforms.noise_frequency.max(1.0);
    let [_static_cov, _static_col, baked_grain, canvas_var] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.basquia.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, freq)
    };

    // Animación stop-motion cuantizada a ~6 FPS.
    let t_q = (uniforms.time * 6.0).floor();
    let (c1, col1, _) = eval_stroke_layer(p, freq * 1.15, uniforms.noise.seed().wrapping_add(33), t_q, true);
    let (c2, col2, _) = eval_stroke_layer(p, freq * 2.15, uniforms.noise.seed().wrapping_add(97), t_q, true);
    let (m_cov, m_col) = eval_motifs(p, freq * 0.95, uniforms.noise.seed().wrapping_add(141), t_q, true);

    // Superposición por capas: motivos > trazos finos > trazos gruesos.
    let (raw_cov, col_norm) = if m_cov > 0.5 {
        (m_cov, m_col)
    } else if c2 > 0.25 {
        (c2, col2)
    } else {
        (c1, col1)
    };

    // Textura seca de oilstick/crayón con huecos donde asoma el lienzo negro.
    let dry_mask = noise::smoothstep(0.24, 0.58, baked_grain + raw_cov * 0.48) * raw_cov;
    let stroke_color = color_from_idx((col_norm * 4.0) as u32);
    let canvas_color = noise::mix_color(CANVAS_DARK, CANVAS_WARM, canvas_var);
    let base = noise::mix_color(canvas_color, stroke_color, dry_mask.clamp(0.0, 1.0));

    let n = fragment.normal.normalize();
    let diffuse = diffuse_factor(n, uniforms);
    // Leve brillo de cera (especular bajo y ancho) en los trazos de oilstick.
    let view = surface_view(fragment, uniforms);
    let half_v = (uniforms.light_dir + view).normalize();
    let wax_spec = n.dot(half_v).max(0.0).powf(14.0) * 0.18 * dry_mask;

    let mut out = base * (0.28 + 0.72 * diffuse) + Vector3::new(wax_spec, wax_spec, wax_spec);
    out.x = out.x.clamp(0.0, 1.0);
    out.y = out.y.clamp(0.0, 1.0);
    out.z = out.z.clamp(0.0, 1.0);
    out
}
