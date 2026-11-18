use raylib::prelude::Vector3;

use crate::math::Mat4;
use crate::obj::Obj;

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
