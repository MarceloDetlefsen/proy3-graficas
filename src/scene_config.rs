#![allow(dead_code)]
//! Configuración centralizada de la escena: escalas, órbitas calibradas para
//! un recorrido de ~1 minuto, atenuación solar, halo del sol y borde atmosférico.

use raylib::prelude::Vector3;

use crate::shaders::Shader;

/// Factor de escala orbital para que el recorrido completo dure ~60 s de vuelo.
pub const ORBIT_DISTANCE_SCALE: f32 = 0.65;
/// Escala global de velocidades orbitales.
pub const ORBIT_SPEED_SCALE: f32 = 1.15;

/// Iluminación solar con atenuación por distancia y ambiente mínimo configurable.
pub const MIN_AMBIENT: f32 = 0.14;
pub const SOLAR_ATTENUATION_K: f32 = 0.00055;
pub const SOLAR_HALO_RADIUS_MULT: f32 = 2.4;
pub const SOLAR_HALO_COLOR: Vector3 = Vector3 { x: 1.0, y: 0.58, z: 0.78 };

#[derive(Clone, Copy, Debug)]
pub struct AtmosphereRim {
    pub color: Vector3,
    pub power: f32,
    pub strength: f32,
}

pub fn solar_attenuation(distance: f32) -> f32 {
    (1.0 / (1.0 + SOLAR_ATTENUATION_K * distance * distance)).clamp(MIN_AMBIENT, 1.0)
}

pub fn atmosphere_rim(shader: Shader) -> Option<AtmosphereRim> {
    match shader {
        Shader::Rocky => Some(AtmosphereRim {
            color: Vector3::new(0.28, 0.72, 0.92),
            power: 3.2,
            strength: 0.34,
        }),
        Shader::GasGiant => Some(AtmosphereRim {
            color: Vector3::new(0.45, 0.88, 0.62),
            power: 2.8,
            strength: 0.30,
        }),
        Shader::Ardens => Some(AtmosphereRim {
            color: Vector3::new(1.0, 0.38, 0.18),
            power: 3.0,
            strength: 0.36,
        }),
        Shader::Abismo => Some(AtmosphereRim {
            color: Vector3::new(0.32, 0.84, 0.88),
            power: 3.0,
            strength: 0.38,
        }),
        Shader::Vestigio => Some(AtmosphereRim {
            color: Vector3::new(0.85, 0.68, 0.56),
            power: 3.4,
            strength: 0.22,
        }),
        Shader::Mural => Some(AtmosphereRim {
            color: Vector3::new(0.62, 0.68, 0.75),
            power: 3.8,
            strength: 0.16,
        }),
        Shader::Fundido => Some(AtmosphereRim {
            color: Vector3::new(0.95, 0.42, 0.14),
            power: 2.9,
            strength: 0.34,
        }),
        Shader::Basquia => Some(AtmosphereRim {
            color: Vector3::new(0.92, 0.82, 0.45),
            power: 4.0,
            strength: 0.14,
        }),
        Shader::Star
        | Shader::Chunky
        | Shader::Triforce
        | Shader::Meteor
        | Shader::Ship
        | Shader::Prop(_) => None,
    }
}
