#![allow(dead_code)]

use raylib::prelude::Vector3;

use crate::math::Mat4;
use crate::obj::Obj;
use crate::shaders::Shader;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyKind {
    Star,
    Planet,
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceParams {
    pub seed: u32,
    pub noise_frequency: f32,
    pub deformation_amplitude: f32,
    pub max_deformation: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub name: &'static str,
    pub kind: BodyKind,
    pub radius: f32,
    pub mass: f32,
    pub orbit_radius: f32,
    pub orbit_period: f32,
    pub phase: f32,
    pub orbit_inclination: f32,
    pub rotation_period: f32,
    pub axial_tilt: f32,
    pub shader: Shader,
    pub surface: SurfaceParams,
}

pub fn solar_system() -> Vec<Body> {
    let mut bodies = vec![
        Body {
            name: "Sol",
            kind: BodyKind::Star,
            radius: 3.5,
            mass: 1000.0,
            orbit_radius: 0.0,
            orbit_period: 0.0,
            phase: 0.0,
            orbit_inclination: 0.0,
            rotation_period: 30.0,
            axial_tilt: 0.0,
            shader: Shader::Star,
            surface: SurfaceParams {
                seed: 0,
                noise_frequency: 4.0,
                deformation_amplitude: 0.0,
                max_deformation: 0.0,
            },
        },
        Body {
            name: "Zebes",
            kind: BodyKind::Planet,
            radius: 1.25,
            mass: 12.0,
            orbit_radius: 10.0,
            orbit_period: 28.0,
            phase: 0.35,
            orbit_inclination: 0.05,
            rotation_period: 12.0,
            axial_tilt: 0.15,
            shader: Shader::Rocky,
            surface: SurfaceParams {
                seed: 0,
                noise_frequency: 4.0,
                deformation_amplitude: 0.12,
                max_deformation: 1.25 * 0.12,
            },
        },
        Body {
            name: "Tau Ceti",
            kind: BodyKind::Planet,
            radius: 2.1,
            mass: 45.0,
            orbit_radius: 18.0,
            orbit_period: 48.0,
            phase: 2.1,
            orbit_inclination: -0.06,
            rotation_period: 9.0,
            axial_tilt: 0.08,
            shader: Shader::GasGiant,
            surface: SurfaceParams {
                seed: 0,
                noise_frequency: 4.0,
                deformation_amplitude: 0.0,
                max_deformation: 0.0,
            },
        },
        Body {
            name: "Ardens",
            kind: BodyKind::Planet,
            radius: 1.1,
            mass: 12.0,
            orbit_radius: 34.0,
            orbit_period: 70.0,
            phase: 1.1,
            orbit_inclination: 0.04,
            rotation_period: 35.0,
            axial_tilt: 0.10,
            shader: Shader::Ardens,
            surface: SurfaceParams {
                seed: 11,
                noise_frequency: 6.0,
                deformation_amplitude: 0.04,
                max_deformation: 1.1 * 0.04,
            },
        },
        Body {
            name: "Abismo",
            kind: BodyKind::Planet,
            radius: 1.3,
            mass: 15.0,
            orbit_radius: 47.6,
            orbit_period: 95.0,
            phase: 2.8,
            orbit_inclination: -0.05,
            rotation_period: 50.0,
            axial_tilt: 0.06,
            shader: Shader::Abismo,
            surface: SurfaceParams {
                seed: 22,
                noise_frequency: 14.0,
                deformation_amplitude: 0.015,
                max_deformation: 1.3 * 0.015,
            },
        },
        Body {
            name: "Basquia",
            kind: BodyKind::Planet,
            radius: 0.9,
            mass: 8.0,
            orbit_radius: 64.6,
            orbit_period: 120.0,
            phase: 4.5,
            orbit_inclination: 0.07,
            rotation_period: 28.0,
            axial_tilt: 0.22,
            shader: Shader::Basquia,
            surface: SurfaceParams {
                seed: 33,
                noise_frequency: 4.0,
                deformation_amplitude: 0.03,
                max_deformation: 0.9 * 0.03,
            },
        },
        Body {
            name: "Vestigio",
            kind: BodyKind::Planet,
            radius: 1.0,
            mass: 10.0,
            orbit_radius: 81.6,
            orbit_period: 150.0,
            phase: 0.85,
            orbit_inclination: -0.04,
            rotation_period: 60.0,
            axial_tilt: 0.14,
            shader: Shader::Vestigio,
            surface: SurfaceParams {
                seed: 44,
                noise_frequency: 5.0,
                deformation_amplitude: 0.07,
                max_deformation: 0.07,
            },
        },
        Body {
            name: "Mural",
            kind: BodyKind::Planet,
            radius: 1.2,
            mass: 13.0,
            orbit_radius: 98.6,
            orbit_period: 180.0,
            phase: 3.7,
            orbit_inclination: 0.05,
            rotation_period: 80.0,
            axial_tilt: 0.12,
            shader: Shader::Mural,
            surface: SurfaceParams {
                seed: 55,
                noise_frequency: 7.0,
                deformation_amplitude: 0.04,
                max_deformation: 1.2 * 0.04,
            },
        },
        Body {
            name: "Fundido",
            kind: BodyKind::Planet,
            radius: 1.0,
            mass: 9.0,
            orbit_radius: 115.6,
            orbit_period: 210.0,
            phase: 5.3,
            orbit_inclination: -0.06,
            rotation_period: 45.0,
            axial_tilt: 0.08,
            shader: Shader::Fundido,
            surface: SurfaceParams {
                seed: 66,
                noise_frequency: 3.0,
                deformation_amplitude: 0.12,
                max_deformation: 1.0 * 0.12,
            },
        },
    ];
    for b in &mut bodies {
        if !matches!(b.kind, BodyKind::Star) {
            b.orbit_radius *= crate::scene_config::ORBIT_DISTANCE_SCALE;
            b.orbit_period /= crate::scene_config::ORBIT_SPEED_SCALE;
        }
    }
    bodies
}

pub fn favorites_system() -> Vec<Body> {
    crate::showcase::favorites_ready_entries()
        .into_iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            let shader = entry.shader?;
            let is_star = entry.tipo == "Estrella";
            Some(Body {
                name: entry.nombre,
                kind: if is_star {
                    BodyKind::Star
                } else {
                    BodyKind::Planet
                },
                radius: if is_star { 3.5 } else { 1.2 },
                mass: if is_star { 1000.0 } else { 12.0 },
                orbit_radius: if is_star { 0.0 } else { 12.0 + i as f32 * 8.0 },
                orbit_period: if is_star { 0.0 } else { 28.0 + i as f32 * 14.0 },
                phase: i as f32 * 1.1,
                orbit_inclination: 0.04,
                rotation_period: 25.0,
                axial_tilt: 0.1,
                shader,
                surface: SurfaceParams {
                    seed: i as u32,
                    noise_frequency: 4.0,
                    deformation_amplitude: 0.0,
                    max_deformation: 0.0,
                },
            })
        })
        .collect()
}

pub fn system_bodies(id: crate::app::SystemId) -> Vec<Body> {
    match id {
        crate::app::SystemId::Realista => solar_system(),
        crate::app::SystemId::Favoritos => favorites_system(),
    }
}

pub fn effective_radius(body: &Body) -> f32 {
    if matches!(body.shader, Shader::PhoenixSuns) {
        return body.radius * 3.4;
    }
    let amp_deform = body.radius * body.surface.deformation_amplitude.max(0.0);
    let deform = body.surface.max_deformation.max(amp_deform).max(0.0);
    body.radius + deform
}

/// Radio de esfera envolvente para frustum culling (en `Shader::Meteor` incluye cola y orbe).
pub fn culling_radius(body: &Body) -> f32 {
    match body.shader {
        Shader::Meteor => body.radius * 5.2,
        Shader::Spurs => body.radius * 4.2,
        _ => effective_radius(body),
    }
}

pub fn body_position_at(body: &Body, time: f32) -> Vector3 {
    if body.orbit_radius <= 0.0 || body.orbit_period.abs() <= 1e-6 {
        return body_static_position(body);
    }
    let angle = body.phase + std::f32::consts::TAU * (time / body.orbit_period);
    Vector3::new(
        body.orbit_radius * angle.cos(),
        body.orbit_radius * angle.sin() * body.orbit_inclination.sin(),
        body.orbit_radius * angle.sin() * body.orbit_inclination.cos(),
    )
}

pub fn body_velocity_at(body: &Body, time: f32) -> Vector3 {
    if body.orbit_radius <= 0.0 || body.orbit_period.abs() <= 1e-6 {
        return Vector3::zero();
    }
    let omega = std::f32::consts::TAU / body.orbit_period;
    let angle = body.phase + omega * time;
    Vector3::new(
        -body.orbit_radius * omega * angle.sin(),
        body.orbit_radius * omega * angle.cos() * body.orbit_inclination.sin(),
        body.orbit_radius * omega * angle.cos() * body.orbit_inclination.cos(),
    )
}

pub fn body_static_position(body: &Body) -> Vector3 {
    Vector3::new(
        body.orbit_radius * body.phase.cos(),
        body.orbit_radius * body.phase.sin() * body.orbit_inclination.sin(),
        body.orbit_radius * body.phase.sin() * body.orbit_inclination.cos(),
    )
}

pub fn body_model_matrix_at(obj: &Obj, body: &Body, time: f32) -> Mat4 {
    let (min, max) = obj.bounds();
    let max_extent = (max.x - min.x)
        .max(max.y - min.y)
        .max(max.z - min.z)
        .max(1e-4);
    let unit_scale = (body.radius * 2.0) / max_extent;
    let center = (min + max) * 0.5;
    let pos = body_position_at(body, time);
    let spin = if body.rotation_period.abs() > 1e-6 {
        std::f32::consts::TAU * (time / body.rotation_period)
    } else {
        0.0
    };
    Mat4::translation(pos)
        * Mat4::rotation_z(body.axial_tilt)
        * Mat4::rotation_y(spin)
        * Mat4::scale(Vector3::new(unit_scale, unit_scale, unit_scale))
        * Mat4::translation(Vector3::new(-center.x, -center.y, -center.z))
}

pub fn body_rt_matrix_at(body: &Body, time: f32) -> Mat4 {
    let pos = body_position_at(body, time);
    let spin = if body.rotation_period.abs() > 1e-6 {
        std::f32::consts::TAU * (time / body.rotation_period)
    } else {
        0.0
    };
    Mat4::translation(pos) * Mat4::rotation_z(body.axial_tilt) * Mat4::rotation_y(spin)
}

pub fn body_model_matrix_unit(body: &Body, time: f32) -> Mat4 {
    body_rt_matrix_at(body, time)
        * Mat4::scale(Vector3::new(body.radius, body.radius, body.radius))
}

/// Calcula la dirección unitaria de luz desde `body_pos` hacia `sun_pos`,
/// usando `fallback` cuando ambos coinciden (p. ej. el propio Sol).
pub fn body_light_dir(body_pos: Vector3, sun_pos: Vector3, fallback: Vector3) -> Vector3 {
    let to_sun = sun_pos - body_pos;
    if to_sun.length_sqr() > 1e-6 {
        to_sun.normalize()
    } else if fallback.length_sqr() > 1e-6 {
        fallback.normalize()
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    }
}

pub fn build_model_matrix(obj: &Obj, yaw: f32, pitch: f32, scale: f32) -> Mat4 {
    let (min, max) = obj.bounds();
    let model_center = Vector3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    Mat4::rotation_y(yaw)
        * Mat4::rotation_x(pitch)
        * Mat4::scale(Vector3::new(scale, scale, scale))
        * Mat4::translation(Vector3::new(
            -model_center.x,
            -model_center.y,
            -model_center.z,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocky_planet_zebes_has_orbit_deformation_and_light_direction() {
        let bodies = solar_system();
        let sol = &bodies[0];
        let zebes = &bodies[1];
        assert!(matches!(sol.shader, Shader::Star));
        assert_eq!(sol.orbit_radius, 0.0);
        assert!(sol.rotation_period > 0.0);

        assert_eq!(zebes.name, "Zebes");
        assert!(matches!(zebes.shader, Shader::Rocky));
        assert!(zebes.orbit_radius > 0.0 && zebes.orbit_period > 0.0);
        assert!(zebes.surface.deformation_amplitude > 0.0);
        assert!(effective_radius(zebes) > zebes.radius);

        let sun_pos = body_position_at(sol, 5.0);
        let zebes_pos = body_position_at(zebes, 5.0);
        let ldir = body_light_dir(zebes_pos, sun_pos, Vector3::new(0.0, 1.0, 0.0));
        assert!((ldir.length() - 1.0).abs() < 1e-5);
        assert!(ldir.dot(sun_pos - zebes_pos) > 0.0);
    }

    #[test]
    fn gas_giant_tau_ceti_has_orbit_and_animated_bands() {
        let bodies = solar_system();
        let tau = &bodies[2];
        assert_eq!(tau.name, "Tau Ceti");
        assert!(matches!(tau.shader, Shader::GasGiant));
        assert!(tau.orbit_radius > 0.0 && tau.orbit_period > 0.0);
        assert!(tau.rotation_period > 0.0);
        assert_eq!(tau.surface.deformation_amplitude, 0.0);

        let p0 = body_position_at(tau, 0.0);
        let p1 = body_position_at(tau, 10.0);
        assert!((p1 - p0).length() > 1.0);
    }

    #[test]
    fn system_bodies_builds_realistic_and_favorites_cleanly() {
        let realistic = system_bodies(crate::app::SystemId::Realista);
        assert_eq!(realistic.len(), 9);
        assert_eq!(realistic[0].name, "Sol");

        let favs = system_bodies(crate::app::SystemId::Favoritos);
        assert_eq!(favs.len(), crate::showcase::favorites_ready_entries().len());
        assert!(!crate::showcase::is_favorites_system_ready(
            crate::showcase::SHOWCASE_ENTRIES
        ));
    }
}
