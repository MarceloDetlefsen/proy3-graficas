#![allow(dead_code)]
//! Generación procedural de icosferas por subdivisión de un icosaedro base.
//!
//! Construye un icosaedro unitario con 12 vértices y 20 triángulos orientados
//! hacia afuera (CCW), y lo subdivide recursivamente usando una caché de puntos
//! medios por arista proyectados a la esfera unitaria para niveles 0..=5.

use raylib::prelude::Vector3;

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
