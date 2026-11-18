#![allow(dead_code)]
//! Generación procedural de icosferas por subdivisión de un icosaedro base.
//!
//! Construye un icosaedro unitario con 12 vértices y 20 triángulos orientados
//! hacia afuera (CCW), y lo subdivide recursivamente usando una caché de puntos
//! medios por arista proyectados a la esfera unitaria para niveles 0..=5.

use raylib::prelude::Vector3;
use std::collections::HashMap;

pub const MAX_ICOSPHERE_LEVEL: usize = 5;

#[derive(Clone, Debug)]
pub struct IcoMesh {
    pub positions: Vec<Vector3>,
    pub indices: Vec<usize>,
}

/// Genera el icosaedro base (nivel 0): 12 vértices unitarios y 20 triángulos CCW.
pub fn base_icosahedron() -> IcoMesh {
    let t = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let raw = [
        Vector3::new(-1.0, t, 0.0),
        Vector3::new(1.0, t, 0.0),
        Vector3::new(-1.0, -t, 0.0),
        Vector3::new(1.0, -t, 0.0),
        Vector3::new(0.0, -1.0, t),
        Vector3::new(0.0, 1.0, t),
        Vector3::new(0.0, -1.0, -t),
        Vector3::new(0.0, 1.0, -t),
        Vector3::new(t, 0.0, -1.0),
        Vector3::new(t, 0.0, 1.0),
        Vector3::new(-t, 0.0, -1.0),
        Vector3::new(-t, 0.0, 1.0),
    ];
    let positions: Vec<Vector3> = raw.into_iter().map(|v| v.normalize()).collect();

    #[rustfmt::skip]
    let indices = vec![
        0, 11, 5,  0, 5, 1,   0, 1, 7,   0, 7, 10,  0, 10, 11,
        1, 5, 9,   5, 11, 4,  11, 10, 2, 10, 7, 6,  7, 1, 8,
        3, 9, 4,   3, 4, 2,   3, 2, 6,   3, 6, 8,   3, 8, 9,
        4, 9, 5,   2, 4, 11,  6, 2, 10,  8, 6, 7,   9, 8, 1,
    ];

    IcoMesh { positions, indices }
}

fn midpoint_index(
    a: usize,
    b: usize,
    positions: &mut Vec<Vector3>,
    cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let mid = ((positions[a] + positions[b]) * 0.5).normalize();
    let idx = positions.len();
    positions.push(mid);
    cache.insert(key, idx);
    idx
}

/// Subdivide cada triángulo en 4 usando caché de puntos medios por arista y
/// proyectando los nuevos vértices sobre la esfera unitaria.
pub fn subdivide_icosphere(mesh: &IcoMesh) -> IcoMesh {
    let mut positions = mesh.positions.clone();
    let mut indices = Vec::with_capacity(mesh.indices.len() * 4);
    let mut cache = HashMap::new();

    for tri in mesh.indices.as_chunks::<3>().0 {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
        let m01 = midpoint_index(i0, i1, &mut positions, &mut cache);
        let m12 = midpoint_index(i1, i2, &mut positions, &mut cache);
        let m20 = midpoint_index(i2, i0, &mut positions, &mut cache);

        indices.extend_from_slice(&[
            i0, m01, m20,
            i1, m12, m01,
            i2, m20, m12,
            m01, m12, m20,
        ]);
    }

    IcoMesh { positions, indices }
}

/// Construye la icosfera en el nivel de subdivisión `level` (limitado a `0..=5`).
pub fn icosphere_level(level: usize) -> IcoMesh {
    let clamped = level.min(MAX_ICOSPHERE_LEVEL);
    let mut mesh = base_icosahedron();
    for _ in 0..clamped {
        mesh = subdivide_icosphere(&mesh);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosphere_levels_0_to_5_match_counts_unit_radius_and_outward_winding() {
        let mut mesh = base_icosahedron();
        for level in 0..=MAX_ICOSPHERE_LEVEL {
            let pow4 = 4_usize.pow(level as u32);
            let expected_verts = 10 * pow4 + 2;
            let expected_tris = 20 * pow4;
            assert_eq!(mesh.positions.len(), expected_verts, "nivel {level}");
            assert_eq!(mesh.indices.len(), expected_tris * 3, "nivel {level}");

            for &p in &mesh.positions {
                assert!(
                    (p.length() - 1.0).abs() < 1e-5,
                    "vértice fuera de esfera unitaria en nivel {level}: {}",
                    p.length()
                );
            }

            for tri in mesh.indices.as_chunks::<3>().0 {
                let p0 = mesh.positions[tri[0]];
                let p1 = mesh.positions[tri[1]];
                let p2 = mesh.positions[tri[2]];
                let n = (p1 - p0).cross(p2 - p0);
                let centroid = p0 + p1 + p2;
                assert!(n.dot(centroid) > 0.0, "cara invertida en nivel {level}");
            }

            if level < MAX_ICOSPHERE_LEVEL {
                mesh = subdivide_icosphere(&mesh);
            }
        }
    }
}
