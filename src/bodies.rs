#![allow(dead_code)]
//! Cuerpos procedurales del sistema de favoritos construidos con `mesh_builder`.

use std::f32::consts::FRAC_PI_2;

use raylib::prelude::Vector3;

use crate::mesh_builder::{
    Mesh, beveled_extruded_contour, box_mesh, cone, cylinder, dodecahedron, dodecahedron_faces,
    extruded_prism, flat_polygon_mesh, loft_rings, raised_sticker, rdp_simplify_closed,
    rounded_box, scatter_instances_in_mesh, sphere, torus,
};
use crate::noise::mix_color;
use crate::rng::SplitMix64;
use crate::shaders::meteor::{BLUE_GREEN, DARK_GREEN, TEAL};

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

/// Construye el Meteoro de FF7 como cometa:
/// - Cabeza (`material_id = 0`): icosfera de radio 1.
/// - Cola (`material_id = 1`): 24..=38 pinceladas en abanico aplanado con 6..=10 segmentos
///   y doble cara, con degradado por vértice de azul verdoso a verde casi negro.
/// - Orbe (`material_id = 2`): esfera pequeña (~6% del diámetro de la cabeza) junto a la cola.
pub fn build_meteor_mesh_with_seed(detail: usize, seed: u64) -> Mesh {
    let mut mesh = Mesh::new();

    // 1. Cabeza: icosfera de radio 1.0 (material 0).
    let ico_lvl = match detail {
        0 => 1, // 80 tris
        1 => 2, // 320 tris
        _ => 3, // 1280 tris
    };
    let ico = crate::icosphere::icosphere_level(ico_lvl);
    let mut head = Mesh::new();
    for &unit in &ico.positions {
        head.push_vertex(unit, unit, crate::icosphere::spherical_uv(unit));
    }
    head.indices = ico.indices;
    head.set_material(0);
    mesh.merge(&head);

    // 2. Cola: cintas delgadas de pinceladas en abanico aplanado hacia arriba a la derecha.
    let tail_dir = Vector3::new(0.86, 0.51, 0.0).normalize();
    let perp = Vector3::new(-tail_dir.y, tail_dir.x, 0.0).normalize();
    let z_axis = Vector3::new(0.0, 0.0, 1.0);

    let streaks = match detail {
        0 => 24,
        1 => 30,
        _ => 38,
    };
    let segs = match detail {
        0 => 6,
        1 => 8,
        _ => 10,
    };

    let mut rng = SplitMix64::new(seed);
    let mut tail = Mesh::new();
    for s_idx in 0..streaks {
        let frac = if streaks > 1 {
            (s_idx as f32 / (streaks - 1) as f32) * 2.0 - 1.0
        } else {
            0.0
        };
        let jitter_lat = (rng.next_f32() - 0.5) * 0.14;
        let lat = (frac * 0.92 + jitter_lat).clamp(-0.98, 0.98);
        let z_off = (rng.next_f32() - 0.5) * 0.28;
        let len = (3.4 + rng.next_f32() * 1.1) * (1.0 - 0.18 * lat.abs());
        let width0 = 0.065 + rng.next_f32() * 0.075;
        let curve = (rng.next_f32() - 0.5) * 0.16 + lat * 0.10;
        let freq = 2.0 + rng.next_f32() * 3.0;
        let phase = rng.next_f32() * std::f32::consts::TAU;

        // Punto de arranque en el hemisferio trasero de la cabeza y convergencia hacia la punta.
        let start_radial =
            tail_dir * (1.0 - lat * lat).max(0.05).sqrt() * 0.55 + perp * (lat * 0.96);
        let start_pos = start_radial + z_axis * z_off;
        let tip_pos = tail_dir * (len + 0.45) + perp * (lat * 0.06) + z_axis * (z_off * 0.2);

        let mut left_pts = Vec::with_capacity(segs + 1);
        let mut right_pts = Vec::with_capacity(segs + 1);
        let mut colors = Vec::with_capacity(segs + 1);

        for k in 0..=segs {
            let t = k as f32 / segs as f32;
            let wobble = (t * freq + phase).sin() * 0.035 * t * (1.0 - t);
            let center = start_pos + (tip_pos - start_pos) * t
                + perp * (curve * t * (1.0 - t) + wobble);
            let w = width0 * (1.0 - t).powf(1.25).max(0.02);
            left_pts.push(center + perp * w);
            right_pts.push(center - perp * w);

            let col = if t < 0.45 {
                mix_color(TEAL, BLUE_GREEN, t / 0.45)
            } else {
                mix_color(BLUE_GREEN, DARK_GREEN, (t - 0.45) / 0.55)
            };
            colors.push(col);
        }

        for k in 0..segs {
            let t0 = k as f32 / segs as f32;
            let t1 = (k + 1) as f32 / segs as f32;
            // Cara frontal (+Z)
            let f0 = tail.push_vertex(left_pts[k], z_axis, [t0, 0.0]);
            let f1 = tail.push_vertex(right_pts[k], z_axis, [t0, 1.0]);
            let f2 = tail.push_vertex(right_pts[k + 1], z_axis, [t1, 1.0]);
            let f3 = tail.push_vertex(left_pts[k + 1], z_axis, [t1, 0.0]);
            tail.colors[f0] = colors[k];
            tail.colors[f1] = colors[k];
            tail.colors[f2] = colors[k + 1];
            tail.colors[f3] = colors[k + 1];
            tail.push_quad(f0, f1, f2, f3);

            // Cara trasera (-Z) para visibilidad con backface culling activo.
            let nz = Vector3::new(0.0, 0.0, -1.0);
            let b0 = tail.push_vertex(left_pts[k], nz, [t0, 0.0]);
            let b1 = tail.push_vertex(left_pts[k + 1], nz, [t1, 0.0]);
            let b2 = tail.push_vertex(right_pts[k + 1], nz, [t1, 1.0]);
            let b3 = tail.push_vertex(right_pts[k], nz, [t0, 1.0]);
            tail.colors[b0] = colors[k];
            tail.colors[b1] = colors[k + 1];
            tail.colors[b2] = colors[k + 1];
            tail.colors[b3] = colors[k];
            tail.push_quad(b0, b1, b2, b3);
        }
    }
    tail.set_material(1);
    mesh.merge(&tail);

    // 3. Orbe pequeño (~6% del diámetro de la cabeza -> radio 0.06) cerca de la cola.
    let (orb_rings, orb_secs) = match detail {
        0 => (3, 4),
        1 => (4, 6),
        _ => (6, 8),
    };
    let mut orb = sphere(0.06, orb_rings, orb_secs);
    orb.set_color(BLUE_GREEN);
    orb.set_material(2);
    orb.translate(tail_dir * 2.25 + perp * 0.85);
    mesh.merge(&orb);

    mesh
}

pub fn build_meteor_mesh(detail: usize) -> Mesh {
    build_meteor_mesh_with_seed(detail, 1997)
}

pub fn meteor_bounding_radius(mesh: &Mesh) -> f32 {
    mesh.positions
        .iter()
        .map(|p| p.length())
        .fold(1.0_f32, f32::max)
}

/// Construye el Cubo Skewb de lado 1 con núcleo negro, rombo central y 4 triángulos
/// rectángulos isósceles por cara (encogidos para holgura negra y elevados 0.02).
pub fn build_skewb_mesh(detail: usize) -> Mesh {
    let face_specs: [(Vector3, Vector3, Vector3, Vector3); 6] = [
        (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0), WHITE),
        (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0), YELLOW),
        (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 1.0, 0.0), ORANGE),
        (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0), RED),
        (Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0), GREEN),
        (Vector3::new(0.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), BLUE),
    ];

    if detail == 0 {
        let mut core = box_mesh(1.0, 1.0, 1.0);
        core.paint_vertices(|_, _, n| {
            if n.z > 0.7 {
                WHITE
            } else if n.z < -0.7 {
                YELLOW
            } else if n.x > 0.7 {
                ORANGE
            } else if n.x < -0.7 {
                RED
            } else if n.y > 0.7 {
                GREEN
            } else {
                BLUE
            }
        });
        return core;
    }

    let mut mesh = if detail >= 2 {
        rounded_box(1.0, 1.0, 1.0, 0.04, 1)
    } else {
        box_mesh(1.0, 1.0, 1.0)
    };
    mesh.set_color(BLACK);

    let thickness = if detail >= 2 { 0.016 } else { 0.0 };
    let h = 0.5_f32;
    for (n, u, v, color) in face_specs {
        let center = n * h;
        let c0 = center - u * h - v * h;
        let c1 = center + u * h - v * h;
        let c2 = center + u * h + v * h;
        let c3 = center - u * h + v * h;
        let m0 = center - v * h;
        let m1 = center + u * h;
        let m2 = center + v * h;
        let m3 = center - u * h;

        let diamond = [m0, m1, m2, m3];
        mesh.merge(&raised_sticker(&diamond, 0.84, 0.02, thickness, color));

        for tri in [[c0, m0, m3], [c1, m1, m0], [c2, m2, m1], [c3, m3, m2]] {
            mesh.merge(&raised_sticker(&tri, 0.78, 0.02, thickness, color));
        }
    }
    mesh
}

fn megaminx_face_color(dir: Vector3) -> Vector3 {
    // Paleta de 12 colores con los 5 visibles en refs/megaminx.webp en su posición relativa:
    // blanco (frente-arriba), rojo (abajo-izq), azul (abajo-der), verde (arriba-izq), amarillo (arriba-der).
    let palette: [(Vector3, Vector3); 12] = [
        (Vector3::new(0.0, 0.53, 0.85), WHITE),                              // Frente-arriba: Blanco
        (Vector3::new(-0.85, -0.53, 0.0), RED),                              // Izq-abajo: Rojo
        (Vector3::new(0.85, -0.53, 0.0), BLUE),                              // Der-abajo: Azul
        (Vector3::new(-0.53, 0.85, 0.0), GREEN),                             // Arriba-izq: Verde
        (Vector3::new(0.53, 0.85, 0.0), YELLOW),                             // Arriba-der: Amarillo
        (Vector3::new(0.0, -0.53, 0.85), Vector3::new(0.58, 0.24, 0.78)),    // Morado
        (Vector3::new(0.0, 0.53, -0.85), Vector3::new(0.55, 0.58, 0.62)),    // Gris
        (Vector3::new(0.0, -0.53, -0.85), Vector3::new(0.95, 0.45, 0.72)),   // Rosa
        (Vector3::new(-0.85, 0.53, 0.0), Vector3::new(0.25, 0.82, 0.92)),    // Celeste
        (Vector3::new(0.85, 0.53, 0.0), ORANGE),                             // Naranja
        (Vector3::new(-0.53, -0.85, 0.0), Vector3::new(0.10, 0.42, 0.20)),   // Verde oscuro
        (Vector3::new(0.53, -0.85, 0.0), Vector3::new(0.92, 0.84, 0.65)),    // Crema
    ];
    let n = dir.normalize();
    let mut best_idx = 0;
    let mut best_dot = -2.0_f32;
    for (i, (ref_dir, _)) in palette.iter().enumerate() {
        let d = n.dot(ref_dir.normalize());
        if d > best_dot {
            best_dot = d;
            best_idx = i;
        }
    }
    palette[best_idx].1
}

/// Construye el Megaminx sobre un dodecaedro regular de circunradio 1 con 11
/// pegatinas por cara (pentágono central, 5 cuadriláteros de arista y 5 cometas de esquina).
pub fn build_megaminx_mesh(detail: usize) -> Mesh {
    let faces = dodecahedron_faces(1.0);
    if detail == 0 {
        let mut mesh = Mesh::new();
        for face in faces {
            let center = (face[0] + face[1] + face[2] + face[3] + face[4]) * 0.2;
            let color = megaminx_face_color(center);
            mesh.merge(&flat_polygon_mesh(&face, color));
        }
        return mesh;
    }

    let mut mesh = dodecahedron(0.985);
    mesh.set_color(BLACK);
    let thickness = if detail >= 2 { 0.015 } else { 0.0 };

    for face in faces {
        let c = (face[0] + face[1] + face[2] + face[3] + face[4]) * 0.2;
        let color = megaminx_face_color(c);
        let p: [Vector3; 5] = std::array::from_fn(|i| c + (face[i] - c) * 0.38);
        let ea: [Vector3; 5] =
            std::array::from_fn(|i| face[i] + (face[(i + 1) % 5] - face[i]) * 0.31);
        let eb: [Vector3; 5] =
            std::array::from_fn(|i| face[i] + (face[(i + 1) % 5] - face[i]) * 0.69);

        // 1. Pentágono central
        mesh.merge(&raised_sticker(&p, 0.84, 0.02, thickness, color));

        for i in 0..5 {
            // 2. Cuadrilátero de arista
            let edge_quad = [p[i], ea[i], eb[i], p[(i + 1) % 5]];
            mesh.merge(&raised_sticker(&edge_quad, 0.82, 0.02, thickness, color));

            // 3. Cuadrilátero de esquina en forma de cometa
            let prev = (i + 4) % 5;
            let kite = [p[i], eb[prev], face[i], ea[i]];
            mesh.merge(&raised_sticker(&kite, 0.80, 0.02, thickness, color));
        }
    }
    mesh
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoyConSide {
    Left,
    Right,
}

/// Construye un Joy-Con (`Left` o `Right`) espejado:
/// caja redondeada (alto 2.0, ancho 0.7, grosor 0.35) gris oscuro mate,
/// riel lateral de color (izquierdo `#00C3E3`, derecho `#FF4554`), palanca,
/// cruceta/ABXY, botón `-`/`+`, captura/home y tira SL/SR.
pub fn joycon(side: JoyConSide, detail: usize) -> Mesh {
    let dark_gray = Vector3::new(0.16, 0.17, 0.20);
    let button_dark = Vector3::new(0.07, 0.07, 0.08);
    let button_ring = Vector3::new(0.11, 0.12, 0.14);
    let rail_color = match side {
        JoyConSide::Left => Vector3::new(0.0, 0.765, 0.890),  // #00C3E3
        JoyConSide::Right => Vector3::new(1.000, 0.271, 0.329), // #FF4554
    };
    let rail_x = match side {
        JoyConSide::Left => 0.33,
        JoyConSide::Right => -0.33,
    };
    let plus_minus_x = match side {
        JoyConSide::Left => 0.18,
        JoyConSide::Right => -0.18,
    };

    let seg_cyl = match detail {
        0 => 6,
        1 => 8,
        _ => 12,
    };

    // Carcasa principal
    let mut mesh = match detail {
        0 => box_mesh(0.70, 2.00, 0.35),
        1 => rounded_box(0.70, 2.00, 0.35, 0.13, 1),
        _ => rounded_box(0.70, 2.00, 0.35, 0.13, 2),
    };
    mesh.set_color(dark_gray);

    // Riel lateral de color y botones SL / SR
    let mut rail = box_mesh(0.10, 1.82, 0.26);
    rail.set_color(rail_color);
    rail.translate(Vector3::new(rail_x, 0.0, 0.0));
    mesh.merge(&rail);

    for sy in [0.42_f32, -0.42] {
        let mut sl_sr = box_mesh(0.05, 0.24, 0.12);
        sl_sr.set_color(button_dark);
        sl_sr.translate(Vector3::new(rail_x + rail_x.signum() * 0.05, sy, 0.0));
        mesh.merge(&sl_sr);
    }

    let front_z = 0.175_f32;
    let stick_y = match side {
        JoyConSide::Left => 0.44,
        JoyConSide::Right => -0.16,
    };
    let pad_y = match side {
        JoyConSide::Left => -0.16,
        JoyConSide::Right => 0.44,
    };

    // Palanca: base cilíndrica + tapa cóncava con anillo
    let mut base = cylinder(0.12, 0.14, 0.08, seg_cyl);
    base.rotate_x(FRAC_PI_2);
    base.set_color(button_dark);
    base.translate(Vector3::new(0.0, stick_y, front_z + 0.04));
    mesh.merge(&base);

    let mut cap_ring = cylinder(0.15, 0.12, 0.04, seg_cyl);
    cap_ring.rotate_x(FRAC_PI_2);
    cap_ring.set_color(button_ring);
    cap_ring.translate(Vector3::new(0.0, stick_y, front_z + 0.09));
    mesh.merge(&cap_ring);

    let mut cap_inner = cylinder(0.10, 0.10, 0.02, seg_cyl);
    cap_inner.rotate_x(FRAC_PI_2);
    cap_inner.set_color(button_dark);
    cap_inner.translate(Vector3::new(0.0, stick_y, front_z + 0.105));
    mesh.merge(&cap_inner);

    // 4 botones (cruceta en izquierdo, ABXY en rombo en derecho)
    for (dx, dy) in [(0.0_f32, 0.15_f32), (0.0, -0.15), (-0.15, 0.0), (0.15, 0.0)] {
        let mut btn = cylinder(0.06, 0.06, 0.04, seg_cyl);
        btn.rotate_x(FRAC_PI_2);
        btn.set_color(button_dark);
        btn.translate(Vector3::new(dx, pad_y + dy, front_z + 0.02));
        mesh.merge(&btn);
    }

    // Botón menos (-) o más (+)
    let mut minus_bar = box_mesh(0.11, 0.03, 0.03);
    minus_bar.set_color(button_dark);
    minus_bar.translate(Vector3::new(plus_minus_x, 0.76, front_z + 0.015));
    mesh.merge(&minus_bar);
    if side == JoyConSide::Right {
        let mut plus_bar = box_mesh(0.03, 0.11, 0.03);
        plus_bar.set_color(button_dark);
        plus_bar.translate(Vector3::new(plus_minus_x, 0.76, front_z + 0.015));
        mesh.merge(&plus_bar);
    }

    // Botón de captura (izquierdo, cuadrado) o Home (derecho, circular)
    match side {
        JoyConSide::Left => {
            let mut cap = box_mesh(0.11, 0.11, 0.035);
            cap.set_color(button_dark);
            cap.translate(Vector3::new(0.04, -0.62, front_z + 0.015));
            mesh.merge(&cap);
        }
        JoyConSide::Right => {
            let mut home = cylinder(0.065, 0.065, 0.035, seg_cyl);
            home.rotate_x(FRAC_PI_2);
            home.set_color(button_dark);
            home.translate(Vector3::new(-0.04, -0.62, front_z + 0.015));
            mesh.merge(&home);
        }
    }

    mesh
}

pub fn build_joycon_left_mesh(detail: usize) -> Mesh {
    joycon(JoyConSide::Left, detail)
}

pub fn build_joycon_right_mesh(detail: usize) -> Mesh {
    joycon(JoyConSide::Right, detail)
}

/// Construye el Sol de los Phoenix Suns: explosión solar con estela de rayos
/// extruida desde los contornos medidos en `refs/suns_logo.png` y balón 3D al frente.
pub fn build_suns_mesh(detail: usize) -> Mesh {
    let [gr, gg, gb] = crate::logo_data::SUNS_GOLD_RGB;
    let [or, og, ob] = crate::logo_data::SUNS_ORANGE_RGB;
    let gold = Vector3::new(gr, gg, gb);
    let orange = Vector3::new(or, og, ob);

    let (eps, rings, sectors) = match detail {
        0 => (0.16_f32, 6, 8),
        1 => (0.06_f32, 10, 14),
        _ => (0.0_f32, 14, 20),
    };

    let mut mesh = Mesh::new();

    // 1. Placas emisivas extruidas de la explosión solar y estela de rayos.
    for contour in crate::logo_data::SUNS_CONTOURS {
        let simplified = rdp_simplify_closed(contour, eps);
        if simplified.len() < 3 {
            continue;
        }
        let contour_xz: Vec<[f32; 2]> = simplified.iter().map(|&[x, y]| [x, -y]).collect();
        let mut plate = beveled_extruded_contour(&contour_xz, 0.12, 0.92, 0.025);
        plate.rotate_x(FRAC_PI_2);
        plate.set_color(gold);
        plate.set_material(0);
        mesh.merge(&plate);
    }

    // 2. Balón de basquetbol 3D ligeramente adelantado frente a la placa solar.
    let mut ball = sphere(1.0, rings, sectors);
    ball.translate(Vector3::new(0.0, 0.0, 0.16));
    ball.set_color(orange);
    ball.set_material(1);
    mesh.merge(&ball);

    mesh
}

/// Construye la Cresta del Tottenham: contorno del gallo medido desde
/// `refs/spurs_crest.jpg` extruido con bisel sobre una esfera 3D de balón.
pub fn build_spurs_mesh(detail: usize) -> Mesh {
    let [nr, ng, nb] = crate::logo_data::SPURS_NAVY_RGB;
    let navy = Vector3::new(nr, ng, nb);

    let (eps, rings, sectors) = match detail {
        0 => (0.14_f32, 6, 8),
        1 => (0.05_f32, 10, 14),
        _ => (0.0_f32, 14, 20),
    };

    let mut mesh = Mesh::new();

    // 1. Contorno del gallo extruido con bisel (~0.18 de altura en grosor, bisel ~0.03).
    let simplified = rdp_simplify_closed(crate::logo_data::SPURS_ROOSTER_CONTOUR, eps);
    let contour_xz: Vec<[f32; 2]> = simplified.iter().map(|&[x, y]| [x, -y]).collect();
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for &[_, y] in &simplified {
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let height = (max_y - min_y).max(1.0);
    let thickness = height * 0.18;
    let bevel_h = height * 0.03;

    let mut rooster = beveled_extruded_contour(&contour_xz, thickness, 0.94, bevel_h);
    rooster.rotate_x(FRAC_PI_2);
    rooster.set_color(navy);
    rooster.set_material(0);
    mesh.merge(&rooster);

    // 2. Balón 3D aparte (radio 1.0 en el origen, costuras/cordones en Shader::Spurs).
    let mut ball = sphere(1.0, rings, sectors);
    ball.set_color(navy);
    ball.set_material(1);
    mesh.merge(&ball);

    mesh
}

/// Construye los AirPods: estuche de caja redondeada (proporción ~2.8:2.1:1) con
/// tapa entreabierta ~70°, bisagra cilíndrica, LED frontal y par de auriculares espejados.
pub fn build_airpods_mesh(detail: usize) -> Mesh {
    let white = Vector3::new(0.961, 0.961, 0.969); // #F5F5F7
    let light_gray = Vector3::new(0.84, 0.85, 0.87);
    let silver = Vector3::new(0.72, 0.74, 0.78);
    let mesh_dark = Vector3::new(0.14, 0.15, 0.17);
    let led_green = Vector3::new(0.28, 0.88, 0.42);

    let (seg_box, rings, sectors, seg) = match detail {
        0 => (0, 4, 6, 5),
        1 => (1, 6, 10, 8),
        _ => (2, 10, 14, 12),
    };

    let mut mesh = Mesh::new();

    // 1. Estuche (1.12 x 0.84 x 0.40 -> proporción 2.8 : 2.1 : 1)
    let mut base = if seg_box == 0 {
        box_mesh(1.12, 0.56, 0.40)
    } else {
        rounded_box(1.12, 0.56, 0.40, 0.12, seg_box)
    };
    base.set_color(white);
    base.translate(Vector3::new(0.0, -0.14, 0.0));
    mesh.merge(&base);

    // Tapa girada ~70° (1.22 rad) alrededor de la bisagra trasera
    let mut lid = if seg_box == 0 {
        box_mesh(1.12, 0.28, 0.40)
    } else {
        rounded_box(1.12, 0.28, 0.40, 0.12, seg_box)
    };
    lid.set_color(white);
    lid.translate(Vector3::new(0.0, 0.14, 0.18));
    lid.rotate_x(-1.22);
    lid.translate(Vector3::new(0.0, 0.14, -0.18));
    mesh.merge(&lid);

    // Bisagra cilíndrica y LED frontal
    let mut hinge = cylinder(0.045, 0.045, 0.60, seg);
    hinge.rotate_z(FRAC_PI_2);
    hinge.set_color(silver);
    hinge.translate(Vector3::new(0.0, 0.14, -0.20));
    mesh.merge(&hinge);

    let mut led = sphere(0.024, 4, 6);
    led.set_color(led_green);
    led.translate(Vector3::new(0.0, 0.02, 0.205));
    mesh.merge(&led);

    // 2. Par de auriculares espejados flotando junto al estuche
    for side in [-1.0_f32, 1.0] {
        let mut pod = Mesh::new();
        let mut head = sphere(0.15, rings, sectors);
        head.scale(Vector3::new(1.15, 0.95, 1.15));
        head.set_color(white);
        head.translate(Vector3::new(0.0, 0.24, 0.0));
        pod.merge(&head);

        // Punta de silicona hacia adentro
        let mut tip = cone(0.10, 0.15, seg);
        tip.rotate_z(side * FRAC_PI_2);
        tip.set_color(light_gray);
        tip.translate(Vector3::new(-side * 0.17, 0.22, 0.03));
        pod.merge(&tip);

        // Vástago aplanado
        let mut stem = cylinder(0.052, 0.046, 0.46, seg);
        stem.scale(Vector3::new(0.85, 1.0, 1.12));
        stem.set_color(white);
        stem.translate(Vector3::new(side * 0.03, -0.02, 0.0));
        pod.merge(&stem);

        // Punta metálica inferior del vástago
        let mut cap = cylinder(0.046, 0.042, 0.04, seg);
        cap.set_color(silver);
        cap.translate(Vector3::new(side * 0.03, -0.26, 0.0));
        pod.merge(&cap);

        // Rejilla oscura superior
        let mut grille = box_mesh(0.06, 0.03, 0.09);
        grille.set_color(mesh_dark);
        grille.translate(Vector3::new(0.0, 0.37, 0.03));
        pod.merge(&grille);

        pod.translate(Vector3::new(side * 0.96, 0.08, 0.06));
        mesh.merge(&pod);
    }

    mesh
}

/// Construye la raqueta de tenis con marco ovalado, garganta en Y, mango con
/// bandas de agarre y las cuerdas como cinturón de asteroides en cuadrícula 14×16.
pub fn build_racket_mesh(detail: usize) -> Mesh {
    let frame_black = Vector3::new(0.09, 0.10, 0.13);
    let teal_accent = Vector3::new(0.12, 0.47, 0.62); // refs/raqueta.webp
    let grip_white = Vector3::new(0.93, 0.94, 0.96);
    let grip_dark = Vector3::new(0.18, 0.20, 0.23);
    let asteroid_red = Vector3::new(0.76, 0.22, 0.18);

    let (maj_seg, min_seg, cyl_seg, step_stride) = match detail {
        0 => (12, 4, 5, 4),
        1 => (18, 5, 6, 2),
        _ => (22, 6, 8, 1),
    };

    let mut mesh = Mesh::new();

    // 1. Marco ovalado (anillo) centrado en y = 0.52.
    let oval_cy = 0.52_f32;
    let oval_rx = 0.62_f32;
    let oval_ry = 0.86_f32;
    let mut frame = torus(1.0, 0.055, maj_seg, min_seg);
    frame.rotate_x(FRAC_PI_2);
    frame.scale(Vector3::new(oval_rx, oval_ry, 1.0));
    frame.paint_vertices(|_, p, _| {
        if p.y < -0.15 {
            teal_accent
        } else {
            frame_black
        }
    });
    frame.translate(Vector3::new(0.0, oval_cy, 0.0));
    mesh.merge(&frame);

    // 2. Garganta en Y.
    for side in [-1.0_f32, 1.0] {
        let mut arm = cylinder(0.042, 0.048, 0.56, cyl_seg);
        arm.rotate_z(side * 0.50);
        arm.set_color(teal_accent);
        arm.translate(Vector3::new(side * 0.17, -0.42, 0.0));
        mesh.merge(&arm);
    }

    // 3. Mango con bandas de agarre y tapón.
    for i in 0..6 {
        let mut band = cylinder(0.058, 0.058, 0.12, cyl_seg);
        band.set_color(if i % 2 == 0 { grip_white } else { grip_dark });
        band.translate(Vector3::new(0.0, -0.72 - i as f32 * 0.12, 0.0));
        mesh.merge(&band);
    }
    let mut cap = cylinder(0.060, 0.082, 0.08, cyl_seg);
    cap.set_color(frame_black);
    cap.translate(Vector3::new(0.0, -1.42, 0.0));
    mesh.merge(&cap);

    // 4. Cuerdas como cinturón de asteroides en cuadrícula 14×16 dentro del óvalo.
    let ico = crate::icosphere::icosphere_level(0); // 20 tris por asteroide
    let noise = crate::noise::Noise::new(48);
    let mut proto = Mesh::new();
    for &unit in &ico.positions {
        let h = crate::surface_map::planet_height(&noise, unit, 3.0);
        let r = 0.8 + (h - 0.5) * 0.65;
        proto.push_vertex(unit * r, unit, [0.0, 0.0]);
    }
    proto.indices = ico.indices;
    proto.compute_flat_normals();
    proto.set_color(asteroid_red);

    let mut nodes = Vec::new();
    let cols = 14usize;
    let rows = 16usize;
    let mut idx = 0usize;
    for r in 0..rows {
        let ty = (r as f32 + 0.5) / rows as f32 * 2.0 - 1.0;
        let y = oval_cy + ty * (oval_ry * 0.90);
        for c in 0..cols {
            let tx = (c as f32 + 0.5) / cols as f32 * 2.0 - 1.0;
            let x = tx * (oval_rx * 0.90);
            let norm_d2 = (x / (oval_rx * 0.92)).powi(2) + ((y - oval_cy) / (oval_ry * 0.92)).powi(2);
            if norm_d2 < 0.86 {
                if idx.is_multiple_of(step_stride) {
                    nodes.push(Vector3::new(x, y, 0.0));
                }
                idx += 1;
            }
        }
    }
    scatter_instances_in_mesh(&mut mesh, &proto, &nodes, 0.02, 0.04, 2026);

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

    #[test]
    fn meteor_builds_three_decreasing_lods_under_4000_bounded_and_deterministic() {
        let m2 = build_meteor_mesh(2);
        let m1 = build_meteor_mesh(1);
        let m0 = build_meteor_mesh(0);

        assert!(m2.tri_count() <= 4000, "Meteor L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            let bound = meteor_bounding_radius(m);
            assert!(bound >= 4.0, "La esfera envolvente debe cubrir la cola");
            assert!(m.positions.iter().all(|p| p.length() <= bound + 1e-4));
        }

        // La misma semilla produce exactamente la misma cola y malla.
        let a = build_meteor_mesh_with_seed(2, 1997);
        let b = build_meteor_mesh_with_seed(2, 1997);
        assert_eq!(a.positions.len(), b.positions.len());
        assert_eq!(a.indices, b.indices);
        for (pa, pb) in a.positions.iter().zip(b.positions.iter()) {
            assert!((*pa - *pb).length() < 1e-6);
        }
    }

    #[test]
    fn skewb_builds_three_decreasing_lods_under_4000_tris() {
        let m2 = build_skewb_mesh(2);
        let m1 = build_skewb_mesh(1);
        let m0 = build_skewb_mesh(0);

        assert!(m2.tri_count() <= 4000);
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
        }
    }

    #[test]
    fn megaminx_builds_three_decreasing_lods_under_4000_tris() {
        let m2 = build_megaminx_mesh(2);
        let m1 = build_megaminx_mesh(1);
        let m0 = build_megaminx_mesh(0);

        assert!(m2.tri_count() <= 4000, "Megaminx L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
        }
    }

    #[test]
    fn joycon_left_and_right_build_three_decreasing_lods_under_4000_tris() {
        for builder in [build_joycon_left_mesh, build_joycon_right_mesh] {
            let m2 = builder(2);
            let m1 = builder(1);
            let m0 = builder(0);

            assert!(m2.tri_count() <= 4000);
            assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
            for m in [&m0, &m1, &m2] {
                assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            }
        }
    }

    #[test]
    fn suns_builds_three_decreasing_watertight_lods_under_4000_tris() {
        let m2 = build_suns_mesh(2);
        let m1 = build_suns_mesh(1);
        let m0 = build_suns_mesh(0);
        assert!(m2.tri_count() <= 4000, "Suns L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.is_watertight());
            assert!(m.signed_volume() > 0.0);
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            let br = m.bounding_radius();
            assert!(br > 3.0 && br < 3.5);
            assert!(m.positions.iter().all(|p| p.length() <= br + 1e-4));
        }
    }

    #[test]
    fn spurs_builds_three_decreasing_lods_under_4000_tris_and_bounded() {
        let m2 = build_spurs_mesh(2);
        let m1 = build_spurs_mesh(1);
        let m0 = build_spurs_mesh(0);
        assert!(m2.tri_count() <= 4000, "Spurs L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.is_watertight());
            assert!(m.signed_volume() > 0.0);
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            let br = m.bounding_radius();
            assert!(m.positions.iter().all(|p| p.length() <= br + 1e-4));
        }
    }

    #[test]
    fn logo_data_contours_have_no_consecutive_duplicates_or_degeneracy() {
        let check_poly = |poly: &[[f32; 2]], name: &str| {
            assert!(poly.len() >= 3, "{name} degenerado: {}", poly.len());
            for w in poly.windows(2) {
                let d = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
                assert!(d > 1e-5, "{name} tiene duplicado consecutivo");
            }
            let first = poly[0];
            let last = poly[poly.len() - 1];
            let wrap_d = ((first[0] - last[0]).powi(2) + (first[1] - last[1]).powi(2)).sqrt();
            assert!(wrap_d > 1e-5, "{name} tiene cierre duplicado");
        };
        check_poly(crate::logo_data::SPURS_ROOSTER_CONTOUR, "SPURS_ROOSTER_CONTOUR");
        for (i, poly) in crate::logo_data::SUNS_CONTOURS.iter().enumerate() {
            check_poly(poly, &format!("SUNS_RAY_{i}"));
        }
    }

    #[test]
    fn airpods_build_three_decreasing_lods_under_4000_tris_and_bounded() {
        let m2 = build_airpods_mesh(2);
        let m1 = build_airpods_mesh(1);
        let m0 = build_airpods_mesh(0);
        assert!(m2.tri_count() <= 4000, "AirPods L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            let br = m.bounding_radius();
            assert!(m.positions.iter().all(|p| p.length() <= br + 1e-4));
        }
    }

    #[test]
    fn racket_builds_three_decreasing_lods_under_4000_tris_and_bounded() {
        let m2 = build_racket_mesh(2);
        let m1 = build_racket_mesh(1);
        let m0 = build_racket_mesh(0);
        assert!(m2.tri_count() <= 4000, "Racket L2={} > 4000", m2.tri_count());
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            let br = m.bounding_radius();
            assert!(m.positions.iter().all(|p| p.length() <= br + 1e-4));
        }
    }
}
