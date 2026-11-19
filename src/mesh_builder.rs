#![allow(dead_code)]
//! Constructor de mallas procedurales cerradas (estancas) con primitivas,
//! transformaciones y verificación de normales.

use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

use raylib::prelude::Vector3;

use crate::vertex::Vertex;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub positions: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub colors: Vec<Vector3>,
    pub material_ids: Vec<u8>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<usize>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            material_ids: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn tri_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn bounding_radius(&self) -> f32 {
        self.positions
            .iter()
            .map(|p| p.length())
            .fold(0.0_f32, f32::max)
    }

    pub fn push_vertex(&mut self, pos: Vector3, normal: Vector3, uv: [f32; 2]) -> usize {
        let idx = self.positions.len();
        self.positions.push(pos);
        self.normals.push(normal.normalize());
        self.colors.push(Vector3::new(1.0, 1.0, 1.0));
        self.material_ids.push(0);
        self.uvs.push(uv);
        idx
    }

    pub fn push_triangle(&mut self, i0: usize, i1: usize, i2: usize) {
        self.indices.extend_from_slice(&[i0, i1, i2]);
    }

    pub fn push_quad(&mut self, i0: usize, i1: usize, i2: usize, i3: usize) {
        self.indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
    }

    pub fn translate(&mut self, offset: Vector3) {
        for p in &mut self.positions {
            *p += offset;
        }
    }

    pub fn rotate_x(&mut self, angle: f32) {
        let (s, c) = angle.sin_cos();
        for p in &mut self.positions {
            *p = Vector3::new(p.x, p.y * c - p.z * s, p.y * s + p.z * c);
        }
        for n in &mut self.normals {
            *n = Vector3::new(n.x, n.y * c - n.z * s, n.y * s + n.z * c).normalize();
        }
    }

    pub fn rotate_y(&mut self, angle: f32) {
        let (s, c) = angle.sin_cos();
        for p in &mut self.positions {
            *p = Vector3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c);
        }
        for n in &mut self.normals {
            *n = Vector3::new(n.x * c + n.z * s, n.y, -n.x * s + n.z * c).normalize();
        }
    }

    pub fn rotate_z(&mut self, angle: f32) {
        let (s, c) = angle.sin_cos();
        for p in &mut self.positions {
            *p = Vector3::new(p.x * c - p.y * s, p.x * s + p.y * c, p.z);
        }
        for n in &mut self.normals {
            *n = Vector3::new(n.x * c - n.y * s, n.x * s + n.y * c, n.z).normalize();
        }
    }

    pub fn scale(&mut self, s: Vector3) {
        let inv = Vector3::new(
            if s.x.abs() > 1e-6 { 1.0 / s.x } else { 1.0 },
            if s.y.abs() > 1e-6 { 1.0 / s.y } else { 1.0 },
            if s.z.abs() > 1e-6 { 1.0 / s.z } else { 1.0 },
        );
        for p in &mut self.positions {
            *p = Vector3::new(p.x * s.x, p.y * s.y, p.z * s.z);
        }
        for n in &mut self.normals {
            let scaled = Vector3::new(n.x * inv.x, n.y * inv.y, n.z * inv.z);
            let len = scaled.length();
            if len > 1e-6 {
                *n = scaled * (1.0 / len);
            }
        }
    }

    /// Reconstruye la malla con vértices independientes por triángulo y normales planas.
    pub fn compute_flat_normals(&mut self) {
        let mut new_pos = Vec::with_capacity(self.indices.len());
        let mut new_nrm = Vec::with_capacity(self.indices.len());
        let mut new_col = Vec::with_capacity(self.indices.len());
        let mut new_mat = Vec::with_capacity(self.indices.len());
        let mut new_uv = Vec::with_capacity(self.indices.len());
        let mut new_idx = Vec::with_capacity(self.indices.len());

        for tri in self.indices.as_chunks::<3>().0 {
            let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
            let p0 = self.positions[i0];
            let p1 = self.positions[i1];
            let p2 = self.positions[i2];
            let e1 = p1 - p0;
            let e2 = p2 - p0;
            let cross = e1.cross(e2);
            let n = if cross.length_sqr() > 1e-12 {
                cross.normalize()
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            let base = new_pos.len();
            for &idx in &[i0, i1, i2] {
                new_pos.push(self.positions[idx]);
                new_nrm.push(n);
                new_col.push(self.colors[idx]);
                new_mat.push(self.material_ids[idx]);
                new_uv.push(self.uvs[idx]);
            }
            new_idx.extend_from_slice(&[base, base + 1, base + 2]);
        }

        self.positions = new_pos;
        self.normals = new_nrm;
        self.colors = new_col;
        self.material_ids = new_mat;
        self.uvs = new_uv;
        self.indices = new_idx;
    }

    /// Promedia normales por posición espacial compartida (suavizado).
    pub fn compute_smooth_normals(&mut self) {
        let mut accum: HashMap<[i64; 3], Vector3> = HashMap::new();
        for tri in self.indices.as_chunks::<3>().0 {
            let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
            let p0 = self.positions[i0];
            let p1 = self.positions[i1];
            let p2 = self.positions[i2];
            let face_n = (p1 - p0).cross(p2 - p0);
            for &idx in &[i0, i1, i2] {
                let key = pos_key(self.positions[idx]);
                *accum.entry(key).or_insert_with(Vector3::zero) += face_n;
            }
        }
        for (i, pos) in self.positions.iter().enumerate() {
            if let Some(sum) = accum.get(&pos_key(*pos))
                && sum.length_sqr() > 1e-12
            {
                self.normals[i] = sum.normalize();
            }
        }
    }

    /// Verifica que la malla es cerrada (estanca): cada arista geométrica no dirigida
    /// es compartida por exactamente 2 triángulos con orientaciones opuestas.
    pub fn is_watertight(&self) -> bool {
        if self.indices.is_empty() || !self.indices.len().is_multiple_of(3) {
            return false;
        }
        let mut directed: HashMap<([i64; 3], [i64; 3]), usize> = HashMap::new();
        let mut undirected: HashMap<([i64; 3], [i64; 3]), usize> = HashMap::new();

        for tri in self.indices.as_chunks::<3>().0 {
            let keys = [
                pos_key(self.positions[tri[0]]),
                pos_key(self.positions[tri[1]]),
                pos_key(self.positions[tri[2]]),
            ];
            for e in 0..3 {
                let a = keys[e];
                let b = keys[(e + 1) % 3];
                if a == b {
                    continue;
                }
                *directed.entry((a, b)).or_insert(0) += 1;
                let ukey = if a < b { (a, b) } else { (b, a) };
                *undirected.entry(ukey).or_insert(0) += 1;
            }
        }

        if undirected.is_empty() {
            return false;
        }
        for (&(a, b), &count) in &undirected {
            if count != 2 {
                return false;
            }
            if directed.get(&(a, b)).copied().unwrap_or(0) != 1
                || directed.get(&(b, a)).copied().unwrap_or(0) != 1
            {
                return false;
            }
        }
        true
    }

    /// Fracción de triángulos cuya normal geométrica apunta hacia afuera respecto al centroide.
    pub fn outward_normal_ratio(&self) -> f32 {
        if self.indices.is_empty() || self.positions.is_empty() {
            return 0.0;
        }
        let mut centroid = Vector3::zero();
        for &p in &self.positions {
            centroid += p;
        }
        centroid *= 1.0 / self.positions.len() as f32;

        let mut outward = 0usize;
        let mut total = 0usize;
        for tri in self.indices.as_chunks::<3>().0 {
            let p0 = self.positions[tri[0]];
            let p1 = self.positions[tri[1]];
            let p2 = self.positions[tri[2]];
            let face_n = (p1 - p0).cross(p2 - p0);
            if face_n.length_sqr() <= 1e-12 {
                continue;
            }
            let tri_center = (p0 + p1 + p2) * (1.0 / 3.0);
            if face_n.dot(tri_center - centroid) > 0.0 {
                outward += 1;
            }
            total += 1;
        }
        if total == 0 {
            0.0
        } else {
            outward as f32 / total as f32
        }
    }

    /// Volumen con signo (`> 0` si las normales apuntan hacia afuera en cualquier malla cerrada).
    pub fn signed_volume(&self) -> f32 {
        let mut sum = 0.0_f32;
        for tri in self.indices.as_chunks::<3>().0 {
            let p0 = self.positions[tri[0]];
            let p1 = self.positions[tri[1]];
            let p2 = self.positions[tri[2]];
            sum += p0.dot(p1.cross(p2));
        }
        sum / 6.0
    }

    pub fn set_color(&mut self, color: Vector3) {
        self.colors.fill(color);
    }

    pub fn set_material(&mut self, mat_id: u8) {
        self.material_ids.fill(mat_id);
    }

    pub fn paint_vertices(&mut self, mut f: impl FnMut(usize, Vector3, Vector3) -> Vector3) {
        for i in 0..self.positions.len() {
            self.colors[i] = f(i, self.positions[i], self.normals[i]);
        }
    }

    pub fn merge(&mut self, other: &Mesh) {
        let base = self.positions.len();
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.colors.extend_from_slice(&other.colors);
        self.material_ids.extend_from_slice(&other.material_ids);
        self.uvs.extend_from_slice(&other.uvs);
        self.indices.extend(other.indices.iter().map(|&idx| base + idx));
    }

    pub fn to_vertex_array(&self) -> Vec<Vertex> {
        self.indices
            .iter()
            .map(|&idx| {
                let mut v = Vertex::with_uv(self.positions[idx], self.normals[idx], self.uvs[idx]);
                v.color = self.colors[idx];
                v.material_id = self.material_ids[idx] as f32;
                v
            })
            .collect()
    }
}

#[inline]
fn pos_key(p: Vector3) -> [i64; 3] {
    [
        (p.x * 10_000.0).round() as i64,
        (p.y * 10_000.0).round() as i64,
        (p.z * 10_000.0).round() as i64,
    ]
}

/// Caja axis-aligned centrada en el origen con 12 triángulos y normales planas.
pub fn box_mesh(sx: f32, sy: f32, sz: f32) -> Mesh {
    let hx = sx * 0.5;
    let hy = sy * 0.5;
    let hz = sz * 0.5;
    let mut mesh = Mesh::new();

    let faces: [([Vector3; 4], Vector3); 6] = [
        // +Z
        (
            [
                Vector3::new(-hx, -hy, hz),
                Vector3::new(hx, -hy, hz),
                Vector3::new(hx, hy, hz),
                Vector3::new(-hx, hy, hz),
            ],
            Vector3::new(0.0, 0.0, 1.0),
        ),
        // -Z
        (
            [
                Vector3::new(hx, -hy, -hz),
                Vector3::new(-hx, -hy, -hz),
                Vector3::new(-hx, hy, -hz),
                Vector3::new(hx, hy, -hz),
            ],
            Vector3::new(0.0, 0.0, -1.0),
        ),
        // +X
        (
            [
                Vector3::new(hx, -hy, hz),
                Vector3::new(hx, -hy, -hz),
                Vector3::new(hx, hy, -hz),
                Vector3::new(hx, hy, hz),
            ],
            Vector3::new(1.0, 0.0, 0.0),
        ),
        // -X
        (
            [
                Vector3::new(-hx, -hy, -hz),
                Vector3::new(-hx, -hy, hz),
                Vector3::new(-hx, hy, hz),
                Vector3::new(-hx, hy, -hz),
            ],
            Vector3::new(-1.0, 0.0, 0.0),
        ),
        // +Y
        (
            [
                Vector3::new(-hx, hy, hz),
                Vector3::new(hx, hy, hz),
                Vector3::new(hx, hy, -hz),
                Vector3::new(-hx, hy, -hz),
            ],
            Vector3::new(0.0, 1.0, 0.0),
        ),
        // -Y
        (
            [
                Vector3::new(-hx, -hy, -hz),
                Vector3::new(hx, -hy, -hz),
                Vector3::new(hx, -hy, hz),
                Vector3::new(-hx, -hy, hz),
            ],
            Vector3::new(0.0, -1.0, 0.0),
        ),
    ];

    let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    for (pts, n) in faces {
        let i0 = mesh.push_vertex(pts[0], n, uvs[0]);
        let i1 = mesh.push_vertex(pts[1], n, uvs[1]);
        let i2 = mesh.push_vertex(pts[2], n, uvs[2]);
        let i3 = mesh.push_vertex(pts[3], n, uvs[3]);
        mesh.push_quad(i0, i1, i2, i3);
    }
    mesh
}

/// Caja redondeada estanca con radio `radius` y `segments` subdivisiones por bisel.
pub fn rounded_box(sx: f32, sy: f32, sz: f32, radius: f32, segments: usize) -> Mesh {
    let seg = segments.max(1);
    let r = radius.min(sx * 0.45).min(sy * 0.45).min(sz * 0.45).max(1e-4);
    let hx = sx * 0.5;
    let hy = sy * 0.5;
    let hz = sz * 0.5;
    let ix = (hx - r).max(0.0);
    let iy = (hy - r).max(0.0);
    let iz = (hz - r).max(0.0);

    // Malla de rejilla por cara con 2*seg+1 celdas para concentrar detalle en los bordes.
    let g = seg * 2 + 1;
    let mut axis_coords = Vec::with_capacity(g + 1);
    for i in 0..=g {
        let t = if i <= seg {
            -1.0 + (i as f32 / seg as f32) * 0.25
        } else {
            1.0 - ((g - i) as f32 / seg as f32) * 0.25
        };
        axis_coords.push(t);
    }

    let project_point = |raw: Vector3| -> (Vector3, Vector3) {
        let p = Vector3::new(raw.x * hx, raw.y * hy, raw.z * hz);
        let clamped = Vector3::new(
            p.x.clamp(-ix, ix),
            p.y.clamp(-iy, iy),
            p.z.clamp(-iz, iz),
        );
        let delta = p - clamped;
        let len = delta.length();
        if len > 1e-6 {
            let n = delta * (1.0 / len);
            (clamped + n * r, n)
        } else {
            let ax = raw.x.abs();
            let ay = raw.y.abs();
            let az = raw.z.abs();
            let n = if ax >= ay && ax >= az {
                Vector3::new(raw.x.signum(), 0.0, 0.0)
            } else if ay >= az {
                Vector3::new(0.0, raw.y.signum(), 0.0)
            } else {
                Vector3::new(0.0, 0.0, raw.z.signum())
            };
            (p, n)
        }
    };

    let mut mesh = Mesh::new();
    // Cada cara definida por (origen_normal, eje_u, eje_v) con u × v = normal.
    let face_axes: [(Vector3, Vector3, Vector3); 6] = [
        (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)),
        (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)),
        (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 1.0, 0.0)),
        (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0)),
        (Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)),
        (Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
    ];

    for (n_axis, u_axis, v_axis) in face_axes {
        let base = mesh.positions.len();
        for &v in &axis_coords {
            for &u in &axis_coords {
                let raw = n_axis + u_axis * u + v_axis * v;
                let (pos, n) = project_point(raw);
                mesh.push_vertex(pos, n, [(u + 1.0) * 0.5, (v + 1.0) * 0.5]);
            }
        }
        let stride = g + 1;
        for row in 0..g {
            for col in 0..g {
                let i0 = base + row * stride + col;
                let i1 = i0 + 1;
                let i2 = i1 + stride;
                let i3 = i0 + stride;
                mesh.push_quad(i0, i1, i2, i3);
            }
        }
    }
    mesh
}

/// Cilindro cerrado centrado en el origen con radios superior e inferior y tapas.
pub fn cylinder(r_top: f32, r_bottom: f32, height: f32, segments: usize) -> Mesh {
    let seg = segments.max(3);
    let hy = height * 0.5;
    let mut mesh = Mesh::new();

    let slope_y = (r_bottom - r_top) / height.max(1e-5);
    let mut side_bot = Vec::with_capacity(seg + 1);
    let mut side_top = Vec::with_capacity(seg + 1);
    for i in 0..=seg {
        let u = i as f32 / seg as f32;
        let theta = u * TAU;
        let (s, c) = theta.sin_cos();
        let n = Vector3::new(c, slope_y, -s).normalize();
        let pb = Vector3::new(r_bottom * c, -hy, -r_bottom * s);
        let pt = Vector3::new(r_top * c, hy, -r_top * s);
        side_bot.push(mesh.push_vertex(pb, n, [u, 0.0]));
        side_top.push(mesh.push_vertex(pt, n, [u, 1.0]));
    }
    for i in 0..seg {
        mesh.push_quad(side_bot[i], side_bot[i + 1], side_top[i + 1], side_top[i]);
    }

    // Tapa superior (+Y)
    let top_center = mesh.push_vertex(
        Vector3::new(0.0, hy, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        [0.5, 0.5],
    );
    let mut top_ring = Vec::with_capacity(seg);
    for i in 0..seg {
        let theta = (i as f32 / seg as f32) * TAU;
        let (s, c) = theta.sin_cos();
        let p = Vector3::new(r_top * c, hy, -r_top * s);
        top_ring.push(mesh.push_vertex(p, Vector3::new(0.0, 1.0, 0.0), [0.5 + 0.5 * c, 0.5 - 0.5 * s]));
    }
    for i in 0..seg {
        mesh.push_triangle(top_center, top_ring[i], top_ring[(i + 1) % seg]);
    }

    // Tapa inferior (-Y)
    let bot_center = mesh.push_vertex(
        Vector3::new(0.0, -hy, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        [0.5, 0.5],
    );
    let mut bot_ring = Vec::with_capacity(seg);
    for i in 0..seg {
        let theta = (i as f32 / seg as f32) * TAU;
        let (s, c) = theta.sin_cos();
        let p = Vector3::new(r_bottom * c, -hy, -r_bottom * s);
        bot_ring.push(mesh.push_vertex(p, Vector3::new(0.0, -1.0, 0.0), [0.5 + 0.5 * c, 0.5 - 0.5 * s]));
    }
    for i in 0..seg {
        mesh.push_triangle(bot_center, bot_ring[(i + 1) % seg], bot_ring[i]);
    }

    mesh
}

/// Cono cerrado centrado en el origen (`-height/2` a `+height/2`).
pub fn cone(radius: f32, height: f32, segments: usize) -> Mesh {
    let seg = segments.max(3);
    let hy = height * 0.5;
    let mut mesh = Mesh::new();
    let slope_y = radius / height.max(1e-5);
    let apex = Vector3::new(0.0, hy, 0.0);

    for i in 0..seg {
        let t0 = (i as f32 / seg as f32) * TAU;
        let t1 = ((i + 1) as f32 / seg as f32) * TAU;
        let tm = 0.5 * (t0 + t1);
        let (s0, c0) = t0.sin_cos();
        let (s1, c1) = t1.sin_cos();
        let (sm, cm) = tm.sin_cos();
        let p0 = Vector3::new(radius * c0, -hy, -radius * s0);
        let p1 = Vector3::new(radius * c1, -hy, -radius * s1);
        let n0 = Vector3::new(c0, slope_y, -s0).normalize();
        let n1 = Vector3::new(c1, slope_y, -s1).normalize();
        let nm = Vector3::new(cm, slope_y, -sm).normalize();
        let i0 = mesh.push_vertex(p0, n0, [i as f32 / seg as f32, 0.0]);
        let i1 = mesh.push_vertex(p1, n1, [(i + 1) as f32 / seg as f32, 0.0]);
        let ia = mesh.push_vertex(apex, nm, [0.5, 1.0]);
        mesh.push_triangle(i0, i1, ia);
    }

    let bot_center = mesh.push_vertex(
        Vector3::new(0.0, -hy, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        [0.5, 0.5],
    );
    let mut bot_ring = Vec::with_capacity(seg);
    for i in 0..seg {
        let theta = (i as f32 / seg as f32) * TAU;
        let (s, c) = theta.sin_cos();
        let p = Vector3::new(radius * c, -hy, -radius * s);
        bot_ring.push(mesh.push_vertex(p, Vector3::new(0.0, -1.0, 0.0), [0.5 + 0.5 * c, 0.5 - 0.5 * s]));
    }
    for i in 0..seg {
        mesh.push_triangle(bot_center, bot_ring[(i + 1) % seg], bot_ring[i]);
    }
    mesh
}

/// Esfera UV estanca con `rings` paralelos y `sectors` meridianos.
pub fn sphere(radius: f32, rings: usize, sectors: usize) -> Mesh {
    let r_cnt = rings.max(2);
    let s_cnt = sectors.max(3);
    let mut mesh = Mesh::new();

    let north = mesh.push_vertex(
        Vector3::new(0.0, radius, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        [0.5, 0.0],
    );

    let mut ring_indices = Vec::with_capacity(r_cnt - 1);
    for r in 1..r_cnt {
        let v = r as f32 / r_cnt as f32;
        let phi = v * PI;
        let (sp, cp) = phi.sin_cos();
        let mut row = Vec::with_capacity(s_cnt);
        for s in 0..s_cnt {
            let u = s as f32 / s_cnt as f32;
            let theta = u * TAU;
            let (st, ct) = theta.sin_cos();
            let n = Vector3::new(sp * ct, cp, -sp * st).normalize();
            row.push(mesh.push_vertex(n * radius, n, [u, v]));
        }
        ring_indices.push(row);
    }

    let south = mesh.push_vertex(
        Vector3::new(0.0, -radius, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        [0.5, 1.0],
    );

    for s in 0..s_cnt {
        let sn = (s + 1) % s_cnt;
        mesh.push_triangle(north, ring_indices[0][s], ring_indices[0][sn]);
    }
    for r in 0..ring_indices.len().saturating_sub(1) {
        for s in 0..s_cnt {
            let sn = (s + 1) % s_cnt;
            mesh.push_quad(
                ring_indices[r][s],
                ring_indices[r + 1][s],
                ring_indices[r + 1][sn],
                ring_indices[r][sn],
            );
        }
    }
    let last = ring_indices.len() - 1;
    for s in 0..s_cnt {
        let sn = (s + 1) % s_cnt;
        mesh.push_triangle(south, ring_indices[last][sn], ring_indices[last][s]);
    }
    mesh
}

/// Toro estanco centrado en el origen sobre el plano XZ.
pub fn torus(major_r: f32, minor_r: f32, major_seg: usize, minor_seg: usize) -> Mesh {
    let maj = major_seg.max(3);
    let min = minor_seg.max(3);
    let mut mesh = Mesh::new();

    let mut grid = Vec::with_capacity(maj);
    for i in 0..maj {
        let u = (i as f32 / maj as f32) * TAU;
        let (su, cu) = u.sin_cos();
        let center_dir = Vector3::new(cu, 0.0, -su);
        let mut ring = Vec::with_capacity(min);
        for j in 0..min {
            let v = (j as f32 / min as f32) * TAU;
            let (sv, cv) = v.sin_cos();
            let offset = center_dir * (minor_r * cv) + Vector3::new(0.0, minor_r * sv, 0.0);
            let pos = center_dir * major_r + offset;
            let n = offset.normalize();
            ring.push(mesh.push_vertex(pos, n, [i as f32 / maj as f32, j as f32 / min as f32]));
        }
        grid.push(ring);
    }

    for i in 0..maj {
        let inext = (i + 1) % maj;
        for j in 0..min {
            let jnext = (j + 1) % min;
            mesh.push_quad(
                grid[i][j],
                grid[i][jnext],
                grid[inext][jnext],
                grid[inext][j],
            );
        }
    }
    mesh
}

/// Prisma extruido a partir de un polígono 2D convexo en sentido antihorario (XZ)
/// a lo largo del eje Y (`-height/2` a `+height/2`).
pub fn extruded_prism(poly_2d: &[[f32; 2]], height: f32) -> Mesh {
    let n = poly_2d.len();
    assert!(n >= 3, "Un prisma requiere al menos 3 vértices");
    let hy = height * 0.5;
    let mut mesh = Mesh::new();

    // Asegurar orientación CCW en el plano XZ (vista desde +Y: x*z' - z*x' < 0).
    let mut signed_area = 0.0_f32;
    for i in 0..n {
        let a = poly_2d[i];
        let b = poly_2d[(i + 1) % n];
        signed_area += a[0] * b[1] - b[0] * a[1];
    }
    let pts: Vec<[f32; 2]> = if signed_area > 0.0 {
        poly_2d.iter().copied().rev().collect()
    } else {
        poly_2d.to_vec()
    };

    // Tapa superior (+Y)
    let mut top_idx = Vec::with_capacity(n);
    for &[x, z] in &pts {
        top_idx.push(mesh.push_vertex(Vector3::new(x, hy, z), Vector3::new(0.0, 1.0, 0.0), [x, z]));
    }
    for i in 1..(n - 1) {
        mesh.push_triangle(top_idx[0], top_idx[i], top_idx[i + 1]);
    }

    // Tapa inferior (-Y)
    let mut bot_idx = Vec::with_capacity(n);
    for &[x, z] in &pts {
        bot_idx.push(mesh.push_vertex(Vector3::new(x, -hy, z), Vector3::new(0.0, -1.0, 0.0), [x, z]));
    }
    for i in 1..(n - 1) {
        mesh.push_triangle(bot_idx[0], bot_idx[i + 1], bot_idx[i]);
    }

    // Caras laterales
    for i in 0..n {
        let j = (i + 1) % n;
        let [x0, z0] = pts[i];
        let [x1, z1] = pts[j];
        let p0 = Vector3::new(x0, -hy, z0);
        let p1 = Vector3::new(x1, -hy, z1);
        let p2 = Vector3::new(x1, hy, z1);
        let p3 = Vector3::new(x0, hy, z0);
        let normal = (p1 - p0).cross(p3 - p0).normalize();
        let v0 = mesh.push_vertex(p0, normal, [0.0, 0.0]);
        let v1 = mesh.push_vertex(p1, normal, [1.0, 0.0]);
        let v2 = mesh.push_vertex(p2, normal, [1.0, 1.0]);
        let v3 = mesh.push_vertex(p3, normal, [0.0, 1.0]);
        mesh.push_quad(v0, v1, v2, v3);
    }

    mesh
}

/// Triangula un polígono 2D simple (convexo o cóncavo) en el plano XZ por recorte de orejas.
/// Asume orientación CCW vista desde `+Y` (`signed_area < 0` en coordenadas `(x, z)`).
pub fn triangulate_polygon_2d(pts: &[[f32; 2]]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![[0, 1, 2]];
    }
    let cross_2d = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| -> f32 {
        // Orientación en plano XZ vista desde +Y: (b-a) × (c-a) tiene componente Y = -(bx-ax)*(cz-az) + (bz-az)*(cx-ax)
        (b[1] - a[1]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[1] - a[1])
    };
    let point_in_tri = |p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]| -> bool {
        let c1 = cross_2d(a, b, p);
        let c2 = cross_2d(b, c, p);
        let c3 = cross_2d(c, a, p);
        c1 >= -1e-7 && c2 >= -1e-7 && c3 >= -1e-7
    };

    let mut rem: Vec<usize> = (0..n).collect();
    let mut tris = Vec::with_capacity(n - 2);
    let mut guard = 0usize;
    while rem.len() > 3 && guard < n * n + 16 {
        let m = rem.len();
        let mut clipped = false;
        for i in 0..m {
            let i0 = rem[(i + m - 1) % m];
            let i1 = rem[i];
            let i2 = rem[(i + 1) % m];
            let (a, b, c) = (pts[i0], pts[i1], pts[i2]);
            if cross_2d(a, b, c) <= 1e-8 {
                continue;
            }
            let mut has_inside = false;
            for &j in &rem {
                if j == i0 || j == i1 || j == i2 {
                    continue;
                }
                if point_in_tri(pts[j], a, b, c) {
                    has_inside = true;
                    break;
                }
            }
            if !has_inside {
                tris.push([i0, i1, i2]);
                rem.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            break;
        }
        guard += 1;
    }
    if rem.len() == 3 {
        tris.push([rem[0], rem[1], rem[2]]);
    }
    tris
}

fn rdp_open_2d(pts: &[[f32; 2]], eps: f32) -> Vec<[f32; 2]> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }
    let a = pts[0];
    let b = pts[pts.len() - 1];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let l2 = dx * dx + dy * dy;
    let mut best_i = 0usize;
    let mut best_d = -1.0_f32;
    for (i, &p) in pts.iter().enumerate().take(pts.len() - 1).skip(1) {
        let d = if l2 < 1e-12 {
            ((p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2)).sqrt()
        } else {
            let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0);
            ((p[0] - (a[0] + t * dx)).powi(2) + (p[1] - (a[1] + t * dy)).powi(2)).sqrt()
        };
        if d > best_d {
            best_d = d;
            best_i = i;
        }
    }
    if best_d > eps {
        let mut left = rdp_open_2d(&pts[..=best_i], eps);
        let right = rdp_open_2d(&pts[best_i..], eps);
        left.pop();
        left.extend_from_slice(&right);
        left
    } else {
        vec![a, b]
    }
}

/// Simplifica un polígono cerrado 2D con Ramer–Douglas–Peucker manteniendo al menos 4 vértices.
pub fn rdp_simplify_closed(poly: &[[f32; 2]], eps: f32) -> Vec<[f32; 2]> {
    if poly.len() <= 4 || eps <= 1e-6 {
        return poly.to_vec();
    }
    let p0 = poly[0];
    let far_i = poly
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let da = (a[0] - p0[0]).powi(2) + (a[1] - p0[1]).powi(2);
            let db = (b[0] - p0[0]).powi(2) + (b[1] - p0[1]).powi(2);
            da.total_cmp(&db)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);
    if far_i == 0 {
        return poly.to_vec();
    }
    let c1 = rdp_open_2d(&poly[..=far_i], eps);
    let mut second = poly[far_i..].to_vec();
    second.push(poly[0]);
    let c2 = rdp_open_2d(&second, eps);
    let mut out = Vec::with_capacity(c1.len() + c2.len());
    out.extend_from_slice(&c1[..c1.len().saturating_sub(1)]);
    out.extend_from_slice(&c2[..c2.len().saturating_sub(1)]);
    if out.len() < 4 {
        poly.to_vec()
    } else {
        out
    }
}

/// Construye una malla cerrada conectando una lista de anillos poligonales
/// `(altura_y, poligono_2d)` con el mismo número de vértices (`>= 3`).
pub fn loft_rings(rings: &[(f32, Vec<[f32; 2]>)]) -> Mesh {
    assert!(rings.len() >= 2, "loft_rings requiere al menos 2 anillos");
    let n = rings[0].1.len();
    assert!(n >= 3, "Cada anillo requiere al menos 3 vértices");
    for (_, poly) in rings {
        assert_eq!(poly.len(), n, "Todos los anillos deben tener el mismo número de vértices");
    }

    // Ordenar anillos por altura ascendente y asegurar orientación CCW en XZ.
    let mut sorted: Vec<(f32, Vec<[f32; 2]>)> = rings.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mut signed_area = 0.0_f32;
    for i in 0..n {
        let a = sorted[0].1[i];
        let b = sorted[0].1[(i + 1) % n];
        signed_area += a[0] * b[1] - b[0] * a[1];
    }
    if signed_area > 0.0 {
        for (_, poly) in &mut sorted {
            poly.reverse();
        }
    }

    let mut mesh = Mesh::new();
    let mut ring_idx = Vec::with_capacity(sorted.len());
    for (r_i, (y, poly)) in sorted.iter().enumerate() {
        let v_coord = r_i as f32 / (sorted.len() - 1) as f32;
        let mut row = Vec::with_capacity(n);
        for (i, &[x, z]) in poly.iter().enumerate() {
            let radial = Vector3::new(x, 0.0, z);
            let nrm = if radial.length_sqr() > 1e-8 {
                radial.normalize()
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            row.push(mesh.push_vertex(
                Vector3::new(x, *y, z),
                nrm,
                [i as f32 / n as f32, v_coord],
            ));
        }
        ring_idx.push(row);
    }

    for r in 0..(sorted.len() - 1) {
        for i in 0..n {
            let j = (i + 1) % n;
            mesh.push_quad(
                ring_idx[r][i],
                ring_idx[r][j],
                ring_idx[r + 1][j],
                ring_idx[r + 1][i],
            );
        }
    }

    // Tapa inferior (-Y)
    let (y_bot, poly_bot) = &sorted[0];
    let mut bot_cap = Vec::with_capacity(n);
    for &[x, z] in poly_bot {
        bot_cap.push(mesh.push_vertex(
            Vector3::new(x, *y_bot, z),
            Vector3::new(0.0, -1.0, 0.0),
            [x, z],
        ));
    }
    let bot_tris = triangulate_polygon_2d(poly_bot);
    for [a, b, c] in bot_tris {
        mesh.push_triangle(bot_cap[a], bot_cap[c], bot_cap[b]);
    }

    // Tapa superior (+Y)
    let (y_top, poly_top) = &sorted[sorted.len() - 1];
    let mut top_cap = Vec::with_capacity(n);
    for &[x, z] in poly_top {
        top_cap.push(mesh.push_vertex(
            Vector3::new(x, *y_top, z),
            Vector3::new(0.0, 1.0, 0.0),
            [x, z],
        ));
    }
    let top_tris = triangulate_polygon_2d(poly_top);
    for [a, b, c] in top_tris {
        mesh.push_triangle(top_cap[a], top_cap[b], top_cap[c]);
    }

    mesh.compute_smooth_normals();
    mesh
}

/// Encoge un polígono 3D convexo hacia su centroide por `factor` (`(0, 1]`).
pub fn inset_polygon(poly: &[Vector3], factor: f32) -> Vec<Vector3> {
    if poly.is_empty() {
        return Vec::new();
    }
    let mut c = Vector3::zero();
    for &p in poly {
        c += p;
    }
    c *= 1.0 / poly.len() as f32;
    poly.iter().map(|&p| c + (p - c) * factor).collect()
}

fn polygon_normal(poly: &[Vector3]) -> Vector3 {
    if poly.len() < 3 {
        return Vector3::new(0.0, 1.0, 0.0);
    }
    let cross = (poly[1] - poly[0]).cross(poly[2] - poly[0]);
    if cross.length_sqr() > 1e-12 {
        cross.normalize()
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    }
}

/// Construye un polígono plano convexo en 3D con normal hacia `(p1-p0)×(p2-p0)`.
pub fn flat_polygon_mesh(poly: &[Vector3], color: Vector3) -> Mesh {
    let n = poly.len();
    assert!(n >= 3, "Un polígono requiere al menos 3 vértices");
    let normal = polygon_normal(poly);
    let mut mesh = Mesh::new();
    let mut idxs = Vec::with_capacity(n);
    for &p in poly {
        let i = mesh.push_vertex(p, normal, [0.0, 0.0]);
        mesh.colors[i] = color;
        idxs.push(i);
    }
    for i in 1..(n - 1) {
        mesh.push_triangle(idxs[0], idxs[i], idxs[i + 1]);
    }
    mesh
}

/// Construye una pegatina poligonal encogida por `inset` y elevada `elevation`
/// sobre el plano de `poly`. Si `thickness > 0.0`, extruye caras laterales cerradas.
pub fn raised_sticker(
    poly: &[Vector3],
    inset: f32,
    elevation: f32,
    thickness: f32,
    color: Vector3,
) -> Mesh {
    let shrunk = inset_polygon(poly, inset);
    let nrm = polygon_normal(&shrunk);
    let top_pts: Vec<Vector3> = shrunk.iter().map(|&p| p + nrm * elevation).collect();
    if thickness <= 1e-6 {
        return flat_polygon_mesh(&top_pts, color);
    }
    let bot_pts: Vec<Vector3> = shrunk
        .iter()
        .map(|&p| p + nrm * (elevation - thickness))
        .collect();
    let n = top_pts.len();
    let mut mesh = Mesh::new();

    // Cara superior
    let mut top_idx = Vec::with_capacity(n);
    for &p in &top_pts {
        let idx = mesh.push_vertex(p, nrm, [0.0, 0.0]);
        mesh.colors[idx] = color;
        top_idx.push(idx);
    }
    for i in 1..(n - 1) {
        mesh.push_triangle(top_idx[0], top_idx[i], top_idx[i + 1]);
    }

    // Cara inferior
    let neg_nrm = nrm * -1.0;
    let mut bot_idx = Vec::with_capacity(n);
    for &p in &bot_pts {
        let idx = mesh.push_vertex(p, neg_nrm, [0.0, 0.0]);
        mesh.colors[idx] = color;
        bot_idx.push(idx);
    }
    for i in 1..(n - 1) {
        mesh.push_triangle(bot_idx[0], bot_idx[i + 1], bot_idx[i]);
    }

    // Caras laterales
    for i in 0..n {
        let j = (i + 1) % n;
        let p0 = bot_pts[i];
        let p1 = bot_pts[j];
        let p2 = top_pts[j];
        let p3 = top_pts[i];
        let side_n = (p1 - p0).cross(p3 - p0);
        let sn = if side_n.length_sqr() > 1e-12 {
            side_n.normalize()
        } else {
            nrm
        };
        let v0 = mesh.push_vertex(p0, sn, [0.0, 0.0]);
        let v1 = mesh.push_vertex(p1, sn, [1.0, 0.0]);
        let v2 = mesh.push_vertex(p2, sn, [1.0, 1.0]);
        let v3 = mesh.push_vertex(p3, sn, [0.0, 1.0]);
        for v in [v0, v1, v2, v3] {
            mesh.colors[v] = color;
        }
        mesh.push_quad(v0, v1, v2, v3);
    }
    mesh
}

/// Devuelve las 12 caras pentagonales (CCW vistas desde afuera) de un dodecaedro
/// regular con circunradio exacto `circumradius`.
pub fn dodecahedron_faces(circumradius: f32) -> Vec<[Vector3; 5]> {
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let inv_phi = 1.0 / phi;
    let scale = circumradius / 3.0_f32.sqrt();

    let mut verts = Vec::with_capacity(20);
    for &sx in &[-1.0_f32, 1.0] {
        for &sy in &[-1.0_f32, 1.0] {
            for &sz in &[-1.0_f32, 1.0] {
                verts.push(Vector3::new(sx, sy, sz) * scale);
            }
        }
    }
    for &s1 in &[-1.0_f32, 1.0] {
        for &s2 in &[-1.0_f32, 1.0] {
            verts.push(Vector3::new(0.0, s1 * phi, s2 * inv_phi) * scale);
            verts.push(Vector3::new(s1 * inv_phi, 0.0, s2 * phi) * scale);
            verts.push(Vector3::new(s1 * phi, s2 * inv_phi, 0.0) * scale);
        }
    }

    let mut face_dirs = Vec::with_capacity(12);
    for &s1 in &[-1.0_f32, 1.0] {
        for &s2 in &[-1.0_f32, 1.0] {
            face_dirs.push(Vector3::new(0.0, s1, s2 * phi).normalize());
            face_dirs.push(Vector3::new(s1, s2 * phi, 0.0).normalize());
            face_dirs.push(Vector3::new(s1 * phi, 0.0, s2).normalize());
        }
    }

    face_dirs
        .into_iter()
        .map(|dir| {
            let mut ranked: Vec<(f32, Vector3)> =
                verts.iter().map(|&v| (v.dot(dir), v)).collect();
            ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
            let mut five: [Vector3; 5] = [
                ranked[0].1,
                ranked[1].1,
                ranked[2].1,
                ranked[3].1,
                ranked[4].1,
            ];
            let ref_up = if dir.y.abs() < 0.9 {
                Vector3::new(0.0, 1.0, 0.0)
            } else {
                Vector3::new(1.0, 0.0, 0.0)
            };
            let u = ref_up.cross(dir).normalize();
            let v_ax = dir.cross(u).normalize();
            five.sort_by(|&a, &b| {
                let ang_a = a.dot(v_ax).atan2(a.dot(u));
                let ang_b = b.dot(v_ax).atan2(b.dot(u));
                ang_a.total_cmp(&ang_b)
            });
            five
        })
        .collect()
}

/// Dodecaedro regular cerrado con circunradio `circumradius` (36 triángulos).
pub fn dodecahedron(circumradius: f32) -> Mesh {
    let faces = dodecahedron_faces(circumradius);
    let mut mesh = Mesh::new();
    for face in faces {
        let poly = flat_polygon_mesh(&face, Vector3::new(1.0, 1.0, 1.0));
        mesh.merge(&poly);
    }
    mesh
}

/// Extruye un contorno 2D en XZ a lo largo de Y con bisel en ambas tapas.
pub fn beveled_extruded_contour(
    poly_2d: &[[f32; 2]],
    height: f32,
    bevel_scale: f32,
    bevel_height: f32,
) -> Mesh {
    let hy = height * 0.5;
    let bh = bevel_height.min(hy * 0.45).max(1e-4);
    let bs = bevel_scale.clamp(0.5, 0.99);
    let mut cx = 0.0_f32;
    let mut cz = 0.0_f32;
    for &[x, z] in poly_2d {
        cx += x;
        cz += z;
    }
    cx /= poly_2d.len().max(1) as f32;
    cz /= poly_2d.len().max(1) as f32;
    let inset_ring: Vec<[f32; 2]> = poly_2d
        .iter()
        .map(|&[x, z]| [cx + (x - cx) * bs, cz + (z - cz) * bs])
        .collect();
    let full_ring: Vec<[f32; 2]> = poly_2d.to_vec();
    let mut m = loft_rings(&[
        (-hy, inset_ring.clone()),
        (-hy + bh, full_ring.clone()),
        (hy - bh, full_ring),
        (hy, inset_ring),
    ]);
    m.compute_flat_normals();
    m
}

/// Perfil 2D CCW de rectángulo redondeado normalizado (`u ∈ [-0.5, 0.5]`, `v ∈ [-0.5, 0.5]`).
pub fn rounded_rect_profile_2d(corner_radius: f32, corner_segs: usize) -> Vec<[f32; 2]> {
    let r = corner_radius.clamp(0.0, 0.45);
    let segs = corner_segs.max(1);
    let inner = 0.5 - r;
    if r <= 1e-4 {
        return vec![[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
    }
    let fpi2 = std::f32::consts::FRAC_PI_2;
    let corners = [
        (inner, inner, 0.0_f32),
        (-inner, inner, fpi2),
        (-inner, -inner, PI),
        (inner, -inner, 3.0 * fpi2),
    ];
    let mut pts = Vec::with_capacity(4 * segs);
    for (cx, cy, start_a) in corners {
        for s in 0..segs {
            let a = start_a + fpi2 * (s as f32 / segs as f32);
            pts.push([cx + r * a.cos(), cy + r * a.sin()]);
        }
    }
    pts
}

/// Barre un perfil 2D (`u` en el plano XY perpendicular a la trayectoria, `v` en eje Z)
/// a lo largo de `path`, con ancho en el plano `widths[i]` y profundidad en Z `depths[i]`.
/// Si `closed` es `true`, cierra el lazo; si es `false`, agrega tapas estancas.
pub fn sweep_profile(
    path: &[Vector3],
    widths: &[f32],
    depths: &[f32],
    profile_unit: &[[f32; 2]],
    closed: bool,
) -> Mesh {
    let n = path.len();
    let p_len = profile_unit.len();
    assert!(n >= 2 && widths.len() == n && depths.len() == n && p_len >= 3);

    let mut m = Mesh::new();
    let z_axis = Vector3::new(0.0, 0.0, 1.0);
    let mut ring_starts = Vec::with_capacity(n);

    for i in 0..n {
        let prev = if i == 0 {
            if closed { path[n - 1] } else { path[0] }
        } else {
            path[i - 1]
        };
        let next = if i + 1 == n {
            if closed { path[0] } else { path[n - 1] }
        } else {
            path[i + 1]
        };
        let tangent = (next - prev).normalize();
        let normal_xy = tangent.cross(z_axis).normalize();

        let start = m.positions.len();
        ring_starts.push(start);
        for &[u, v] in profile_unit {
            let pos = path[i] + normal_xy * (u * widths[i]) + z_axis * (v * depths[i]);
            let norm = (normal_xy * u + z_axis * v).normalize();
            m.push_vertex(pos, norm, [u + 0.5, i as f32 / n as f32]);
        }
    }

    let seg_count = if closed { n } else { n - 1 };
    for i in 0..seg_count {
        let i_next = (i + 1) % n;
        let r0 = ring_starts[i];
        let r1 = ring_starts[i_next];
        for j in 0..p_len {
            let j_next = (j + 1) % p_len;
            let a0 = r0 + j;
            let a1 = r1 + j;
            let b0 = r0 + j_next;
            let b1 = r1 + j_next;
            m.push_triangle(a0, a1, b1);
            m.push_triangle(a0, b1, b0);
        }
    }

    if !closed {
        let t_start = (path[1] - path[0]).normalize();
        let c_start = m.push_vertex(path[0], -t_start, [0.5, 0.5]);
        let r0 = ring_starts[0];
        for j in 0..p_len {
            let j_next = (j + 1) % p_len;
            m.push_triangle(c_start, r0 + j, r0 + j_next);
        }

        let t_end = (path[n - 1] - path[n - 2]).normalize();
        let c_end = m.push_vertex(path[n - 1], t_end, [0.5, 0.5]);
        let rn = ring_starts[n - 1];
        for j in 0..p_len {
            let j_next = (j + 1) % p_len;
            m.push_triangle(c_end, rn + j_next, rn + j);
        }
    }

    m.compute_flat_normals();
    m
}

/// Dispersa copias de `proto` sobre `points` con escala y rotación reproducibles por `seed`.
pub fn scatter_instances_in_mesh(
    base: &mut Mesh,
    proto: &Mesh,
    points: &[Vector3],
    scale_min: f32,
    scale_max: f32,
    seed: u64,
) {
    let mut rng = crate::rng::SplitMix64::new(seed);
    for &pt in points {
        let s = rng.range_f32(scale_min, scale_max);
        let ry = rng.range_f32(0.0, TAU);
        let rx = rng.range_f32(-0.6, 0.6);
        let mut inst = proto.clone();
        inst.scale(Vector3::new(s, s, s));
        inst.rotate_x(rx);
        inst.rotate_y(ry);
        inst.translate(pt);
        base.merge(&inst);
    }
}

/// Caché de reconstrucción por nivel de detalle (`0` = bajo, `1` = medio, `2` = alto).
pub struct LodMeshCache {
    levels: [Option<Vec<Vertex>>; 3],
    tri_counts: [usize; 3],
    builder: fn(usize) -> Mesh,
}

impl LodMeshCache {
    pub fn new(builder: fn(usize) -> Mesh) -> Self {
        Self {
            levels: [None, None, None],
            tri_counts: [0, 0, 0],
            builder,
        }
    }

    pub fn get_or_build(&mut self, level: usize) -> &[Vertex] {
        let lvl = level.min(2);
        if self.levels[lvl].is_none() {
            let mesh = (self.builder)(lvl);
            self.tri_counts[lvl] = mesh.tri_count();
            self.levels[lvl] = Some(mesh.to_vertex_array());
        }
        self.levels[lvl].as_deref().unwrap_or(&[])
    }

    pub fn tri_count(&mut self, level: usize) -> usize {
        let lvl = level.min(2);
        let _ = self.get_or_build(lvl);
        self.tri_counts[lvl]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_have_exact_triangle_counts_watertight_and_outward_normals() {
        let b = box_mesh(1.0, 2.0, 3.0);
        assert_eq!(b.tri_count(), 12);
        assert!(b.is_watertight());
        assert!((b.outward_normal_ratio() - 1.0).abs() < 1e-5);

        let rb = rounded_box(1.0, 1.0, 1.0, 0.2, 2);
        let g = 2 * 2 + 1;
        assert_eq!(rb.tri_count(), 6 * 2 * g * g);
        assert!(rb.is_watertight());
        assert!((rb.outward_normal_ratio() - 1.0).abs() < 1e-5);

        let cyl = cylinder(0.5, 0.8, 2.0, 12);
        assert_eq!(cyl.tri_count(), 4 * 12);
        assert!(cyl.is_watertight());
        assert!((cyl.outward_normal_ratio() - 1.0).abs() < 1e-5);

        let cn = cone(1.0, 2.0, 16);
        assert_eq!(cn.tri_count(), 2 * 16);
        assert!(cn.is_watertight());
        assert!((cn.outward_normal_ratio() - 1.0).abs() < 1e-5);

        let sp = sphere(1.0, 8, 12);
        assert_eq!(sp.tri_count(), 2 * (8 - 1) * 12);
        assert!(sp.is_watertight());
        assert!((sp.outward_normal_ratio() - 1.0).abs() < 1e-5);

        let tr = torus(2.0, 0.5, 16, 8);
        assert_eq!(tr.tri_count(), 2 * 16 * 8);
        assert!(tr.is_watertight());

        let tri_poly = [[0.0, -1.0], [0.866, 0.5], [-0.866, 0.5]];
        let pr = extruded_prism(&tri_poly, 0.4);
        assert_eq!(pr.tri_count(), 4 * 3 - 4);
        assert!(pr.is_watertight());
        assert!((pr.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn transforms_preserve_watertightness_and_outward_normals() {
        let mut m = box_mesh(1.0, 1.0, 1.0);
        m.scale(Vector3::new(2.0, 0.5, 1.5));
        m.rotate_y(0.7);
        m.rotate_x(-0.3);
        m.translate(Vector3::new(3.0, -2.0, 1.0));
        assert!(m.is_watertight());
        assert!((m.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn merge_and_vertex_color_preserve_geometry() {
        let mut a = box_mesh(1.0, 1.0, 1.0);
        a.set_color(Vector3::new(1.0, 0.0, 0.0));
        let mut b = box_mesh(0.5, 0.5, 0.5);
        b.set_color(Vector3::new(0.0, 0.0, 1.0));
        b.translate(Vector3::new(2.0, 0.0, 0.0));
        a.merge(&b);

        assert_eq!(a.tri_count(), 24);
        assert!(a.is_watertight());
        let va = a.to_vertex_array();
        assert_eq!(va.len(), 72);
    }

    #[test]
    fn loft_rings_is_watertight_with_outward_normals() {
        let ring = |r: f32| -> Vec<[f32; 2]> {
            (0..8)
                .map(|i| {
                    let a = (i as f32 / 8.0) * TAU;
                    [r * a.cos(), -r * a.sin()]
                })
                .collect()
        };
        let mesh = loft_rings(&[(-1.0, ring(0.8)), (0.0, ring(1.2)), (1.0, ring(0.5))]);
        assert!(mesh.is_watertight());
        assert!((mesh.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn lod_mesh_cache_reconstructs_and_caches_levels() {
        fn build_test(detail: usize) -> Mesh {
            match detail {
                0 => sphere(1.0, 4, 6),
                1 => sphere(1.0, 6, 10),
                _ => sphere(1.0, 10, 16),
            }
        }
        let mut cache = LodMeshCache::new(build_test);
        let t0 = cache.tri_count(0);
        let t1 = cache.tri_count(1);
        let t2 = cache.tri_count(2);
        assert!(t0 < t1 && t1 < t2);
        assert_eq!(cache.get_or_build(1).len(), t1 * 3);
    }

    #[test]
    fn inset_and_raised_stickers_have_exact_counts_and_outward_normals() {
        let quad = [
            Vector3::new(-0.5, -0.5, 0.5),
            Vector3::new(0.5, -0.5, 0.5),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(-0.5, 0.5, 0.5),
        ];
        let shrunk = inset_polygon(&quad, 0.8);
        assert!((shrunk[0].x + 0.4).abs() < 1e-5);

        let flat = raised_sticker(&quad, 0.8, 0.02, 0.0, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(flat.tri_count(), 2);
        assert!((flat.normals[0].z - 1.0).abs() < 1e-5);

        let ext = raised_sticker(&quad, 0.8, 0.02, 0.015, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(ext.tri_count(), 12);
        assert!(ext.is_watertight());
        assert!((ext.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn regular_dodecahedron_has_12_pentagons_exact_radius_watertight_and_outward() {
        let faces = dodecahedron_faces(1.0);
        assert_eq!(faces.len(), 12);
        for face in &faces {
            for v in face {
                assert!((v.length() - 1.0).abs() < 1e-5);
            }
        }
        let d = dodecahedron(1.0);
        assert_eq!(d.tri_count(), 36);
        assert!(d.is_watertight());
        assert!((d.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn beveled_extruded_contour_is_watertight_and_outward() {
        let shield = [
            [-0.7, -0.8],
            [0.7, -0.8],
            [0.7, 0.2],
            [0.0, 0.9],
            [-0.7, 0.2],
        ];
        let m = beveled_extruded_contour(&shield, 0.2, 0.92, 0.04);
        assert!(m.is_watertight());
        assert!((m.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn concave_polygon_ear_clipping_and_extrusion_are_watertight() {
        // Polígono cóncavo en forma de L
        let l_shape = [
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            [0.0, 1.0],
            [-1.0, 1.0],
        ];
        let m = beveled_extruded_contour(&l_shape, 0.3, 0.92, 0.04);
        assert!(m.is_watertight());
        assert!((m.outward_normal_ratio() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn scatter_instances_in_mesh_is_reproducible_by_seed() {
        let proto = box_mesh(1.0, 1.0, 1.0);
        let pts = [
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        ];
        let mut a = Mesh::new();
        let mut b = Mesh::new();
        scatter_instances_in_mesh(&mut a, &proto, &pts, 0.02, 0.04, 123);
        scatter_instances_in_mesh(&mut b, &proto, &pts, 0.02, 0.04, 123);
        assert_eq!(a.tri_count(), 36);
        assert_eq!(a.positions.len(), b.positions.len());
        for (pa, pb) in a.positions.iter().zip(b.positions.iter()) {
            assert!((*pa - *pb).length() < 1e-6);
        }
    }

    #[test]
    fn sweep_profile_closed_and_open_are_watertight_with_outward_normals() {
        let profile = rounded_rect_profile_2d(0.22, 2);
        let mut loop_path = Vec::new();
        for i in 0..24 {
            let a = TAU * (i as f32 / 24.0);
            loop_path.push(Vector3::new(0.4 * a.cos(), 0.5 * a.sin(), 0.0));
        }
        let widths = vec![0.04; 24];
        let depths = vec![0.08; 24];
        let hoop = sweep_profile(&loop_path, &widths, &depths, &profile, true);
        assert!(hoop.is_watertight());
        assert!(hoop.signed_volume() > 0.0);

        let open_path = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.1, 0.3, 0.0),
            Vector3::new(0.25, 0.6, 0.0),
        ];
        let open = sweep_profile(&open_path, &[0.04, 0.045, 0.05], &[0.10, 0.09, 0.08], &profile, false);
        assert!(open.is_watertight());
        assert!(open.signed_volume() > 0.0);
    }
}
