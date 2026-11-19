#![allow(dead_code)]
//! Cuerpos procedurales del sistema de favoritos construidos con `mesh_builder`.

use std::f32::consts::FRAC_PI_2;

use raylib::prelude::Vector3;

use crate::mesh_builder::{
    Mesh, beveled_extruded_contour, box_mesh, cone, cylinder, dodecahedron, dodecahedron_faces,
    extruded_prism, flat_polygon_mesh, loft_rings, raised_sticker, rounded_box, sphere, torus,
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

/// Construye la esfera del Sol de los Phoenix Suns en 3 niveles de detalle.
pub fn build_suns_mesh(detail: usize) -> Mesh {
    let ico_lvl = match detail {
        0 => 1, // 80 tris
        1 => 2, // 320 tris
        _ => 3, // 1280 tris
    };
    let ico = crate::icosphere::icosphere_level(ico_lvl);
    let mut mesh = Mesh::new();
    for &unit in &ico.positions {
        mesh.push_vertex(unit, unit, crate::icosphere::spherical_uv(unit));
    }
    mesh.indices = ico.indices;
    mesh
}

/// Construye el Escudo del Tottenham Hotspur con placa extruida con bisel,
/// barras de cinta, balón con parches y gallo estilizado armado con primitivas.
pub fn build_spurs_mesh(detail: usize) -> Mesh {
    let navy = Vector3::new(0.075, 0.133, 0.341); // #132257
    let field_white = Vector3::new(0.96, 0.97, 0.99);

    let (rings, sectors, seg) = match detail {
        0 => (4, 6, 5),
        1 => (6, 10, 8),
        _ => (10, 16, 12),
    };

    let mut mesh = Mesh::new();

    // 1. Placa de escudo extruida con bisel (borde azul marino).
    let shield_2d: [[f32; 2]; 7] = [
        [-0.78, -0.82],
        [0.0, -0.92],
        [0.78, -0.82],
        [0.74, 0.18],
        [0.44, 0.68],
        [0.0, 0.94],
        [-0.44, 0.68],
    ];
    let mut shield_2d_full = shield_2d.to_vec();
    shield_2d_full.push([-0.74, 0.18]);
    let mut plaque = beveled_extruded_contour(&shield_2d_full, 0.18, 0.93, 0.035);
    plaque.rotate_x(FRAC_PI_2);
    plaque.set_color(navy);
    mesh.merge(&plaque);

    // Campo blanco hundido en el frente (+Z).
    let mut field = beveled_extruded_contour(&shield_2d_full, 0.04, 0.85, 0.01);
    field.rotate_x(FRAC_PI_2);
    field.scale(Vector3::new(0.86, 0.86, 1.0));
    field.set_color(field_white);
    field.translate(Vector3::new(0.0, 0.0, 0.085));
    mesh.merge(&field);

    // Barras de cinta inferiores (sin texto 3D).
    for (by, bw) in [(-0.56_f32, 0.96_f32), (-0.70, 0.72)] {
        let mut bar = box_mesh(bw, 0.08, 0.04);
        bar.set_color(navy);
        bar.translate(Vector3::new(0.0, by, 0.11));
        mesh.merge(&bar);
    }

    // 2. Balón clásico con parches/costuras en la parte inferior.
    let ball_pos = Vector3::new(0.0, -0.24, 0.22);
    let mut ball = sphere(0.22, rings, sectors);
    ball.set_color(navy);
    ball.translate(ball_pos);
    mesh.merge(&ball);

    let mut seam_ring = torus(0.215, 0.014, sectors, 6);
    seam_ring.set_color(field_white);
    seam_ring.translate(ball_pos);
    mesh.merge(&seam_ring);

    let mut patch = box_mesh(0.11, 0.06, 0.04);
    patch.set_color(field_white);
    patch.translate(ball_pos + Vector3::new(0.06, -0.02, 0.20));
    mesh.merge(&patch);

    // 3. Gallo estilizado armado con primitivas posado sobre el balón.
    let mut body = sphere(0.16, rings, sectors);
    body.scale(Vector3::new(0.85, 1.25, 0.65));
    body.rotate_z(0.32);
    body.set_color(navy);
    body.translate(Vector3::new(0.02, 0.26, 0.22));
    mesh.merge(&body);

    let mut neck = cylinder(0.065, 0.095, 0.24, seg);
    neck.rotate_z(0.15);
    neck.set_color(navy);
    neck.translate(Vector3::new(-0.07, 0.48, 0.22));
    mesh.merge(&neck);

    let mut collar = sphere(0.068, rings, sectors);
    collar.scale(Vector3::new(0.7, 1.4, 0.6));
    collar.set_color(field_white);
    collar.translate(Vector3::new(-0.05, 0.48, 0.25));
    mesh.merge(&collar);

    let mut head = sphere(0.085, rings, sectors);
    head.set_color(navy);
    head.translate(Vector3::new(-0.10, 0.62, 0.22));
    mesh.merge(&head);

    let mut beak = cone(0.038, 0.12, seg);
    beak.rotate_z(FRAC_PI_2 + 0.15);
    beak.set_color(navy);
    beak.translate(Vector3::new(-0.20, 0.61, 0.22));
    mesh.merge(&beak);

    let mut crest = box_mesh(0.08, 0.06, 0.05);
    crest.set_color(navy);
    crest.translate(Vector3::new(-0.08, 0.70, 0.22));
    mesh.merge(&crest);

    // Cola de 3 plumas curvas hacia arriba-derecha.
    for (ang, dx, dy) in [(-0.45_f32, 0.18, 0.34), (-0.75, 0.22, 0.28), (-1.05, 0.24, 0.21)] {
        let mut feather = box_mesh(0.07, 0.24, 0.05);
        feather.rotate_z(ang);
        feather.set_color(navy);
        feather.translate(Vector3::new(dx, dy, 0.22));
        mesh.merge(&feather);
    }

    // Pata con espolón hacia atrás.
    let mut leg = cylinder(0.032, 0.042, 0.24, seg);
    leg.set_color(navy);
    leg.translate(Vector3::new(0.02, 0.05, 0.22));
    mesh.merge(&leg);

    let mut spur = cone(0.025, 0.10, seg);
    spur.rotate_z(-FRAC_PI_2 + 0.25);
    spur.set_color(navy);
    spur.translate(Vector3::new(0.09, 0.03, 0.22));
    mesh.merge(&spur);

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
        assert!(m2.tri_count() <= 4000);
        assert!(m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count());
        for m in [&m0, &m1, &m2] {
            assert!(m.is_watertight());
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            assert!((m.bounding_radius() - 1.0).abs() < 1e-4);
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
            assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
            let br = m.bounding_radius();
            assert!(m.positions.iter().all(|p| p.length() <= br + 1e-4));
        }
    }
}
