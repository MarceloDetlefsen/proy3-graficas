#![allow(dead_code)]
//! Paso de tiempo fijo, gravedad y colisiones del sistema solar.
//!
//! Implementa un acumulador de paso fijo (`FIXED_DT = 1/120 s`, máximo 8
//! subpasos por cuadro) con integración semi-implícita de Euler para que la
//! trayectoria de la nave sea determinista e independiente de los FPS.
//! Los cuerpos celestes siguen órbitas analíticas cinemáticas.

pub const FIXED_DT: f32 = 1.0 / 120.0;
pub const MAX_SUBSTEPS: usize = 8;

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
}
