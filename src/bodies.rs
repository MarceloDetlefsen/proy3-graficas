#![allow(dead_code)]
//! Cuerpos procedurales del sistema de favoritos construidos con `mesh_builder`.

use std::f32::consts::FRAC_PI_2;

use raylib::prelude::Vector3;

use crate::mesh_builder::{Mesh, box_mesh, extruded_prism, loft_rings, rounded_box};

const BLACK: Vector3 = Vector3 { x: 0.06, y: 0.06, z: 0.07 };
const WHITE: Vector3 = Vector3 { x: 0.96, y: 0.96, z: 0.94 };
const YELLOW: Vector3 = Vector3 { x: 0.98, y: 0.84, z: 0.08 };
const RED: Vector3 = Vector3 { x: 0.86, y: 0.14, z: 0.14 };
const ORANGE: Vector3 = Vector3 { x: 0.96, y: 0.44, z: 0.06 };
const GREEN: Vector3 = Vector3 { x: 0.10, y: 0.72, z: 0.24 };
const BLUE: Vector3 = Vector3 { x: 0.10, y: 0.34, z: 0.84 };

fn paint_box_outer_faces(m: &mut Mesh, gx: i32, gy: i32, gz: i32) {
    m.paint_vertices(|_, _, n| {
        if gz == 1 && n.z > 0.7 {
            RED
        } else if gz == -1 && n.z < -0.7 {
            ORANGE
        } else if gx == 1 && n.x > 0.7 {
            GREEN
        } else if gx == -1 && n.x < -0.7 {
            BLUE
        } else if gy == 1 && n.y > 0.7 {
            WHITE
        } else if gy == -1 && n.y < -0.7 {
            YELLOW
        } else {
            BLACK
        }
    });
}

/// Construye el Cubo de Rubik en 3 niveles de detalle (`0` = bajo, `1` = medio, `2` = alto).
pub fn build_rubik_mesh(detail: usize) -> Mesh {
    match detail {
        0 => {
            // Nivel 0: cubo único con cuadrícula 3x3 coloreada por cara (108 triángulos).
            let mut mesh = Mesh::new();
            let s = 0.95_f32;
            let step = (2.0 * s) / 3.0;
            let faces: [(Vector3, Vector3, Vector3, Vector3); 6] = [
                (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0), RED),
                (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0), ORANGE),
                (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 1.0, 0.0), GREEN),
                (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0), BLUE),
                (Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), WHITE),
                (Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), YELLOW),
            ];
            for (n, u, v, col) in faces {
                for row in 0..3 {
                    for cell in 0..3 {
                        let u0 = -s + cell as f32 * step;
                        let u1 = u0 + step;
                        let v0 = -s + row as f32 * step;
                        let v1 = v0 + step;
                        let p0 = n * s + u * u0 + v * v0;
                        let p1 = n * s + u * u1 + v * v0;
                        let p2 = n * s + u * u1 + v * v1;
                        let p3 = n * s + u * u0 + v * v1;
                        let base = mesh.positions.len();
                        for p in [p0, p1, p2, p3] {
                            let idx = mesh.push_vertex(p, n, [0.0, 0.0]);
                            mesh.colors[idx] = col;
                        }
                        mesh.push_quad(base, base + 1, base + 2, base + 3);
                    }
                }
            }
            mesh
        }
        1 => {
            // Nivel 1: 27 cubitos con colores por cara sin geometría extra de calcomanías (324 tris).
            let mut mesh = Mesh::new();
            let pitch = 0.64_f32;
            let size = 0.58_f32;
            for gx in -1..=1 {
                for gy in -1..=1 {
                    for gz in -1..=1 {
                        let mut cubie = box_mesh(size, size, size);
                        paint_box_outer_faces(&mut cubie, gx, gy, gz);
                        cubie.translate(Vector3::new(
                            gx as f32 * pitch,
                            gy as f32 * pitch,
                            gz as f32 * pitch,
                        ));
                        mesh.merge(&cubie);
                    }
                }
            }
            mesh
        }
        _ => {
            // Nivel 2: 27 cubitos biselados negros con holgura + 54 calcomanías delgadas (3564 tris).
            let mut mesh = Mesh::new();
            let pitch = 0.64_f32;
            let size = 0.58_f32;
            let sticker_w = 0.48_f32;
            let sticker_t = 0.03_f32;
            let half = size * 0.5 + sticker_t * 0.45;

            for gx in -1..=1 {
                for gy in -1..=1 {
                    for gz in -1..=1 {
                        let center = Vector3::new(
                            gx as f32 * pitch,
                            gy as f32 * pitch,
                            gz as f32 * pitch,
                        );
                        let mut cubie = rounded_box(size, size, size, 0.05, 1);
                        cubie.set_color(BLACK);
                        cubie.translate(center);
                        mesh.merge(&cubie);

                        let sticker_specs = [
                            (gz == 1, sticker_w, sticker_w, sticker_t, Vector3::new(0.0, 0.0, half), RED),
                            (gz == -1, sticker_w, sticker_w, sticker_t, Vector3::new(0.0, 0.0, -half), ORANGE),
                            (gx == 1, sticker_t, sticker_w, sticker_w, Vector3::new(half, 0.0, 0.0), GREEN),
                            (gx == -1, sticker_t, sticker_w, sticker_w, Vector3::new(-half, 0.0, 0.0), BLUE),
                            (gy == 1, sticker_w, sticker_t, sticker_w, Vector3::new(0.0, half, 0.0), WHITE),
                            (gy == -1, sticker_w, sticker_t, sticker_w, Vector3::new(0.0, -half, 0.0), YELLOW),
                        ];
                        for (enabled, sx, sy, sz, offset, color) in sticker_specs {
                            if enabled {
                                let mut sticker = box_mesh(sx, sy, sz);
                                sticker.set_color(color);
                                sticker.translate(center + offset);
                                mesh.merge(&sticker);
                            }
                        }
                    }
                }
            }
            mesh
        }
    }
}

/// Construye la Trifuerza con tres prismas triangulares equiláteros y el centro vacío.
pub fn build_triforce_mesh(detail: usize) -> Mesh {
    let side = 0.88_f32;
    let h = side * 3.0_f32.sqrt() * 0.5;
    let r_top = h * (2.0 / 3.0);
    let r_bot = h * (1.0 / 3.0);
    let base_tri = [
        [0.0, -r_top],
        [side * 0.5, r_bot],
        [-side * 0.5, r_bot],
    ];
    let scale_poly = |s: f32| -> Vec<[f32; 2]> {
        base_tri.iter().map(|&[x, z]| [x * s, z * s]).collect()
    };

    let build_prism = || -> Mesh {
        let mut p = match detail {
            0 => extruded_prism(&base_tri, 0.24),
            1 => loft_rings(&[
                (-0.12, scale_poly(0.90)),
                (0.0, scale_poly(1.0)),
                (0.12, scale_poly(0.90)),
            ]),
            _ => loft_rings(&[
                (-0.13, scale_poly(0.84)),
                (-0.09, scale_poly(0.96)),
                (0.0, scale_poly(1.0)),
                (0.09, scale_poly(0.96)),
                (0.13, scale_poly(0.84)),
            ]),
        };
        p.rotate_x(FRAC_PI_2);
        p.compute_flat_normals();
        p
    };

    let gap_s = side * 1.04;
    let gap_h = h * 1.04;
    let offsets = [
        Vector3::new(0.0, gap_h * 0.5, 0.0),
        Vector3::new(-gap_s * 0.5, -gap_h * 0.5, 0.0),
        Vector3::new(gap_s * 0.5, -gap_h * 0.5, 0.0),
    ];
    let mut mesh = Mesh::new();
    for off in offsets {
        let mut prism = build_prism();
        prism.set_color(Vector3::new(0.95, 0.78, 0.20));
        prism.translate(off);
        mesh.merge(&prism);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubik_builds_three_decreasing_watertight_lods_under_4000_tris() {
        let m2 = build_rubik_mesh(2);
        let m1 = build_rubik_mesh(1);
        let m0 = build_rubik_mesh(0);

        assert!(m2.tri_count() <= 4000, "Rubik L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.is_watertight());
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
        }
    }

    #[test]
    fn triforce_builds_three_decreasing_watertight_lods_under_4000_tris() {
        let m2 = build_triforce_mesh(2);
        let m1 = build_triforce_mesh(1);
        let m0 = build_triforce_mesh(0);

        assert!(m2.tri_count() <= 4000);
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.is_watertight());
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
        }
    }
}
