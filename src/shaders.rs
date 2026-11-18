use raylib::prelude::*;

use crate::Uniforms;
use crate::clipping::ClipVertex;
use crate::fragment::Fragment;
use crate::math::Vec4;
use crate::vertex::Vertex;

pub mod ardens;
pub mod gas_giant;
pub mod rocky;
pub mod ship;
pub mod star;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Shader {
    Star,
    Rocky,
    GasGiant,
    Ardens,
    Ship,
    Prop(crate::surface_scatter::SurfacePropKind),
}

#[allow(dead_code)]
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

    let screen_x = (ndc_x * 0.5 + 0.5) * uniforms.viewport_width as f32;
    let screen_y = (0.5 - ndc_y * 0.5) * uniforms.viewport_height as f32;
    let depth = ndc_z * 0.5 + 0.5;

    let mut result = vertex.clone();
    result.world_position = world.xyz();
    result.transformed_position = Vector3::new(screen_x, screen_y, depth);
    // Usa solo la parte de rotación/escala uniforme de la matriz de modelo.
    result.transformed_normal = uniforms.model_matrix.transform_normal(vertex.normal);
    result.clip_w = clip.w;
    result
}

pub fn vertex_to_clip(vertex: &Vertex, uniforms: &Uniforms) -> ClipVertex {
    let world = uniforms.model_matrix * Vec4::from_point(vertex.position);
    let clip_pos = uniforms.projection_matrix * (uniforms.view_matrix * world);
    ClipVertex {
        clip_pos,
        object_pos: vertex.position,
        world_pos: world.xyz(),
        normal: uniforms.model_matrix.transform_normal(vertex.normal),
        uv: vertex.uv,
    }
}

/// Vector desde la superficie hacia la cámara, en espacio mundo.
pub fn surface_view(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    (uniforms.camera_pos - fragment.world_pos).normalize()
}

/// Coseno de vista radial entre la normal en mundo y la dirección hacia la cámara real.
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
        let value = uniforms.noise.fbm(sample_position, 5, 2.0, 0.5);
        return Vector3::new(value, value, value);
    }

    match shader {
        Shader::Star => star::shade(fragment, uniforms),
        Shader::Rocky => rocky::shade(fragment, uniforms),
        Shader::GasGiant => gas_giant::shade(fragment, uniforms),
        Shader::Ardens => ardens::shade(fragment, uniforms),
        Shader::Ship => ship::shade(fragment, uniforms),
        Shader::Prop(kind) => {
            use crate::surface_scatter::SurfacePropKind;
            let base = match kind {
                SurfacePropKind::Tree => Vector3::new(0.22, 0.72, 0.28),
                SurfacePropKind::Rock => Vector3::new(0.48, 0.45, 0.44),
                SurfacePropKind::Building => Vector3::new(0.62, 0.74, 0.85),
                SurfacePropKind::Monument => Vector3::new(0.95, 0.76, 0.28),
            };
            let diff = diffuse_factor(fragment.normal, uniforms);
            base * (0.22 + 0.78 * diff)
        }
    }
}

pub fn shader_name(shader: Shader) -> &'static str {
    match shader {
        Shader::Star => "Shader: Star",
        Shader::Rocky => "Shader: Rocky",
        Shader::GasGiant => "Shader: Gas Giant",
        Shader::Ardens => "Shader: Ardens",
        Shader::Ship => "Shader: Ship",
        Shader::Prop(_) => "Shader: Surface Prop",
    }
}
