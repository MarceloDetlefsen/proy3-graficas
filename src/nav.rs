#![allow(dead_code)]
//! Navegación contextual (espacio profundo vs. órbita local de planeta).
//!
//! Detecta el cuerpo celeste más cercano, administra la transición con histéresis
//! entre marco global y marco local en movimiento junto al planeta, alinea
//! suavemente la vertical de la nave con la normal local y escala la velocidad
//! y sensibilidad de giro según la altitud sobre la superficie.

use raylib::prelude::Vector3;

use crate::scene::{body_position_at, body_velocity_at, effective_radius, Body};
use crate::ship::Ship;

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

/// Posición de la nave expresada en el marco activo (`Global` o relativo al cuerpo `Local`).
pub fn frame_position(pos: Vector3, mode: NavMode, bodies: &[Body], time: f32) -> Vector3 {
    match mode {
        NavMode::Global => pos,
        NavMode::Local(idx) => bodies
            .get(idx)
            .map_or(pos, |b| pos - body_position_at(b, time)),
    }
}

/// Velocidad de la nave expresada en el marco activo (`Global` o relativa a la órbita del cuerpo).
pub fn frame_velocity(vel: Vector3, mode: NavMode, bodies: &[Body], time: f32) -> Vector3 {
    match mode {
        NavMode::Global => vel,
        NavMode::Local(idx) => bodies
            .get(idx)
            .map_or(vel, |b| vel - body_velocity_at(b, time)),
    }
}

/// Convierte una posición y velocidad desde el marco activo de vuelta a espacio mundial.
pub fn to_world_state(
    local_pos: Vector3,
    local_vel: Vector3,
    mode: NavMode,
    bodies: &[Body],
    time: f32,
) -> (Vector3, Vector3) {
    match mode {
        NavMode::Global => (local_pos, local_vel),
        NavMode::Local(idx) => match bodies.get(idx) {
            Some(b) => (
                local_pos + body_position_at(b, time),
                local_vel + body_velocity_at(b, time),
            ),
            None => (local_pos, local_vel),
        },
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlightControls {
    pub throttle: f32,
    pub boost: bool,
    pub braking: bool,
}

/// Aplica empuje y frenado en el marco de referencia activo (`Global` o `Local`),
/// de modo que frenar en modo local detiene la nave respecto al planeta cercano.
pub fn apply_controls_in_frame(
    ship: &mut Ship,
    mode: NavMode,
    bodies: &[Body],
    time: f32,
    controls: FlightControls,
    dt: f32,
) {
    let ref_vel = match mode {
        NavMode::Global => Vector3::zero(),
        NavMode::Local(idx) => bodies
            .get(idx)
            .map_or(Vector3::zero(), |b| body_velocity_at(b, time)),
    };
    ship.velocity -= ref_vel;
    ship.apply_thrust(controls.throttle, controls.boost, dt);
    ship.apply_damping(controls.braking, dt);
    ship.velocity += ref_vel;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::solar_system;

    #[test]
    fn switching_frames_produces_zero_position_or_velocity_jump() {
        let bodies = solar_system();
        let zebes = &bodies[1];
        let time = 7.5_f32;
        let planet_pos = body_position_at(zebes, time);
        let r = effective_radius(zebes);

        // Posición justo dentro del umbral de entrada (< 2.5 * r de altitud).
        let pos_in = planet_pos + Vector3::new(0.0, 0.0, r + 2.0 * r);
        let vel_world = Vector3::new(3.2, -1.4, 2.1);

        let mut nav = NavState::new();
        assert_eq!(nav.mode, NavMode::Global);

        nav.update_mode(pos_in, &bodies, time);
        assert_eq!(nav.mode, NavMode::Local(1));

        let local_p = frame_position(pos_in, nav.mode, &bodies, time);
        let local_v = frame_velocity(vel_world, nav.mode, &bodies, time);
        let (round_p, round_v) = to_world_state(local_p, local_v, nav.mode, &bodies, time);
        assert!((round_p - pos_in).length() < 1e-5);
        assert!((round_v - vel_world).length() < 1e-5);

        // En la banda de histéresis (2.75 * r) sigue en modo local.
        let pos_hyst = planet_pos + Vector3::new(0.0, 0.0, r + 2.75 * r);
        nav.update_mode(pos_hyst, &bodies, time);
        assert_eq!(nav.mode, NavMode::Local(1));

        // Al salir (> 3.0 * r) regresa a Global sin alterar el estado mundial.
        let pos_out = planet_pos + Vector3::new(0.0, 0.0, r + 3.2 * r);
        nav.update_mode(pos_out, &bodies, time);
        assert_eq!(nav.mode, NavMode::Global);

        let (global_p, global_v) = to_world_state(pos_out, vel_world, nav.mode, &bodies, time);
        assert!((global_p - pos_out).length() < 1e-6);
        assert!((global_v - vel_world).length() < 1e-6);
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
