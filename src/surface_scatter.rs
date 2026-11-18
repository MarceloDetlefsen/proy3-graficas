#![allow(dead_code)]
//! Distribución procedural y colocación de elementos sobre superficies esféricas.
//!
//! Genera puntos candidatos sobre la esfera unitaria mediante espiral de
//! Fibonacci con perturbación reproducible, los filtra por zonas de densidad con
//! ruido 3D según la semilla, y calcula sus transformadas alineadas a la normal
//! de la superficie deformada.

use raylib::prelude::Vector3;

use crate::icosphere::spherical_tangent_basis;
use crate::math::Mat4;
use crate::noise::Noise;
use crate::rng::SplitMix64;
use crate::scene::Body;
use crate::surface_map::planet_height;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfacePropKind {
    Tree,
    Rock,
    Building,
    Monument,
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceCandidate {
    pub dir: Vector3,
    pub kind: SurfacePropKind,
    pub yaw: f32,
    pub scale: f32,
}

/// Distribuye `count` direcciones unitarias sobre la esfera usando espiral de
/// Fibonacci con perturbación tangencial reproducible controlada por `seed`.
pub fn fibonacci_sphere_points(count: usize, seed: u64, jitter: f32) -> Vec<Vector3> {
    if count == 0 {
        return Vec::new();
    }
    let mut rng = SplitMix64::new(seed);
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let mut dirs = Vec::with_capacity(count);

    for i in 0..count {
        let y = 1.0 - ((i as f32 + 0.5) / count as f32) * 2.0;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden_angle * i as f32;
        let base = Vector3::new(r * theta.cos(), y, r * theta.sin()).normalize();
        let (t, b) = spherical_tangent_basis(base, base);
        let j1 = rng.range_f32(-jitter, jitter);
        let j2 = rng.range_f32(-jitter, jitter);
        dirs.push((base + t * j1 + b * j2).normalize());
    }
    dirs
}

/// Filtra direcciones candidatas con ruido 3D propio para formar zonas de
/// densidad variable por tipo (`Tree`, `Rock`, `Building`, `Monument`).
pub fn filter_candidates_by_noise(
    dirs: &[Vector3],
    seed: u64,
    allow_trees: bool,
) -> Vec<SurfaceCandidate> {
    let noise = Noise::new(seed as u32);
    let mut rng = SplitMix64::new(seed ^ 0xA5A5_5A5A_5A5A_5A5A);
    let mut out = Vec::new();

    for &dir in dirs {
        let p = dir.normalize();
        let tree_val = if allow_trees {
            noise.fbm(p * 2.3, 3, 2.0, 0.5)
        } else {
            0.0
        };
        let rock_val = noise.ridged(p * 2.8 + Vector3::new(4.1, 1.7, -2.3), 3, 2.0, 0.5);
        let bldg_val = noise.fbm(p * 2.0 + Vector3::new(-3.4, 5.2, 1.9), 3, 2.0, 0.5);
        let mono_val = noise.fbm(p * 1.5 + Vector3::new(7.2, -4.1, 3.8), 3, 2.0, 0.5);

        let roll = rng.next_f32();
        let yaw = rng.range_f32(0.0, std::f32::consts::TAU);
        let scale_jitter = rng.range_f32(0.85, 1.25);

        let chosen = if mono_val > 0.66 && roll < (mono_val - 0.66) * 1.8 {
            Some((SurfacePropKind::Monument, 0.14 * scale_jitter))
        } else if bldg_val > 0.59 && roll < (bldg_val - 0.59) * 2.4 {
            Some((SurfacePropKind::Building, 0.12 * scale_jitter))
        } else if allow_trees && tree_val > 0.53 && roll < (tree_val - 0.53) * 3.0 {
            Some((SurfacePropKind::Tree, 0.11 * scale_jitter))
        } else if rock_val > 0.55 && roll < (rock_val - 0.55) * 2.5 {
            Some((SurfacePropKind::Rock, 0.09 * scale_jitter))
        } else {
            None
        };

        if let Some((kind, scale)) = chosen {
            out.push(SurfaceCandidate {
                dir: p,
                kind,
                yaw,
                scale,
            });
        }
    }
    out
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceInstance {
    pub kind: SurfacePropKind,
    pub local_pos: Vector3,
    pub local_normal: Vector3,
    pub local_matrix: Mat4,
    pub bounding_radius: f32,
}

/// Calcula la posición sobre la superficie deformada (con leve hundimiento) y
/// la normal geométrica local en la dirección `dir`.
pub fn surface_point_and_normal(body: &Body, dir: Vector3, noise: &Noise) -> (Vector3, Vector3) {
    let p = dir.normalize();
    let amp = body.surface.deformation_amplitude;
    let freq = body.surface.noise_frequency;
    let r = body.radius * (1.0 + amp * planet_height(noise, p, freq));
    let sink = 0.02 * body.radius;
    let local_pos = p * (r - sink);

    if amp.abs() <= 1e-6 {
        return (local_pos, p);
    }

    let (t, b) = spherical_tangent_basis(p, p);
    let eps = 0.015_f32;
    let dt = (p + t * eps).normalize();
    let db = (p + b * eps).normalize();
    let rt = body.radius * (1.0 + amp * planet_height(noise, dt, freq));
    let rb = body.radius * (1.0 + amp * planet_height(noise, db, freq));

    let pos0 = p * r;
    let post = dt * rt;
    let posb = db * rb;
    let mut normal = (post - pos0).cross(posb - pos0);
    if normal.length_sqr() <= 1e-10 {
        normal = p;
    } else {
        if normal.dot(p) < 0.0 {
            normal = -normal;
        }
        normal = normal.normalize();
    }
    (local_pos, normal)
}

/// Construye la matriz local de una instancia orientando `+Y` hacia `normal`,
/// con rotación `yaw` alrededor de `normal` y escala uniforme `world_scale`.
pub fn instance_local_matrix(
    local_pos: Vector3,
    normal: Vector3,
    yaw: f32,
    world_scale: f32,
) -> Mat4 {
    let up = normal.normalize();
    let (t0, b0) = spherical_tangent_basis(up, up);
    let (s, c) = yaw.sin_cos();
    let right = (t0 * c + b0 * s).normalize();
    let forward = up.cross(right).normalize();
    let neg_fwd = -forward;

    Mat4::from_cols(
        [right.x * world_scale, right.y * world_scale, right.z * world_scale, 0.0],
        [up.x * world_scale, up.y * world_scale, up.z * world_scale, 0.0],
        [neg_fwd.x * world_scale, neg_fwd.y * world_scale, neg_fwd.z * world_scale, 0.0],
        [local_pos.x, local_pos.y, local_pos.z, 1.0],
    )
}

/// Construye las instancias colocadas sobre un cuerpo celeste dado.
pub fn build_body_instances(
    body: &Body,
    candidates: &[SurfaceCandidate],
    noise: &Noise,
) -> Vec<SurfaceInstance> {
    candidates
        .iter()
        .map(|cand| {
            let (local_pos, local_normal) = surface_point_and_normal(body, cand.dir, noise);
            let world_scale = cand.scale * body.radius;
            let local_matrix =
                instance_local_matrix(local_pos, local_normal, cand.yaw, world_scale);
            SurfaceInstance {
                kind: cand.kind,
                local_pos,
                local_normal,
                local_matrix,
                bounding_radius: world_scale * 1.25,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_distribution_is_reproducible_and_seed_sensitive() {
        let pts_a = fibonacci_sphere_points(500, 42, 0.03);
        let pts_b = fibonacci_sphere_points(500, 42, 0.03);
        let pts_c = fibonacci_sphere_points(500, 99, 0.03);

        let cand_a = filter_candidates_by_noise(&pts_a, 42, true);
        let cand_b = filter_candidates_by_noise(&pts_b, 42, true);
        let cand_c = filter_candidates_by_noise(&pts_c, 99, true);

        assert!(!cand_a.is_empty());
        assert_eq!(cand_a.len(), cand_b.len());
        for (a, b) in cand_a.iter().zip(cand_b.iter()) {
            assert_eq!(a.kind, b.kind);
            assert!((a.dir - b.dir).length() < 1e-6);
            assert!((a.yaw - b.yaw).abs() < 1e-6);
        }

        let same_prefix = cand_a
            .iter()
            .zip(cand_c.iter())
            .filter(|(a, c)| (a.dir - c.dir).length() < 1e-4)
            .count();
        assert!(same_prefix < cand_a.len());
    }

    #[test]
    fn instance_vertical_axis_matches_surface_normal() {
        let bodies = crate::scene::solar_system();
        let zebes = &bodies[1];
        let noise = Noise::new(0);
        let dirs = fibonacci_sphere_points(200, 42, 0.03);
        let cands = filter_candidates_by_noise(&dirs, 42, true);
        let instances = build_body_instances(zebes, &cands, &noise);

        assert!(!instances.is_empty());
        for (inst, cand) in instances.iter().zip(cands.iter()) {
            let up = inst
                .local_matrix
                .transform_normal(Vector3::new(0.0, 1.0, 0.0));
            assert!((up - inst.local_normal).length() < 1e-5);
            assert!(up.dot(cand.dir) > 0.0);
        }
    }
}
