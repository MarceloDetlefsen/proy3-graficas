#![allow(dead_code)]
//! Reloj de simulación independiente del tiempo real y de la tasa de cuadros.
//!
//! Mantiene el tiempo acumulado de simulación, una escala multiplicadora
//! (por ejemplo 0.25×, 1×, 2×, 5×) y un estado de pausa. Todo movimiento de
//! órbitas, rotaciones y física avanza usando `dt * scale` cuando no está
//! en pausa, garantizando comportamiento idéntico a cualquier FPS.

#[derive(Clone, Copy, Debug)]
pub struct SimClock {
    pub time: f32,
    pub scale: f32,
    pub paused: bool,
}

impl SimClock {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            scale: 1.0,
            paused: false,
        }
    }

    pub fn advance(&mut self, real_dt: f32) -> f32 {
        if self.paused {
            return 0.0;
        }
        let sim_dt = real_dt.max(0.0) * self.scale;
        self.time += sim_dt;
        sim_dt
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.clamp(0.0, 64.0);
    }
}
