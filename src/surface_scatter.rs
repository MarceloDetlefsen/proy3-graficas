#![allow(dead_code)]
//! Distribución procedural y colocación de elementos sobre superficies esféricas.
//!
//! Genera puntos candidatos sobre la esfera unitaria mediante espiral de
//! Fibonacci con perturbación reproducible, los filtra por zonas de densidad con
//! ruido 3D según la semilla, y calcula sus transformadas alineadas a la normal
//! de la superficie deformada.

use raylib::prelude::Vector3;

use crate::icosphere::spherical_tangent_basis;
use crate::noise::Noise;
use crate::rng::SplitMix64;

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
}
