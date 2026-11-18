#![allow(dead_code)]
//! Convenciones matemáticas del renderizador:
//! - Sistema de coordenadas de mano derecha (right-handed).
//! - La cámara mira hacia `-Z` en su espacio local/vista.
//! - Vectores columna: las transformaciones se aplican como `M * v`.
//! - Coordenadas normalizadas de dispositivo (NDC): `x, y` en `[-1, 1]`, `z` en `[-1, 1]`.
//! - Profundidad en el z-buffer mapeada a `[0, 1]` (menor valor gana).
//! - Vectores 3D: se utiliza `raylib::prelude::Vector3`.

use raylib::prelude::Vector3;
use std::ops::Mul;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_point(v: Vector3) -> Self {
        Self::new(v.x, v.y, v.z, 1.0)
    }

    pub fn from_dir(v: Vector3) -> Self {
        Self::new(v.x, v.y, v.z, 0.0)
    }

    pub fn xyz(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

/// Matriz 4x4 almacenada por columnas (`cols[col][row]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub cols: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_cols(c0: [f32; 4], c1: [f32; 4], c2: [f32; 4], c3: [f32; 4]) -> Self {
        Self {
            cols: [c0, c1, c2, c3],
        }
    }

    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.cols[0][0] * v.x
                + self.cols[1][0] * v.y
                + self.cols[2][0] * v.z
                + self.cols[3][0] * v.w,
            self.cols[0][1] * v.x
                + self.cols[1][1] * v.y
                + self.cols[2][1] * v.z
                + self.cols[3][1] * v.w,
            self.cols[0][2] * v.x
                + self.cols[1][2] * v.y
                + self.cols[2][2] * v.z
                + self.cols[3][2] * v.w,
            self.cols[0][3] * v.x
                + self.cols[1][3] * v.y
                + self.cols[2][3] * v.z
                + self.cols[3][3] * v.w,
        )
    }

    pub fn mul_mat4(&self, rhs: &Mat4) -> Mat4 {
        let mut cols = [[0.0; 4]; 4];
        for (c, col) in cols.iter_mut().enumerate() {
            let v = Vec4::new(
                rhs.cols[c][0],
                rhs.cols[c][1],
                rhs.cols[c][2],
                rhs.cols[c][3],
            );
            let res = self.mul_vec4(v);
            *col = [res.x, res.y, res.z, res.w];
        }
        Mat4 { cols }
    }

    pub fn translation(t: Vector3) -> Self {
        Self::from_cols(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [t.x, t.y, t.z, 1.0],
        )
    }

    pub fn scale(s: Vector3) -> Self {
        Self::from_cols(
            [s.x, 0.0, 0.0, 0.0],
            [0.0, s.y, 0.0, 0.0],
            [0.0, 0.0, s.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::from_cols(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, s, 0.0],
            [0.0, -s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::from_cols(
            [c, 0.0, -s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self::from_cols(
            [c, s, 0.0, 0.0],
            [-s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Transforma una normal usando únicamente la submatriz 3x3 superior izquierda.
    /// Esto es exacto para rotaciones y escalas uniformes (no requiere inversa transpuesta
    /// mientras no se aplique escala no uniforme).
    pub fn transform_normal(&self, n: Vector3) -> Vector3 {
        Vector3::new(
            self.cols[0][0] * n.x + self.cols[1][0] * n.y + self.cols[2][0] * n.z,
            self.cols[0][1] * n.x + self.cols[1][1] * n.y + self.cols[2][1] * n.z,
            self.cols[0][2] * n.x + self.cols[1][2] * n.y + self.cols[2][2] * n.z,
        )
        .normalize()
    }

    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Self {
        let forward = (target - eye).normalize();
        let right = forward.cross(up).normalize();
        let camera_up = right.cross(forward);

        Self::from_cols(
            [right.x, camera_up.x, -forward.x, 0.0],
            [right.y, camera_up.y, -forward.y, 0.0],
            [right.z, camera_up.z, -forward.z, 0.0],
            [
                -right.dot(eye),
                -camera_up.dot(eye),
                forward.dot(eye),
                1.0,
            ],
        )
    }

    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let focal = 1.0 / (fov_y_rad * 0.5).tan();
        let inv_range = 1.0 / (near - far);

        Self::from_cols(
            [focal / aspect, 0.0, 0.0, 0.0],
            [0.0, focal, 0.0, 0.0],
            [0.0, 0.0, (far + near) * inv_range, -1.0],
            [0.0, 0.0, 2.0 * far * near * inv_range, 0.0],
        )
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        self.mul_vec4(rhs)
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        self.mul_mat4(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn assert_vec4_near(a: Vec4, b: Vec4, eps: f32) {
        assert!(
            (a.x - b.x).abs() <= eps
                && (a.y - b.y).abs() <= eps
                && (a.z - b.z).abs() <= eps
                && (a.w - b.w).abs() <= eps,
            "esperado {:?}, obtenido {:?}",
            b,
            a
        );
    }

    #[test]
    fn identity_preserves_vector_and_matrix() {
        let id = Mat4::identity();
        let v = Vec4::new(3.0, -2.0, 5.0, 1.0);
        assert_vec4_near(id * v, v, 1e-6);

        let t = Mat4::translation(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(id * t, t);
        assert_eq!(t * id, t);
    }

    #[test]
    fn translation_moves_point_but_not_direction() {
        let t = Mat4::translation(Vector3::new(4.0, -5.0, 6.0));
        let point = Vec4::from_point(Vector3::new(1.0, 2.0, 3.0));
        let dir = Vec4::from_dir(Vector3::new(1.0, 2.0, 3.0));

        assert_vec4_near(t * point, Vec4::new(5.0, -3.0, 9.0, 1.0), 1e-6);
        assert_vec4_near(t * dir, dir, 1e-6);
    }

    #[test]
    fn rotations_follow_right_hand_rule() {
        let rx = Mat4::rotation_x(FRAC_PI_2);
        let ry = Mat4::rotation_y(FRAC_PI_2);
        let rz = Mat4::rotation_z(FRAC_PI_2);

        // +Y rotado +90° alrededor de +X va hacia +Z
        assert_vec4_near(
            rx * Vec4::from_point(Vector3::new(0.0, 1.0, 0.0)),
            Vec4::new(0.0, 0.0, 1.0, 1.0),
            1e-6,
        );
        // +Z rotado +90° alrededor de +Y va hacia +X
        assert_vec4_near(
            ry * Vec4::from_point(Vector3::new(0.0, 0.0, 1.0)),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            1e-6,
        );
        // +X rotado +90° alrededor de +Z va hacia +Y
        assert_vec4_near(
            rz * Vec4::from_point(Vector3::new(1.0, 0.0, 0.0)),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            1e-6,
        );
    }

    #[test]
    fn composition_applies_scale_then_rotation_then_translation() {
        let s = Mat4::scale(Vector3::new(2.0, 2.0, 2.0));
        let r = Mat4::rotation_z(FRAC_PI_2);
        let t = Mat4::translation(Vector3::new(10.0, 0.0, 0.0));
        let m = t * r * s;

        let p = Vec4::from_point(Vector3::new(1.0, 0.0, 0.0));
        // Escala (2,0,0) -> rota Z 90° (0,2,0) -> traslada (10,2,0)
        assert_vec4_near(m * p, Vec4::new(10.0, 2.0, 0.0, 1.0), 1e-6);
    }
}
