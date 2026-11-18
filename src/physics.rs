#![allow(dead_code)]
//! Paso de tiempo fijo, gravedad y colisiones del sistema solar.
//!
//! Implementa un acumulador de paso fijo (`FIXED_DT = 1/120 s`, máximo 8
//! subpasos por cuadro) con integración semi-implícita de Euler para que la
//! trayectoria de la nave sea determinista e independiente de los FPS.
//! Los cuerpos celestes siguen órbitas analíticas cinemáticas.

use raylib::prelude::Vector3;

use crate::scene::{body_position_at, Body};

pub const FIXED_DT: f32 = 1.0 / 120.0;
pub const MAX_SUBSTEPS: usize = 8;
/// Constante gravitatoria calibrada para que a ~3 radios de un planeta la
/// aceleración sea aproximadamente la mitad del empuje máximo de la nave.
pub const GRAVITY_G: f32 = 7.2;
pub const GRAVITY_SOFTENING_FACTOR: f32 = 0.5;
pub const MAX_GRAVITY_ACCEL: f32 = 15.0;

#[derive(Clone, Copy, Debug)]
pub struct PhysicsStepper {
    pub accumulator: f32,
}

impl PhysicsStepper {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    pub fn step<F>(&mut self, frame_dt: f32, mut on_substep: F) -> usize
    where
        F: FnMut(f32),
    {
        self.accumulator += frame_dt.clamp(0.0, 0.25);
        let mut steps = 0usize;
        while self.accumulator >= FIXED_DT && steps < MAX_SUBSTEPS {
            on_substep(FIXED_DT);
            self.accumulator -= FIXED_DT;
            steps += 1;
        }
        if steps == MAX_SUBSTEPS && self.accumulator > FIXED_DT {
            self.accumulator = 0.0;
        }
        steps
    }
}

/// Calcula la aceleración gravitatoria neta sobre `pos` debida a todos los
/// cuerpos en el instante `time`, usando ablandamiento proporcional al radio.
pub fn gravity_acceleration(pos: Vector3, bodies: &[Body], time: f32) -> Vector3 {
    let mut total = Vector3::zero();
    for body in bodies {
        let delta = body_position_at(body, time) - pos;
        let dist_sq = delta.length_sqr();
        let softening = body.radius * GRAVITY_SOFTENING_FACTOR;
        let denom_sq = dist_sq + softening * softening;
        if dist_sq <= 1e-10 || denom_sq <= 1e-10 {
            continue;
        }
        let dist = dist_sq.sqrt();
        let mag = GRAVITY_G * body.mass / denom_sq;
        total += delta * (mag / dist);
    }
    total
}

/// Integra la aceleración gravitatoria (con tope `MAX_GRAVITY_ACCEL`) sobre la
/// velocidad de la nave y avanza su pose con Euler semi-implícito.
/// Devuelve la magnitud efectiva de gravedad aplicada.
pub fn integrate_ship_with_gravity(
    ship: &mut crate::ship::Ship,
    bodies: &[Body],
    time: f32,
    dt: f32,
    gravity_enabled: bool,
) -> f32 {
    let mut mag = 0.0;
    if gravity_enabled && dt > 0.0 {
        let mut g = gravity_acceleration(ship.position, bodies, time);
        mag = g.length();
        if mag > MAX_GRAVITY_ACCEL {
            g *= MAX_GRAVITY_ACCEL / mag;
            mag = MAX_GRAVITY_ACCEL;
        }
        ship.velocity += g * dt;
    }
    ship.integrate_pose(dt);
    mag
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::Vector3;

    fn simulate_trajectory(fps: usize) -> (Vector3, Vector3, usize) {
        let mut stepper = PhysicsStepper::new();
        let mut pos = Vector3::zero();
        let mut vel = Vector3::zero();
        let accel = Vector3::new(3.5, -1.2, -8.0);
        let frame_dt = 1.0 / fps as f32 + 1e-6;
        let mut total_steps = 0usize;

        for _ in 0..fps {
            total_steps += stepper.step(frame_dt, |dt| {
                vel += accel * dt;
                pos += vel * dt;
            });
        }
        (pos, vel, total_steps)
    }

    #[test]
    fn same_input_produces_same_trajectory_at_30_60_and_144_fps() {
        let (p30, v30, s30) = simulate_trajectory(30);
        let (p60, v60, s60) = simulate_trajectory(60);
        let (p144, v144, s144) = simulate_trajectory(144);

        assert_eq!(s30, 120);
        assert_eq!(s60, 120);
        assert_eq!(s144, 120);
        assert!((p30 - p60).length() < 1e-5);
        assert!((p30 - p144).length() < 1e-5);
        assert!((v30 - v60).length() < 1e-5);
        assert!((v30 - v144).length() < 1e-5);
    }

    #[test]
    fn gravity_follows_inverse_square_and_directional_symmetry() {
        let mut planet = crate::scene::solar_system().remove(1);
        planet.orbit_radius = 0.0;
        planet.radius = 0.01;
        let bodies = [planet];

        let g_near = gravity_acceleration(Vector3::new(10.0, 0.0, 0.0), &bodies, 0.0);
        let g_far = gravity_acceleration(Vector3::new(20.0, 0.0, 0.0), &bodies, 0.0);
        let ratio = g_near.length() / g_far.length();
        assert!((ratio - 4.0).abs() < 1e-3, "ratio esperado ~4.0, obtenido {ratio}");

        let p = Vector3::new(12.0, -5.0, 8.0);
        let g_pos = gravity_acceleration(p, &bodies, 0.0);
        let g_neg = gravity_acceleration(-p, &bodies, 0.0);
        assert!((g_pos + g_neg).length() < 1e-5);

        let g_y = gravity_acceleration(Vector3::new(0.0, 10.0, 0.0), &bodies, 0.0);
        assert!((g_y.length() - g_near.length()).abs() < 1e-5);
    }
}
