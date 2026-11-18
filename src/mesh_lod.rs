#![allow(dead_code)]
//! Decimación de mallas por agrupación de vértices en rejilla 3D (respaldo LOD).
//!
//! Divide la caja envolvente de la malla en celdas cúbicas uniformes, fusiona
//! los vértices que caen en la misma celda promediando posición, normal y UV,
//! y elimina los triángulos degenerados resultantes. Sirve como respaldo en
//! tiempo de carga cuando los archivos `ship_lod*.obj` de Blender no existen.

use crate::obj::load_obj;
use crate::vertex::Vertex;
use raylib::prelude::Vector3;
use std::collections::HashMap;
use std::path::Path;

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

pub struct ShipLods {
    pub levels: [(Vec<Vertex>, Vec<usize>); 3],
}

pub fn load_ship_lods() -> ShipLods {
    let specs = [
        ("assets/ship_lod0.obj", 3000usize),
        ("assets/ship_lod1.obj", 800usize),
        ("assets/ship_lod2.obj", 150usize),
    ];
    let mut base_mesh: Option<(Vec<Vertex>, Vec<usize>)> = None;

    let mut load_level = |path: &str, target: usize| -> (Vec<Vertex>, Vec<usize>) {
        if Path::new(path).exists()
            && let Ok(obj) = load_obj(path)
        {
            return obj.to_indexed_mesh();
        }
        let (base_v, base_i) = base_mesh.get_or_insert_with(|| {
            let obj = load_obj("assets/ship.obj").expect("assets/ship.obj debe existir");
            obj.to_indexed_mesh()
        });
        decimate_to_target(base_v, base_i, target)
    };

    let l0 = load_level(specs[0].0, specs[0].1);
    let l1 = load_level(specs[1].0, specs[1].1);
    let l2 = load_level(specs[2].0, specs[2].1);
    println!(
        "LODs de nave listos: lod0={} tris, lod1={} tris, lod2={} tris",
        l0.1.len() / 3,
        l1.1.len() / 3,
        l2.1.len() / 3,
    );
    ShipLods {
        levels: [l0, l1, l2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimation_reduces_triangles_and_preserves_bounding_box() {
        let obj = load_obj("assets/ship.obj").expect("ship.obj debe cargar");
        let (verts, indices) = obj.to_indexed_mesh();
        let orig_tris = indices.len() / 3;
        let (orig_min, orig_max) = mesh_bounds(&verts);
        let orig_span = (orig_max - orig_min).length();

        let (v0, i0) = decimate_to_target(&verts, &indices, 3000);
        let (v1, i1) = decimate_to_target(&verts, &indices, 800);
        let (v2, i2) = decimate_to_target(&verts, &indices, 150);

        let t0 = i0.len() / 3;
        let t1 = i1.len() / 3;
        let t2 = i2.len() / 3;
        println!("Ship fallback LODs -> orig: {orig_tris}, lod0: {t0}, lod1: {t1}, lod2: {t2}");

        assert!(t0 < orig_tris && t0 > t1 && t1 > t2 && t2 > 0);

        for dec_verts in [&v0, &v1, &v2] {
            let (dmin, dmax) = mesh_bounds(dec_verts);
            assert!(dmin.x >= orig_min.x - 1e-4 && dmax.x <= orig_max.x + 1e-4);
            assert!(dmin.y >= orig_min.y - 1e-4 && dmax.y <= orig_max.y + 1e-4);
            assert!(dmin.z >= orig_min.z - 1e-4 && dmax.z <= orig_max.z + 1e-4);
            let dec_span = (dmax - dmin).length();
            assert!(
                dec_span >= orig_span * 0.75,
                "Caja envolvente colapsada: {dec_span} vs {orig_span}"
            );
        }
    }
}
