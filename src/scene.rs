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
    vec![
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
            name: "Pyros",
            kind: BodyKind::Planet,
            radius: 1.0,
            mass: 8.0,
            orbit_radius: 26.0,
            orbit_period: 70.0,
            phase: 4.2,
            orbit_inclination: 0.08,
            rotation_period: 16.0,
            axial_tilt: 0.25,
            shader: Shader::Rocky,
            surface: SurfaceParams {
                seed: 1,
                noise_frequency: 4.0,
                deformation_amplitude: 0.10,
                max_deformation: 0.10,
            },
        },
    ]
}

pub fn effective_radius(body: &Body) -> f32 {
    let amp_deform = body.radius * body.surface.deformation_amplitude.max(0.0);
    let deform = body.surface.max_deformation.max(amp_deform).max(0.0);
    body.radius + deform
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
}
