use raylib::prelude::*;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::math::Vec4;
use crate::noise;
use crate::rasterizer::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
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
    let world = uniforms.model_matrix * position;
    let clip = uniforms.projection_matrix * (uniforms.view_matrix * world);

    let inv_w = if clip.w.abs() > 1e-6 {
        1.0 / clip.w
    } else {
        0.0
    };
    let ndc_x = clip.x * inv_w;
    let ndc_y = clip.y * inv_w;
    let ndc_z = clip.z * inv_w;

    let screen_x = (ndc_x * 0.5 + 0.5) * VIEWPORT_WIDTH as f32;
    let screen_y = (0.5 - ndc_y * 0.5) * VIEWPORT_HEIGHT as f32;
    let depth = ndc_z * 0.5 + 0.5;

    let mut result = vertex.clone();
    result.world_position = world.xyz();
    result.transformed_position = Vector3::new(screen_x, screen_y, depth);
    // Usa solo la parte de rotación/escala uniforme de la matriz de modelo.
    result.transformed_normal = uniforms.model_matrix.transform_normal(vertex.normal);
    result.clip_w = clip.w;
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
