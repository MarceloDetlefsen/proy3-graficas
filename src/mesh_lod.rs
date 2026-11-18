#![allow(dead_code)]
//! Decimación de mallas por agrupación de vértices en rejilla 3D (respaldo LOD).
//!
//! Divide la caja envolvente de la malla en celdas cúbicas uniformes, fusiona
//! los vértices que caen en la misma celda promediando posición, normal y UV,
//! y elimina los triángulos degenerados resultantes. Sirve como respaldo en
//! tiempo de carga cuando los archivos `ship_lod*.obj` de Blender no existen.

use raylib::prelude::Vector3;
use std::collections::HashMap;

use crate::vertex::Vertex;

pub fn mesh_bounds(vertices: &[Vertex]) -> (Vector3, Vector3) {
    if vertices.is_empty() {
        return (Vector3::zero(), Vector3::zero());
    }
    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for v in vertices {
        min.x = min.x.min(v.position.x);
        min.y = min.y.min(v.position.y);
        min.z = min.z.min(v.position.z);
        max.x = max.x.max(v.position.x);
        max.y = max.y.max(v.position.y);
        max.z = max.z.max(v.position.z);
    }
    (min, max)
}

#[derive(Clone, Copy)]
struct CellAccum {
    pos: Vector3,
    normal: Vector3,
    uv: [f32; 2],
    count: u32,
}

/// Decima una malla indexada usando una rejilla de `grid_res` celdas por eje.
pub fn decimate_vertex_clustering(
    vertices: &[Vertex],
    indices: &[usize],
    grid_res: usize,
) -> (Vec<Vertex>, Vec<usize>) {
    if vertices.is_empty() || indices.len() < 3 || grid_res == 0 {
        return (Vec::new(), Vec::new());
    }
    let (min, max) = mesh_bounds(vertices);
    let extent = max - min;
    let max_span = extent.x.max(extent.y).max(extent.z).max(1e-6);
    let cell_size = max_span / grid_res as f32;
    let inv_cell = 1.0 / cell_size;

    let mut cell_map: HashMap<(i32, i32, i32), usize> = HashMap::new();
    let mut cells: Vec<CellAccum> = Vec::new();
    let mut vert_to_cell = vec![0usize; vertices.len()];

    for (vi, v) in vertices.iter().enumerate() {
        let cx = ((v.position.x - min.x) * inv_cell).floor() as i32;
        let cy = ((v.position.y - min.y) * inv_cell).floor() as i32;
        let cz = ((v.position.z - min.z) * inv_cell).floor() as i32;
        let key = (cx, cy, cz);

        let cell_idx = *cell_map.entry(key).or_insert_with(|| {
            let idx = cells.len();
            cells.push(CellAccum {
                pos: Vector3::zero(),
                normal: Vector3::zero(),
                uv: [0.0, 0.0],
                count: 0,
            });
            idx
        });
        let acc = &mut cells[cell_idx];
        acc.pos += v.position;
        acc.normal += v.normal;
        acc.uv[0] += v.uv[0];
        acc.uv[1] += v.uv[1];
        acc.count += 1;
        vert_to_cell[vi] = cell_idx;
    }

    let out_vertices: Vec<Vertex> = cells
        .into_iter()
        .map(|acc| {
            let inv = 1.0 / acc.count.max(1) as f32;
            let pos = acc.pos * inv;
            let n = if acc.normal.length_sqr() > 1e-12 {
                acc.normal.normalize()
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            Vertex::with_uv(pos, n, [acc.uv[0] * inv, acc.uv[1] * inv])
        })
        .collect();

    let mut out_indices = Vec::with_capacity(indices.len());
    for tri in indices.as_chunks::<3>().0 {
        let a = vert_to_cell[tri[0]];
        let b = vert_to_cell[tri[1]];
        let c = vert_to_cell[tri[2]];
        if a == b || b == c || c == a {
            continue;
        }
        let pa = out_vertices[a].position;
        let pb = out_vertices[b].position;
        let pc = out_vertices[c].position;
        if (pb - pa).cross(pc - pa).length_sqr() <= 1e-12 {
            continue;
        }
        out_indices.push(a);
        out_indices.push(b);
        out_indices.push(c);
    }

    (out_vertices, out_indices)
}

/// Busca una resolución de rejilla que se acerque al número objetivo de triángulos.
pub fn decimate_to_target(
    vertices: &[Vertex],
    indices: &[usize],
    target_tris: usize,
) -> (Vec<Vertex>, Vec<usize>) {
    let mut lo = 3usize;
    let mut hi = 96usize;
    let mut best = decimate_vertex_clustering(vertices, indices, lo);
    let mut best_diff = (best.1.len() / 3).abs_diff(target_tris);

    while lo <= hi {
        let mid = (lo + hi) / 2;
        let candidate = decimate_vertex_clustering(vertices, indices, mid);
        let tris = candidate.1.len() / 3;
        let diff = tris.abs_diff(target_tris);
        if diff < best_diff {
            best = candidate;
            best_diff = diff;
        }
        if tris < target_tris {
            lo = mid + 1;
        } else if tris > target_tris {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        } else {
            break;
        }
    }

    best
}
