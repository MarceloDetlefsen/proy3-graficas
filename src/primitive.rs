#![allow(dead_code)]

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::shaders::vertex_shader;
use crate::vertex::Vertex;

#[derive(Clone, Copy, Debug)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub inv_w: f32,
    pub object_pos: Vector3,
    pub world_pos: Vector3,
    pub normal: Vector3,
    pub uv: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct ScreenTriangle {
    pub vertices: [ScreenVertex; 3],
}

pub fn assemble_screen_triangles(
    vertex_array: &[Vertex],
    uniforms: &Uniforms,
    transformed_scratch: &mut Vec<Vertex>,
    out_triangles: &mut Vec<ScreenTriangle>,
) {
    transformed_scratch.clear();
    transformed_scratch.extend(vertex_array.iter().map(|v| vertex_shader(v, uniforms)));

    out_triangles.clear();
    // Solución temporal: descartar triángulos con algún vértice detrás del plano near;
    // reemplazar en el issue de clipping.
    for group in transformed_scratch.as_chunks::<3>().0 {
        if group
            .iter()
            .any(|v| v.clip_w <= 0.0 || v.transformed_position.z < 0.0)
        {
            continue;
        }
        let to_sv = |v: &Vertex| ScreenVertex {
            x: v.transformed_position.x,
            y: v.transformed_position.y,
            z: v.transformed_position.z,
            inv_w: 1.0 / v.clip_w,
            object_pos: v.position,
            world_pos: v.world_position,
            normal: v.transformed_normal,
            uv: [0.0, 0.0],
        };
        out_triangles.push(ScreenTriangle {
            vertices: [to_sv(&group[0]), to_sv(&group[1]), to_sv(&group[2])],
        });
    }
}

pub fn primitive_assembly(transformed_vertices: &[Vertex]) -> Vec<[Vertex; 3]> {
    assert!(
        transformed_vertices.len().is_multiple_of(3),
        "La cantidad de vértices debe ser múltiplo de 3",
    );

    transformed_vertices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|group| [group[0].clone(), group[1].clone(), group[2].clone()])
        .collect()
}
