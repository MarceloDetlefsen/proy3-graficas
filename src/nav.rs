#![allow(dead_code)]
//! Navegación contextual (espacio profundo vs. órbita local de planeta).
//!
//! Detecta el cuerpo celeste más cercano, administra la transición con histéresis
//! entre marco global y marco local en movimiento junto al planeta, alinea
//! suavemente la vertical de la nave con la normal local y escala la velocidad
//! y sensibilidad de giro según la altitud sobre la superficie.

use raylib::prelude::Vector3;

use crate::scene::{body_position_at, effective_radius, Body};

#[derive(Clone, Copy, Debug)]
pub struct NearestBody {
    pub body_index: usize,
    pub altitude: f32,
    pub distance: f32,
    pub normal: Vector3,
}

/// Encuentra el cuerpo celeste cuya superficie está más cerca de `pos` en `time`.
pub fn nearest_body(pos: Vector3, bodies: &[Body], time: f32) -> Option<NearestBody> {
    let mut best: Option<NearestBody> = None;
    for (idx, body) in bodies.iter().enumerate() {
        let center = body_position_at(body, time);
        let diff = pos - center;
        let distance = diff.length();
        let altitude = distance - effective_radius(body);
        let normal = if distance > 1e-6 {
            diff * (1.0 / distance)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };
        if best.is_none_or(|b| altitude < b.altitude) {
            best = Some(NearestBody {
                body_index: idx,
                altitude,
                distance,
                normal,
            });
        }
    }
    best
}
