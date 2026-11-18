#![allow(dead_code)]
//! Navegación contextual (espacio profundo vs. órbita local de planeta).
//!
//! Detecta el cuerpo celeste más cercano, administra la transición con histéresis
//! entre marco global y marco local en movimiento junto al planeta, alinea
//! suavemente la vertical de la nave con la normal local y escala la velocidad
//! y sensibilidad de giro según la altitud sobre la superficie.

use raylib::prelude::Vector3;

use crate::scene::{body_position_at, effective_radius, Body};

pub const LOCAL_ENTER_RADII: f32 = 2.5;
pub const LOCAL_EXIT_RADII: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavMode {
    Global,
    Local(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavOverride {
    Auto,
    ForceLocal,
    ForceGlobal,
}

#[derive(Clone, Copy, Debug)]
pub struct NavState {
    pub mode: NavMode,
    pub override_mode: NavOverride,
    pub nearest: Option<NearestBody>,
}

impl NavState {
    pub fn new() -> Self {
        Self {
            mode: NavMode::Global,
            override_mode: NavOverride::Auto,
            nearest: None,
        }
    }

    pub fn cycle_override(&mut self) {
        self.override_mode = match self.override_mode {
            NavOverride::Auto => NavOverride::ForceLocal,
            NavOverride::ForceLocal => NavOverride::ForceGlobal,
            NavOverride::ForceGlobal => NavOverride::Auto,
        };
    }

    /// Actualiza el modo (Global vs Local) aplicando histéresis de altitud.
    pub fn update_mode(&mut self, pos: Vector3, bodies: &[Body], time: f32) {
        self.nearest = nearest_body(pos, bodies, time);
        match self.override_mode {
            NavOverride::ForceGlobal => {
                self.mode = NavMode::Global;
            }
            NavOverride::ForceLocal => {
                self.mode = match self.nearest {
                    Some(n) => NavMode::Local(n.body_index),
                    None => NavMode::Global,
                };
            }
            NavOverride::Auto => match self.mode {
                NavMode::Global => {
                    if let Some(n) = self.nearest {
                        let threshold = LOCAL_ENTER_RADII * effective_radius(&bodies[n.body_index]);
                        if n.altitude < threshold {
                            self.mode = NavMode::Local(n.body_index);
                        }
                    }
                }
                NavMode::Local(idx) => {
                    if let Some(body) = bodies.get(idx) {
                        let dist = (pos - body_position_at(body, time)).length();
                        let alt = dist - effective_radius(body);
                        let exit_threshold = LOCAL_EXIT_RADII * effective_radius(body);
                        if alt > exit_threshold {
                            self.mode = NavMode::Global;
                        }
                    } else {
                        self.mode = NavMode::Global;
                    }
                }
            },
        }
    }
}

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
