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
        | Shader::PhoenixSuns
        | Shader::Spurs
        | Shader::Chunky
        | Shader::Triforce
        | Shader::Meteor
        | Shader::Ship
        | Shader::Prop(_) => None,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FavoritesBodySpec {
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_period: f32,
    pub phase: f32,
    pub orbit_inclination: f32,
    pub rotation_period: f32,
    pub axial_tilt: f32,
    pub bounding_mult: f32,
}

/// Parámetros orbitales, escalas, rotaciones y radios envolventes del sistema de favoritos.
pub fn favorites_body_spec(id: &str) -> FavoritesBodySpec {
    match id {
        "phoenix_suns" => FavoritesBodySpec {
            radius: 2.6,
            orbit_radius: 0.0,
            orbit_period: 0.0,
            phase: 0.0,
            orbit_inclination: 0.0,
            rotation_period: 0.0,
            axial_tilt: 0.0,
            bounding_mult: 3.4,
        },
        "cubo_rubik" => FavoritesBodySpec {
            radius: 1.05,
            orbit_radius: 14.5,
            orbit_period: 26.0,
            phase: 0.4,
            orbit_inclination: 0.03,
            rotation_period: 14.0,
            axial_tilt: 0.22,
            bounding_mult: 1.65,
        },
        "cubo_skweb" => FavoritesBodySpec {
            radius: 1.00,
            orbit_radius: 20.5,
            orbit_period: 34.0,
            phase: 1.5,
            orbit_inclination: -0.04,
            rotation_period: 16.0,
            axial_tilt: 0.20,
            bounding_mult: 1.05,
        },
        "trifuerza" => FavoritesBodySpec {
            radius: 1.15,
            orbit_radius: 26.5,
            orbit_period: 42.0,
            phase: 2.6,
            orbit_inclination: 0.05,
            rotation_period: 18.0,
            axial_tilt: 0.14,
            bounding_mult: 1.25,
        },
        "joycon_izquierdo" => FavoritesBodySpec {
            radius: 1.10,
            orbit_radius: 32.5,
            orbit_period: 50.0,
            phase: 3.7,
            orbit_inclination: -0.03,
            rotation_period: 20.0,
            axial_tilt: 0.18,
            bounding_mult: 1.40,
        },
        "joycon_derecho" => FavoritesBodySpec {
            radius: 1.10,
            orbit_radius: 38.5,
            orbit_period: 58.0,
            phase: 4.8,
            orbit_inclination: 0.04,
            rotation_period: 21.0,
            axial_tilt: -0.18,
            bounding_mult: 1.40,
        },
        "airpods" => FavoritesBodySpec {
            radius: 1.25,
            orbit_radius: 45.0,
            orbit_period: 66.0,
            phase: 5.7,
            orbit_inclination: -0.05,
            rotation_period: 22.0,
            axial_tilt: 0.16,
            bounding_mult: 1.65,
        },
        "casco_samus" => FavoritesBodySpec {
            radius: 1.30,
            orbit_radius: 52.0,
            orbit_period: 75.0,
            phase: 0.9,
            orbit_inclination: 0.04,
            rotation_period: 24.0,
            axial_tilt: 0.08,
            bounding_mult: 1.25,
        },
        "megaminx" => FavoritesBodySpec {
            radius: 1.35,
            orbit_radius: 59.5,
            orbit_period: 84.0,
            phase: 2.1,
            orbit_inclination: -0.04,
            rotation_period: 26.0,
            axial_tilt: 0.24,
            bounding_mult: 1.15,
        },
        "raqueta_tenis" => FavoritesBodySpec {
            radius: 1.50,
            orbit_radius: 67.5,
            orbit_period: 94.0,
            phase: 3.3,
            orbit_inclination: 0.05,
            rotation_period: 16.0,
            axial_tilt: 0.0,
            bounding_mult: 1.05,
        },
        "la_bombonera" => FavoritesBodySpec {
            radius: 1.85,
            orbit_radius: 76.5,
            orbit_period: 106.0,
            phase: 4.5,
            orbit_inclination: -0.02,
            rotation_period: 30.0,
            axial_tilt: 0.10,
            bounding_mult: 1.25,
        },
        "spurs_crest" => FavoritesBodySpec {
            radius: 1.10,
            orbit_radius: 88.0,
            orbit_period: 120.0,
            phase: 5.5,
            orbit_inclination: 0.03,
            rotation_period: 28.0,
            axial_tilt: 0.0,
            bounding_mult: 5.4,
        },
        "meteoro_ff7" => FavoritesBodySpec {
            radius: 1.20,
            orbit_radius: 104.0,
            orbit_period: 138.0,
            phase: 1.2,
            orbit_inclination: -0.06,
            rotation_period: 32.0,
            axial_tilt: 0.0,
            bounding_mult: 5.6,
        },
        _ => FavoritesBodySpec {
            radius: 1.2,
            orbit_radius: 120.0,
            orbit_period: 150.0,
            phase: 0.0,
            orbit_inclination: 0.0,
            rotation_period: 24.0,
            axial_tilt: 0.1,
            bounding_mult: 1.2,
        },
    }
}
