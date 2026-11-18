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

#[cfg(test)]
mod tests {
    use super::SimClock;
    use crate::obj::load_obj;
    use crate::scene::{body_model_matrix_at, body_position_at, solar_system};

    #[test]
    fn positions_and_rotations_depend_on_time_not_frame_count() {
        let mut clock_30 = SimClock::new();
        for _ in 0..30 {
            clock_30.advance(1.0 / 30.0);
        }

        let mut clock_144 = SimClock::new();
        for _ in 0..144 {
            clock_144.advance(1.0 / 144.0);
        }

        let bodies = solar_system();
        let sphere = load_obj("assets/sphere.obj").expect("sphere.obj debe cargar");

        for body in &bodies {
            let p30 = body_position_at(body, clock_30.time);
            let p144 = body_position_at(body, clock_144.time);
            assert!(
                (p30 - p144).length() < 1e-4,
                "{}: p30={p30:?} vs p144={p144:?}",
                body.name
            );

            let m30 = body_model_matrix_at(&sphere, body, clock_30.time);
            let m144 = body_model_matrix_at(&sphere, body, clock_144.time);
            for col in 0..4 {
                for row in 0..4 {
                    assert!(
                        (m30.cols[col][row] - m144.cols[col][row]).abs() < 1e-3,
                        "{}: matriz difiere en [{col}][{row}]",
                        body.name
                    );
                }
            }
        }
    }
}
