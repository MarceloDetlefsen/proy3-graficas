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
