#![allow(dead_code)]
//! Distribución procedural y colocación de elementos sobre superficies esféricas.
//!
//! Genera puntos candidatos sobre la esfera unitaria mediante espiral de
//! Fibonacci con perturbación reproducible, los filtra por zonas de densidad con
//! ruido 3D según la semilla, y calcula sus transformadas alineadas a la normal
//! de la superficie deformada.

use raylib::prelude::Vector3;

use crate::icosphere::spherical_tangent_basis;
use crate::rng::SplitMix64;

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
