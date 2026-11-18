#![allow(dead_code)]
//! Marcadores de posición procedurales y cargador de modelos de superficie.
//!
//! Genera mallas de respaldo por código para árbol, roca, edificio y monumento
//! siguiendo la convención del proyecto (origen en el centro de la base, `+Y`
//! arriba, altura `1.0` y <= 300 triángulos), y carga los `.obj` desde
//! `assets/surface/` cuando estén disponibles.

use raylib::prelude::Vector3;

use crate::mesh_lod::decimate_vertex_clustering;
use crate::obj::load_obj;
use crate::surface_scatter::SurfacePropKind;
use crate::vertex::Vertex;
use std::path::Path;

fn push_tri(out: &mut Vec<Vertex>, a: Vector3, b: Vector3, c: Vector3) {
    let e1 = b - a;
    let e2 = c - a;
    let cross = e1.cross(e2);
    let n = if cross.length_sqr() > 1e-12 {
        cross.normalize()
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    out.push(Vertex::with_uv(a, n, [0.0, 0.0]));
    out.push(Vertex::with_uv(b, n, [1.0, 0.0]));
    out.push(Vertex::with_uv(c, n, [0.5, 1.0]));
}

fn push_quad(out: &mut Vec<Vertex>, a: Vector3, b: Vector3, c: Vector3, d: Vector3) {
    push_tri(out, a, b, c);
    push_tri(out, a, c, d);
}

fn push_prism(
    out: &mut Vec<Vertex>,
    sides: usize,
    y0: f32,
    r0: f32,
    y1: f32,
    r1: f32,
) {
    let step = std::f32::consts::TAU / sides as f32;
    for i in 0..sides {
        let a0 = step * i as f32;
        let a1 = step * ((i + 1) % sides) as f32;
        let (s0, c0) = a0.sin_cos();
        let (s1, c1) = a1.sin_cos();
        let p00 = Vector3::new(c0 * r0, y0, s0 * r0);
        let p10 = Vector3::new(c1 * r0, y0, s1 * r0);
        let p01 = Vector3::new(c0 * r1, y1, s0 * r1);
        let p11 = Vector3::new(c1 * r1, y1, s1 * r1);
        push_quad(out, p00, p01, p11, p10);
    }
}

fn push_cone(out: &mut Vec<Vertex>, sides: usize, y0: f32, r0: f32, y_tip: f32) {
    let step = std::f32::consts::TAU / sides as f32;
    let tip = Vector3::new(0.0, y_tip, 0.0);
    let base_center = Vector3::new(0.0, y0, 0.0);
    for i in 0..sides {
        let a0 = step * i as f32;
        let a1 = step * ((i + 1) % sides) as f32;
        let p0 = Vector3::new(a0.cos() * r0, y0, a0.sin() * r0);
        let p1 = Vector3::new(a1.cos() * r0, y0, a1.sin() * r0);
        push_tri(out, p0, tip, p1);
        push_tri(out, base_center, p0, p1);
    }
}

fn push_box(out: &mut Vec<Vertex>, half_x: f32, y0: f32, y1: f32, half_z: f32) {
    let v000 = Vector3::new(-half_x, y0, -half_z);
    let v100 = Vector3::new(half_x, y0, -half_z);
    let v101 = Vector3::new(half_x, y0, half_z);
    let v001 = Vector3::new(-half_x, y0, half_z);
    let v010 = Vector3::new(-half_x, y1, -half_z);
    let v110 = Vector3::new(half_x, y1, -half_z);
    let v111 = Vector3::new(half_x, y1, half_z);
    let v011 = Vector3::new(-half_x, y1, half_z);

    push_quad(out, v001, v101, v111, v011); // +Z
    push_quad(out, v100, v000, v010, v110); // -Z
    push_quad(out, v101, v100, v110, v111); // +X
    push_quad(out, v000, v001, v011, v010); // -X
    push_quad(out, v011, v111, v110, v010); // +Y
    push_quad(out, v000, v100, v101, v001); // -Y
}

pub fn placeholder_mesh(kind: SurfacePropKind) -> Vec<Vertex> {
    let mut tris = Vec::new();
    match kind {
        SurfacePropKind::Tree => {
            push_prism(&mut tris, 6, 0.0, 0.09, 0.42, 0.07);
            push_cone(&mut tris, 7, 0.34, 0.34, 0.78);
            push_cone(&mut tris, 7, 0.60, 0.24, 1.00);
        }
        SurfacePropKind::Rock => {
            push_prism(&mut tris, 6, 0.0, 0.38, 0.48, 0.44);
            push_cone(&mut tris, 6, 0.48, 0.44, 0.95);
            push_prism(&mut tris, 5, 0.0, 0.28, 0.55, 0.18);
        }
        SurfacePropKind::Building => {
            push_box(&mut tris, 0.30, 0.0, 0.45, 0.26);
            push_box(&mut tris, 0.20, 0.45, 0.84, 0.20);
            push_box(&mut tris, 0.04, 0.84, 1.00, 0.04);
        }
        SurfacePropKind::Monument => {
            push_box(&mut tris, 0.26, 0.0, 0.20, 0.26);
            push_prism(&mut tris, 4, 0.20, 0.20, 0.86, 0.11);
            push_cone(&mut tris, 4, 0.86, 0.11, 1.00);
        }
    }
    tris
}

pub fn low_placeholder_mesh(kind: SurfacePropKind) -> Vec<Vertex> {
    let mut tris = Vec::new();
    match kind {
        SurfacePropKind::Tree => push_cone(&mut tris, 4, 0.0, 0.30, 1.00),
        SurfacePropKind::Rock => push_box(&mut tris, 0.30, 0.0, 0.70, 0.30),
        SurfacePropKind::Building => push_box(&mut tris, 0.24, 0.0, 0.90, 0.24),
        SurfacePropKind::Monument => push_cone(&mut tris, 4, 0.0, 0.22, 1.00),
    }
    tris
}

#[derive(Clone, Debug)]
pub struct PropMeshLod {
    pub high: Vec<Vertex>,
    pub low: Vec<Vertex>,
}

#[derive(Clone, Debug)]
pub struct PropMeshSet {
    pub tree: PropMeshLod,
    pub rock: PropMeshLod,
    pub building: PropMeshLod,
    pub monument: PropMeshLod,
}

impl PropMeshSet {
    pub fn mesh_for(&self, kind: SurfacePropKind, lod: usize) -> &[Vertex] {
        let entry = match kind {
            SurfacePropKind::Tree => &self.tree,
            SurfacePropKind::Rock => &self.rock,
            SurfacePropKind::Building => &self.building,
            SurfacePropKind::Monument => &self.monument,
        };
        if lod == 0 {
            &entry.high
        } else {
            &entry.low
        }
    }
}

fn load_or_fallback(kind: SurfacePropKind, path: &str) -> PropMeshLod {
    if Path::new(path).exists() {
        let obj = load_obj(path).expect("No se pudo cargar modelo de superficie");
        let (verts, idxs) = obj.to_indexed_mesh();
        let high: Vec<Vertex> = idxs.iter().map(|&i| verts[i].clone()).collect();
        let (lv, li) = decimate_vertex_clustering(&verts, &idxs, 4);
        let low = if li.is_empty() {
            low_placeholder_mesh(kind)
        } else {
            li.iter().map(|&i| lv[i].clone()).collect()
        };
        return PropMeshLod { high, low };
    }
    println!("Aviso: no se encontró {path}; usando marcador procedural de respaldo.");
    PropMeshLod {
        high: placeholder_mesh(kind),
        low: low_placeholder_mesh(kind),
    }
}

/// Carga los modelos de superficie desde `assets/surface/` con respaldo procedural.
pub fn load_surface_props() -> PropMeshSet {
    PropMeshSet {
        tree: load_or_fallback(SurfacePropKind::Tree, "assets/surface/tree.obj"),
        rock: load_or_fallback(SurfacePropKind::Rock, "assets/surface/rock.obj"),
        building: load_or_fallback(SurfacePropKind::Building, "assets/surface/building.obj"),
        monument: load_or_fallback(SurfacePropKind::Monument, "assets/surface/monument.obj"),
    }
}
