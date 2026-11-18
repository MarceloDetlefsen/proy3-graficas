#![allow(dead_code)]
//! Generación procedural de icosferas por subdivisión de un icosaedro base.
//!
//! Construye un icosaedro unitario con 12 vértices y 20 triángulos orientados
//! hacia afuera (CCW), y lo subdivide recursivamente usando una caché de puntos
//! medios por arista proyectados a la esfera unitaria para niveles 0..=5.

use crate::noise::Noise;
use crate::surface_map::planet_height;
use crate::vertex::Vertex;
use raylib::prelude::Vector3;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

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

/// Calcula coordenadas UV esféricas `(u, v) in [0, 1]^2` para un punto unitario.
pub fn spherical_uv(unit_pos: Vector3) -> [f32; 2] {
    let p = unit_pos.normalize();
    let u = (0.5 + p.z.atan2(p.x) / TAU).rem_euclid(1.0);
    let v = (0.5 - p.y.clamp(-1.0, 1.0).asin() / PI).clamp(0.0, 1.0);
    [u, v]
}

/// Calcula la base tangente `(tangent, bitangent)` ortonormal a `normal` a partir
/// de la dirección esférica `dir` (tangente de longitud y respaldo en los polos).
pub fn spherical_tangent_basis(dir: Vector3, normal: Vector3) -> (Vector3, Vector3) {
    let n = normal.normalize();
    let lon_raw = Vector3::new(-dir.z, 0.0, dir.x);
    let t_raw = if lon_raw.length_sqr() > 1e-8 {
        lon_raw
    } else if n.x.abs() < 0.9 {
        Vector3::new(1.0, 0.0, 0.0)
    } else {
        Vector3::new(0.0, 0.0, 1.0)
    };
    let t_ortho = t_raw - n * n.dot(t_raw);
    let tangent = if t_ortho.length_sqr() > 1e-8 {
        t_ortho.normalize()
    } else {
        Vector3::new(0.0, 0.0, 1.0).cross(n).normalize()
    };
    let bitangent = n.cross(tangent).normalize();
    (tangent, bitangent)
}

impl IcoMesh {
    /// Recalcula normales suaves por vértice promediando las normales de caras adyacentes.
    pub fn compute_smooth_normals(&self) -> Vec<Vector3> {
        let mut accum = vec![Vector3::zero(); self.positions.len()];
        for tri in self.indices.as_chunks::<3>().0 {
            let p0 = self.positions[tri[0]];
            let p1 = self.positions[tri[1]];
            let p2 = self.positions[tri[2]];
            let face_n = (p1 - p0).cross(p2 - p0);
            accum[tri[0]] += face_n;
            accum[tri[1]] += face_n;
            accum[tri[2]] += face_n;
        }
        accum
            .into_iter()
            .zip(self.positions.iter())
            .map(|(n, &p)| {
                if n.length_sqr() > 1e-12 {
                    n.normalize()
                } else {
                    p.normalize()
                }
            })
            .collect()
    }

    /// Expande la malla indexada en un arreglo de vértices por triángulo con
    /// normales suaves recalculadas y coordenadas esféricas.
    pub fn to_vertex_array(&self) -> Vec<Vertex> {
        let normals = self.compute_smooth_normals();
        let vertices: Vec<Vertex> = self
            .positions
            .iter()
            .zip(normals.iter())
            .map(|(&pos, &normal)| {
                let dir = pos.normalize();
                let (tangent, bitangent) = spherical_tangent_basis(dir, normal);
                let mut v = Vertex::with_uv(pos, normal, spherical_uv(dir));
                v.tangent = tangent;
                v.bitangent = bitangent;
                v
            })
            .collect();
        self.indices
            .iter()
            .map(|&idx| vertices[idx].clone())
            .collect()
    }
}

/// Genera todos los niveles `0..=MAX_ICOSPHERE_LEVEL` listos para renderizar.
pub fn build_icosphere_lods() -> Vec<Vec<Vertex>> {
    let mut levels = Vec::with_capacity(MAX_ICOSPHERE_LEVEL + 1);
    let mut mesh = base_icosahedron();
    for level in 0..=MAX_ICOSPHERE_LEVEL {
        levels.push(mesh.to_vertex_array());
        if level < MAX_ICOSPHERE_LEVEL {
            mesh = subdivide_icosphere(&mesh);
        }
    }
    levels
}

/// Desplaza cada vértice de la icosfera unitaria en dirección radial según `planet_height`.
pub fn deform_icosphere(
    unit_mesh: &IcoMesh,
    noise: &Noise,
    amplitude: f32,
    frequency: f32,
) -> IcoMesh {
    if amplitude.abs() <= 1e-6 {
        return unit_mesh.clone();
    }
    let positions = unit_mesh
        .positions
        .iter()
        .map(|&p| {
            let dir = p.normalize();
            let h = planet_height(noise, dir, frequency);
            dir * (1.0 + amplitude * h)
        })
        .collect();
    IcoMesh {
        positions,
        indices: unit_mesh.indices.clone(),
    }
}

/// Caché perezoso por nivel de subdivisión para mallas deformadas.
#[derive(Clone, Debug)]
pub struct DeformedIcosphereCache {
    unit_meshes: [Option<IcoMesh>; MAX_ICOSPHERE_LEVEL + 1],
    deformed_arrays: [Option<Vec<Vertex>>; MAX_ICOSPHERE_LEVEL + 1],
    amplitude: f32,
    frequency: f32,
}

impl DeformedIcosphereCache {
    pub fn new(amplitude: f32, frequency: f32) -> Self {
        Self {
            unit_meshes: std::array::from_fn(|_| None),
            deformed_arrays: std::array::from_fn(|_| None),
            amplitude,
            frequency,
        }
    }

    pub fn invalidate(&mut self, amplitude: f32, frequency: f32) {
        self.amplitude = amplitude;
        self.frequency = frequency;
        self.deformed_arrays.fill(None);
    }

    fn ensure_unit_mesh(&mut self, level: usize) -> &IcoMesh {
        let lvl = level.min(MAX_ICOSPHERE_LEVEL);
        if self.unit_meshes[lvl].is_none() {
            self.unit_meshes[lvl] = Some(icosphere_level(lvl));
        }
        self.unit_meshes[lvl].as_ref().unwrap()
    }

    pub fn get_or_build(&mut self, level: usize, noise: &Noise) -> &[Vertex] {
        let lvl = level.min(MAX_ICOSPHERE_LEVEL);
        if self.deformed_arrays[lvl].is_none() {
            let amp = self.amplitude;
            let freq = self.frequency;
            let unit = self.ensure_unit_mesh(lvl);
            let deformed = deform_icosphere(unit, noise, amp, freq);
            self.deformed_arrays[lvl] = Some(deformed.to_vertex_array());
        }
        self.deformed_arrays[lvl].as_deref().unwrap()
    }
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

    #[test]
    fn tangent_basis_is_orthonormal_at_equator_midlatitudes_and_poles() {
        let dirs = [
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.5, 0.6, -0.624).normalize(),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        ];
        for dir in dirs {
            let (t, b) = spherical_tangent_basis(dir, dir);
            assert!((t.length() - 1.0).abs() < 1e-5);
            assert!((b.length() - 1.0).abs() < 1e-5);
            assert!(t.dot(dir).abs() < 1e-5);
            assert!(b.dot(dir).abs() < 1e-5);
            assert!(t.dot(b).abs() < 1e-5);
            assert!((t.cross(b).dot(dir) - 1.0).abs() < 1e-5);
        }

        let noise = Noise::new(0);
        let deformed = deform_icosphere(&icosphere_level(2), &noise, 0.12, 4.0);
        for v in deformed.to_vertex_array() {
            assert!((v.tangent.length() - 1.0).abs() < 1e-5);
            assert!((v.bitangent.length() - 1.0).abs() < 1e-5);
            assert!(v.tangent.dot(v.normal).abs() < 1e-5);
            assert!(v.bitangent.dot(v.normal).abs() < 1e-5);
            assert!(v.tangent.dot(v.bitangent).abs() < 1e-5);
        }
    }
}
