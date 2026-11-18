use raylib::prelude::Vector3;

use crate::math::Mat4;
use crate::obj::Obj;
use crate::rasterizer::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

pub fn build_model_matrix(obj: &Obj, scale: f32, yaw: f32, pitch: f32, zoom: f32) -> Mat4 {
    let (min, max) = obj.bounds();
    let model_center = Vector3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let linear = Mat4::rotation_y(yaw)
        * Mat4::rotation_x(pitch)
        * Mat4::scale(Vector3::new(scale * zoom, -scale * zoom, scale * zoom));
    let transformed_center = linear.transform_normal(model_center) * 0.0
        + Vector3::new(
            linear.cols[0][0] * model_center.x
                + linear.cols[1][0] * model_center.y
                + linear.cols[2][0] * model_center.z,
            linear.cols[0][1] * model_center.x
                + linear.cols[1][1] * model_center.y
                + linear.cols[2][1] * model_center.z,
            linear.cols[0][2] * model_center.x
                + linear.cols[1][2] * model_center.y
                + linear.cols[2][2] * model_center.z,
        );
    let mut model_matrix = linear;
    model_matrix.cols[3][0] = VIEWPORT_WIDTH as f32 / 2.0 - transformed_center.x;
    model_matrix.cols[3][1] = VIEWPORT_HEIGHT as f32 / 2.0 - transformed_center.y;
    model_matrix.cols[3][2] = -transformed_center.z;
    model_matrix
}
