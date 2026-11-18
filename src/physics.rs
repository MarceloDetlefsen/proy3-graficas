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
