use raylib::prelude::*;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::math::Vec4;
use crate::noise;
use crate::vertex::Vertex;

mod gas_giant;
mod rocky;
mod star;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Shader {
    Star,
    Rocky,
    GasGiant,
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::from_point(vertex.position);
    let transformed = uniforms.model_matrix * position;

    let mut result = vertex.clone();
    result.world_position = Vector3::new(
        transformed.x - uniforms.model_matrix.cols[3][0],
        transformed.y - uniforms.model_matrix.cols[3][1],
        transformed.z - uniforms.model_matrix.cols[3][2],
    );
    result.transformed_position = transformed.xyz();
    result.transformed_normal = uniforms.model_matrix.transform_normal(vertex.normal);
    result
}

/// Vector desde la superficie hacia la cámara, en espacio mundo.
pub fn surface_view(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    (uniforms.camera_pos - fragment.world_pos).normalize()
}

/// Coseno de vista radial. El valor absoluto compensa que el z-buffer usa
/// menor z como más cercano mientras la cámara ortográfica está en +z; la
/// cara visible tiene entonces la normal opuesta al vector hacia la cámara.
pub fn surface_mu(fragment: &Fragment, uniforms: &Uniforms) -> f32 {
    fragment
        .normal
        .normalize()
        .dot(surface_view(fragment, uniforms))
        .abs()
        .clamp(0.0, 1.0)
}

/// Difusa con light_dir apuntando desde la superficie hacia la fuente.
pub fn diffuse_factor(normal: Vector3, uniforms: &Uniforms) -> f32 {
    normal.normalize().dot(uniforms.light_dir).max(0.0)
}

pub fn fragment_shader(fragment: &Fragment, shader: Shader, uniforms: &Uniforms) -> Vector3 {
    if uniforms.debug_noise {
        let sample_position = fragment.object_pos.normalize() * uniforms.noise_frequency;
        let value = noise::fbm(sample_position, 5, 2.0, 0.5);
        return Vector3::new(value, value, value);
    }

    match shader {
        Shader::Star => star::shade(fragment, uniforms),
        Shader::Rocky => rocky::shade(fragment, uniforms),
        Shader::GasGiant => gas_giant::shade(fragment, uniforms),
    }
}

pub fn shader_name(shader: Shader) -> &'static str {
    match shader {
        Shader::Star => "Shader: Star",
        Shader::Rocky => "Shader: Rocky",
        Shader::GasGiant => "Shader: Gas Giant",
    }
}
