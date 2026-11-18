#![allow(dead_code)]
//! Fondo estelar procedural sin geometría para píxeles vacíos del framebuffer.
//!
//! Reconstruye la dirección de la vista por píxel mediante la inversa de la
//! matriz `P * V` y evalúa un campo de estrellas procedural basado en hash de
//! celdas 3D sobre la esfera unitaria, con temperatura de color y parpadeo suave.

use raylib::prelude::Vector3;

use crate::math::{Mat4, Vec4};

/// Reconstruye la dirección unitaria del rayo de vista para el centro del píxel `(x, y)`.
pub fn view_direction_for_pixel(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    inv_vp: &Mat4,
    camera_pos: Vector3,
) -> Vector3 {
    let ndc_x = ((x + 0.5) / width.max(1.0)) * 2.0 - 1.0;
    let ndc_y = 1.0 - ((y + 0.5) / height.max(1.0)) * 2.0;
    let world_h = *inv_vp * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    let inv_w = if world_h.w.abs() > 1e-8 {
        1.0 / world_h.w
    } else {
        1.0
    };
    let world_pt = world_h.xyz() * inv_w;
    (world_pt - camera_pos).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;

    #[test]
    fn center_pixel_unprojects_to_camera_forward_direction() {
        let cam = Camera::new(
            Vector3::new(3.0, 2.0, 12.0),
            Vector3::new(-1.0, 0.5, 0.0),
            45.0_f32.to_radians(),
            0.1,
            300.0,
        );
        let vp = cam.projection_matrix(4.0 / 3.0) * cam.view_matrix();
        let inv_vp = vp.inverse().expect("VP debe ser invertible");
        let dir = view_direction_for_pixel(399.5, 299.5, 800.0, 600.0, &inv_vp, cam.position);
        let expected = (cam.target - cam.position).normalize();
        assert!((dir - expected).length() < 1e-4);
    }
}
