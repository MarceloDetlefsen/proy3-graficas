//! Recorte de triángulos contra el plano near en espacio de recorte.
//!
//! Implementa Sutherland–Hodgman sobre la desigualdad `z >= -w` (convención
//! mano derecha con cámara hacia `-Z` y `z_ndc` en `[-1, 1]`), antes de la
//! división perspectiva por `w`. Cada triángulo de entrada produce 0, 1 o 2
//! triángulos recortados con todos sus atributos interpolados linealmente en
//! espacio de recorte, sin asignaciones en el heap.

use raylib::prelude::Vector3;

use crate::math::Vec4;

#[derive(Clone, Copy, Debug)]
pub struct ClipVertex {
    pub clip_pos: Vec4,
    pub object_pos: Vector3,
    pub world_pos: Vector3,
    pub normal: Vector3,
    pub uv: [f32; 2],
    pub color: Vector3,
    pub material_id: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ClippedTriangles {
    pub tris: [[ClipVertex; 3]; 2],
    pub count: usize,
}

impl ClippedTriangles {
    pub fn as_slice(&self) -> &[[ClipVertex; 3]] {
        &self.tris[..self.count]
    }
}

#[inline]
fn near_distance(v: &ClipVertex) -> f32 {
    v.clip_pos.z + v.clip_pos.w
}

#[inline]
fn lerp_vec3(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a + (b - a) * t
}

#[inline]
fn lerp_clip_vertex(a: &ClipVertex, b: &ClipVertex, t: f32) -> ClipVertex {
    ClipVertex {
        clip_pos: Vec4::new(
            a.clip_pos.x + (b.clip_pos.x - a.clip_pos.x) * t,
            a.clip_pos.y + (b.clip_pos.y - a.clip_pos.y) * t,
            a.clip_pos.z + (b.clip_pos.z - a.clip_pos.z) * t,
            a.clip_pos.w + (b.clip_pos.w - a.clip_pos.w) * t,
        ),
        object_pos: lerp_vec3(a.object_pos, b.object_pos, t),
        world_pos: lerp_vec3(a.world_pos, b.world_pos, t),
        normal: lerp_vec3(a.normal, b.normal, t),
        uv: [
            a.uv[0] + (b.uv[0] - a.uv[0]) * t,
            a.uv[1] + (b.uv[1] - a.uv[1]) * t,
        ],
        color: lerp_vec3(a.color, b.color, t),
        material_id: a.material_id + (b.material_id - a.material_id) * t,
    }
}

/// Recorta un triángulo contra el plano near (`z >= -w`) usando Sutherland–Hodgman.
/// Devuelve 0, 1 o 2 triángulos en un arreglo fijo en el stack.
pub fn clip_triangle_near(tri: &[ClipVertex; 3]) -> ClippedTriangles {
    // Recorte contra un único semiplano `z + w >= 0`.
    let mut out = [tri[0]; 4];
    let mut out_len = 0usize;

    for i in 0..3 {
        let cur = tri[i];
        let next = tri[(i + 1) % 3];
        let d_cur = near_distance(&cur);
        let d_next = near_distance(&next);
        let cur_in = d_cur >= 0.0;
        let next_in = d_next >= 0.0;

        if cur_in && next_in {
            if out_len < 4 {
                out[out_len] = next;
                out_len += 1;
            }
        } else if cur_in && !next_in {
            let denom = d_cur - d_next;
            let t = if denom.abs() > 1e-8 {
                (d_cur / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if out_len < 4 {
                out[out_len] = lerp_clip_vertex(&cur, &next, t);
                out_len += 1;
            }
        } else if !cur_in && next_in {
            let denom = d_cur - d_next;
            let t = if denom.abs() > 1e-8 {
                (d_cur / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if out_len < 4 {
                out[out_len] = lerp_clip_vertex(&cur, &next, t);
                out_len += 1;
            }
            if out_len < 4 {
                out[out_len] = next;
                out_len += 1;
            }
        }
    }
    let dummy = [tri[0], tri[0], tri[0]];
    if out_len < 3 {
        ClippedTriangles {
            tris: [dummy, dummy],
            count: 0,
        }
    } else if out_len == 3 {
        ClippedTriangles {
            tris: [[out[0], out[1], out[2]], dummy],
            count: 1,
        }
    } else {
        ClippedTriangles {
            tris: [[out[0], out[1], out[2]], [out[0], out[2], out[3]]],
            count: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_v(x: f32, y: f32, z: f32, w: f32, attr: f32) -> ClipVertex {
        ClipVertex {
            clip_pos: Vec4::new(x, y, z, w),
            object_pos: Vector3::new(attr, attr * 2.0, 0.0),
            world_pos: Vector3::new(attr, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            uv: [attr, 1.0 - attr],
            color: Vector3::new(1.0, 1.0, 1.0),
            material_id: 0.0,
        }
    }

    #[test]
    fn zero_vertices_behind_near_keeps_triangle() {
        let tri = [
            mk_v(-1.0, 0.0, 0.0, 2.0, 0.0),
            mk_v(1.0, 0.0, 0.0, 2.0, 0.5),
            mk_v(0.0, 1.0, 0.0, 2.0, 1.0),
        ];
        let clipped = clip_triangle_near(&tri);
        assert_eq!(clipped.count, 1);
    }

    #[test]
    fn all_vertices_behind_near_discards_triangle() {
        let tri = [
            mk_v(-1.0, 0.0, -3.0, 2.0, 0.0),
            mk_v(1.0, 0.0, -4.0, 2.0, 0.5),
            mk_v(0.0, 1.0, -2.5, 2.0, 1.0),
        ];
        let clipped = clip_triangle_near(&tri);
        assert_eq!(clipped.count, 0);
    }

    #[test]
    fn one_vertex_behind_near_splits_into_two_triangles_with_interpolated_attrs() {
        // v0 detrás (z+w = -1), v1 y v2 delante (z+w = +1) -> t = 0.5 en las dos aristas cortadas
        let tri = [
            mk_v(0.0, 0.0, -2.0, 1.0, 0.0),
            mk_v(2.0, 0.0, 0.0, 1.0, 1.0),
            mk_v(0.0, 2.0, 0.0, 1.0, 1.0),
        ];
        let clipped = clip_triangle_near(&tri);
        assert_eq!(clipped.count, 2);
        for t in clipped.as_slice() {
            for v in t {
                assert!(v.clip_pos.z + v.clip_pos.w >= -1e-5);
            }
        }
        // Debe contener vértices interpolados en z+w == 0 con attr == 0.5
        let has_mid = clipped
            .as_slice()
            .iter()
            .flatten()
            .any(|v| (v.clip_pos.z + v.clip_pos.w).abs() < 1e-5 && (v.uv[0] - 0.5).abs() < 1e-5);
        assert!(has_mid);
    }

    #[test]
    fn two_vertices_behind_near_produces_one_triangle_with_interpolated_attrs() {
        // v0 delante (z+w = +1), v1 detrás (z+w = -1 -> t=0.5), v2 detrás (z+w = -3 -> t=0.25)
        let tri = [
            mk_v(0.0, 0.0, 0.0, 1.0, 0.0),
            mk_v(2.0, 0.0, -2.0, 1.0, 1.0),
            mk_v(0.0, 2.0, -4.0, 1.0, 1.0),
        ];
        let clipped = clip_triangle_near(&tri);
        assert_eq!(clipped.count, 1);
        let out_tri = &clipped.as_slice()[0];
        for v in out_tri {
            assert!(v.clip_pos.z + v.clip_pos.w >= -1e-5);
        }
        assert!((out_tri[0].uv[0] - 0.5).abs() < 1e-5);
        assert!((out_tri[1].uv[0] - 0.25).abs() < 1e-5);
        assert!((out_tri[2].uv[0] - 0.0).abs() < 1e-5);
    }
}
