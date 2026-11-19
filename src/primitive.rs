use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::clipping::{ClipVertex, clip_triangle_near};
use crate::shaders::vertex_to_clip;
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
    #[allow(dead_code)]
    pub uv: [f32; 2],
    pub color: Vector3,
}

#[derive(Clone, Copy, Debug)]
pub struct ScreenTriangle {
    pub vertices: [ScreenVertex; 3],
}

#[inline]
fn clip_to_screen_vertex(v: &ClipVertex, viewport_w: f32, viewport_h: f32) -> ScreenVertex {
    let w = v.clip_pos.w.max(1e-6);
    let inv_w = 1.0 / w;
    let ndc_x = v.clip_pos.x * inv_w;
    let ndc_y = v.clip_pos.y * inv_w;
    let ndc_z = v.clip_pos.z * inv_w;

    ScreenVertex {
        x: (ndc_x * 0.5 + 0.5) * viewport_w,
        y: (0.5 - ndc_y * 0.5) * viewport_h,
        z: ndc_z * 0.5 + 0.5,
        inv_w,
        object_pos: v.object_pos,
        world_pos: v.world_pos,
        normal: v.normal.normalize(),
        uv: v.uv,
        color: v.color,
    }
}

pub fn assemble_screen_triangles(
    vertex_array: &[Vertex],
    uniforms: &Uniforms,
    clip_scratch: &mut Vec<ClipVertex>,
    out_triangles: &mut Vec<ScreenTriangle>,
) {
    out_triangles.clear();
    append_screen_triangles(vertex_array, uniforms, clip_scratch, out_triangles);
}

pub fn append_screen_triangles(
    vertex_array: &[Vertex],
    uniforms: &Uniforms,
    clip_scratch: &mut Vec<ClipVertex>,
    out_triangles: &mut Vec<ScreenTriangle>,
) {
    clip_scratch.clear();
    clip_scratch.extend(vertex_array.iter().map(|v| vertex_to_clip(v, uniforms)));

    let vw = uniforms.viewport_width as f32;
    let vh = uniforms.viewport_height as f32;

    for group in clip_scratch.as_chunks::<3>().0 {
        let clipped = clip_triangle_near(group);
        for tri in clipped.as_slice() {
            out_triangles.push(ScreenTriangle {
                vertices: [
                    clip_to_screen_vertex(&tri[0], vw, vh),
                    clip_to_screen_vertex(&tri[1], vw, vh),
                    clip_to_screen_vertex(&tri[2], vw, vh),
                ],
            });
        }
    }
}
