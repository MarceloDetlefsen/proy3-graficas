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
