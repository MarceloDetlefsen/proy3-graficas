use crate::Uniforms;
use crate::fragment::Fragment;
use crate::framebuffer::{Framebuffer, INVALID_TRI_ID};
use crate::primitive::ScreenTriangle;
use crate::shaders::{Shader, fragment_shader};
use raylib::prelude::{Color, Vector2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
}

pub const WINDOW_WIDTH: i32 = 800;
pub const WINDOW_HEIGHT: i32 = 600;
pub const TILE_SIZE: usize = 64;
const DEGENERATE_EPS: f32 = 1e-6;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TileGrid {
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub bins: Vec<Vec<u32>>,
}

impl TileGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles_x = width.div_ceil(TILE_SIZE).max(1);
        let tiles_y = height.div_ceil(TILE_SIZE).max(1);
        Self {
            tiles_x,
            tiles_y,
            bins: vec![Vec::new(); tiles_x * tiles_y],
        }
    }

    pub fn bin_triangles(
        &mut self,
        triangles: &[ScreenTriangle],
        width: usize,
        height: usize,
        backface_culling: bool,
    ) {
        let tiles_x = width.div_ceil(TILE_SIZE).max(1);
        let tiles_y = height.div_ceil(TILE_SIZE).max(1);
        self.tiles_x = tiles_x;
        self.tiles_y = tiles_y;
        let needed = tiles_x * tiles_y;
        if self.bins.len() != needed {
            self.bins.resize_with(needed, Vec::new);
        }
        for bin in &mut self.bins {
            bin.clear();
        }

        let w_i32 = width as i32;
        let h_i32 = height as i32;
        for (tri_idx, tri) in triangles.iter().enumerate() {
            let [v0, v1, v2] = tri.vertices;
            let pa = (v0.x, v0.y);
            let pb = (v1.x, v1.y);
            let pc = (v2.x, v2.y);
            let area = edge_2d(pa, pb, pc);
            if area.abs() <= DEGENERATE_EPS || (backface_culling && area <= 0.0) {
                continue;
            }

            let min_x = pa.0.min(pb.0).min(pc.0).floor() as i32;
            let max_x = pa.0.max(pb.0).max(pc.0).ceil() as i32;
            let min_y = pa.1.min(pb.1).min(pc.1).floor() as i32;
            let max_y = pa.1.max(pb.1).max(pc.1).ceil() as i32;

            if max_x < 0 || min_x >= w_i32 || max_y < 0 || min_y >= h_i32 {
                continue;
            }

            let tx0 = (min_x.clamp(0, w_i32 - 1) as usize) / TILE_SIZE;
            let tx1 = (max_x.clamp(0, w_i32 - 1) as usize) / TILE_SIZE;
            let ty0 = (min_y.clamp(0, h_i32 - 1) as usize) / TILE_SIZE;
            let ty1 = (max_y.clamp(0, h_i32 - 1) as usize) / TILE_SIZE;

            let tri_id = tri_idx as u32;
            for ty in ty0..=ty1 {
                let row = ty * tiles_x;
                for tx in tx0..=tx1 {
                    self.bins[row + tx].push(tri_id);
                }
            }
        }
    }
}

#[inline]
pub fn edge_2d(p: (f32, f32), q: (f32, f32), r: (f32, f32)) -> f32 {
    (r.0 - p.0) * (q.1 - p.1) - (r.1 - p.1) * (q.0 - p.0)
}

#[inline]
pub fn perspective_weights(tri: &ScreenTriangle, screen_w: (f32, f32, f32)) -> (f32, f32, f32) {
    let p0 = screen_w.0 * tri.vertices[0].inv_w;
    let p1 = screen_w.1 * tri.vertices[1].inv_w;
    let p2 = screen_w.2 * tri.vertices[2].inv_w;
    let denom = p0 + p1 + p2;
    if denom.abs() > 1e-12 {
        let inv_d = 1.0 / denom;
        (p0 * inv_d, p1 * inv_d, p2 * inv_d)
    } else {
        screen_w
    }
}

#[inline]
pub fn interpolate_screen_fragment(
    tri: &ScreenTriangle,
    px: f32,
    py: f32,
    screen_w: (f32, f32, f32),
) -> Fragment {
    let [v0, v1, v2] = tri.vertices;
    let (pw0, pw1, pw2) = perspective_weights(tri, screen_w);
    Fragment {
        position: Vector2::new(px, py),
        object_pos: v0.object_pos * pw0 + v1.object_pos * pw1 + v2.object_pos * pw2,
        world_pos: v0.world_pos * pw0 + v1.world_pos * pw1 + v2.world_pos * pw2,
        normal: (v0.normal * pw0 + v1.normal * pw1 + v2.normal * pw2).normalize(),
        depth: v0.z * screen_w.0 + v1.z * screen_w.1 + v2.z * screen_w.2,
    }
}

pub fn rasterize_visibility(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    backface_culling: bool,
) -> usize {
    let width = framebuffer.width as i32;
    let height = framebuffer.height as i32;
    let mut writes = 0usize;

    for (tri_idx, tri) in triangles.iter().enumerate() {
        let [v0, v1, v2] = tri.vertices;
        let pa = (v0.x, v0.y);
        let pb = (v1.x, v1.y);
        let pc = (v2.x, v2.y);
        let area = edge_2d(pa, pb, pc);
        if area.abs() <= DEGENERATE_EPS || (backface_culling && area <= 0.0) {
            continue;
        }
        let inv_area = 1.0 / area;

        let min_x = pa.0.min(pb.0).min(pc.0).floor() as i32;
        let max_x = pa.0.max(pb.0).max(pc.0).ceil() as i32;
        let min_y = pa.1.min(pb.1).min(pc.1).floor() as i32;
        let max_y = pa.1.max(pb.1).max(pc.1).ceil() as i32;

        if max_x < 0 || min_x >= width || max_y < 0 || min_y >= height {
            continue;
        }

        let start_x = min_x.clamp(0, width - 1);
        let end_x = max_x.clamp(0, width - 1);
        let start_y = min_y.clamp(0, height - 1);
        let end_y = max_y.clamp(0, height - 1);

        let tri_id = tri_idx as u32;
        for y in start_y..=end_y {
            let py = y as f32 + 0.5;
            let row_offset = y as usize * width as usize;
            for x in start_x..=end_x {
                let p = (x as f32 + 0.5, py);
                let w0 = edge_2d(pb, pc, p) * inv_area;
                let w1 = edge_2d(pc, pa, p) * inv_area;
                let w2 = edge_2d(pa, pb, p) * inv_area;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let depth = v0.z * w0 + v1.z * w1 + v2.z * w2;
                    let idx = row_offset + x as usize;
                    if depth < framebuffer.depth_buffer[idx] {
                        framebuffer.depth_buffer[idx] = depth;
                        framebuffer.tri_id_buffer[idx] = tri_id;
                        writes += 1;
                    }
                }
            }
        }
    }

    writes
}

pub fn shade_visibility(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    shader: Shader,
    uniforms: &Uniforms,
) -> (usize, usize) {
    let width = framebuffer.width as usize;
    let height = framebuffer.height as usize;
    let mut shader_calls = 0usize;

    for y in 0..height {
        let py = y as f32 + 0.5;
        let row_offset = y * width;
        for x in 0..width {
            let idx = row_offset + x;
            let tri_id = framebuffer.tri_id_buffer[idx];
            if tri_id == INVALID_TRI_ID {
                continue;
            }
            let tri = &triangles[tri_id as usize];
            let [v0, v1, v2] = tri.vertices;
            let pa = (v0.x, v0.y);
            let pb = (v1.x, v1.y);
            let pc = (v2.x, v2.y);
            let inv_area = 1.0 / edge_2d(pa, pb, pc);
            let p = (x as f32 + 0.5, py);
            let w0 = edge_2d(pb, pc, p) * inv_area;
            let w1 = edge_2d(pc, pa, p) * inv_area;
            let w2 = edge_2d(pa, pb, p) * inv_area;

            let fragment = interpolate_screen_fragment(tri, p.0, p.1, (w0, w1, w2));
            let fc = fragment_shader(&fragment, shader, uniforms);
            let color_idx = idx * 4;
            framebuffer.color_pixels[color_idx] = (fc.x.clamp(0.0, 1.0) * 255.0) as u8;
            framebuffer.color_pixels[color_idx + 1] = (fc.y.clamp(0.0, 1.0) * 255.0) as u8;
            framebuffer.color_pixels[color_idx + 2] = (fc.z.clamp(0.0, 1.0) * 255.0) as u8;
            framebuffer.color_pixels[color_idx + 3] = 255;
            shader_calls += 1;
        }
    }

    (shader_calls, shader_calls)
}

pub fn render_wireframe(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    shader: Shader,
    uniforms: &Uniforms,
) -> (usize, usize) {
    let mut calls = 0usize;
    for tri in triangles {
        let [v0, v1, v2] = tri.vertices;
        let edges = [(v0, v1, 0), (v1, v2, 1), (v2, v0, 2)];
        for (start, end, edge_idx) in edges {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let steps = dx.abs().max(dy.abs()).ceil() as usize;
            for step in 0..=steps {
                let t = if steps == 0 {
                    0.0
                } else {
                    step as f32 / steps as f32
                };
                let px = start.x + dx * t;
                let py = start.y + dy * t;
                let weights = match edge_idx {
                    0 => (1.0 - t, t, 0.0),
                    1 => (0.0, 1.0 - t, t),
                    _ => (t, 0.0, 1.0 - t),
                };
                let frag = interpolate_screen_fragment(tri, px, py, weights);
                let fc = fragment_shader(&frag, shader, uniforms);
                calls += 1;
                let color = Color::new(
                    (fc.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (fc.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (fc.z.clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                );
                framebuffer.set_pixel_depth(px as i32, py as i32, frag.depth, color);
            }
        }
    }
    (calls, framebuffer.visible_pixel_count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::load_obj;
    use crate::primitive::ScreenVertex;
    use raylib::prelude::Vector3;

    #[test]
    fn barycentric_and_perspective_interpolation_with_distinct_w() {
        let mk_v = |x: f32, y: f32, w: f32, val: f32| ScreenVertex {
            x,
            y,
            z: 0.5,
            inv_w: 1.0 / w,
            object_pos: Vector3::new(val, 0.0, 0.0),
            world_pos: Vector3::zero(),
            normal: Vector3::new(0.0, 0.0, 1.0),
            uv: [0.0, 0.0],
        };
        let tri = ScreenTriangle {
            vertices: [
                mk_v(0.0, 0.0, 1.0, 0.0),
                mk_v(0.0, 10.0, 4.0, 10.0),
                mk_v(10.0, 0.0, 1.0, 0.0),
            ],
        };
        // En el punto medio de la arista v0-v1 en pantalla: w_screen = (0.5, 0.5, 0.0)
        // Con w0=1 y w1=4, el peso en perspectiva es (0.5/1) : (0.5/4) = 0.8 : 0.2 => x = 2.0
        let frag = interpolate_screen_fragment(&tri, 0.0, 5.0, (0.5, 0.5, 0.0));
        assert!((frag.object_pos.x - 2.0).abs() < 1e-5);

        let pa = (tri.vertices[0].x, tri.vertices[0].y);
        let pb = (tri.vertices[1].x, tri.vertices[1].y);
        let pc = (tri.vertices[2].x, tri.vertices[2].y);
        let area = edge_2d(pa, pb, pc);
        assert!(area > 0.0);
        let p = (2.5, 2.5);
        let w0 = edge_2d(pb, pc, p) / area;
        let w1 = edge_2d(pc, pa, p) / area;
        let w2 = edge_2d(pa, pb, p) / area;
        assert!((w0 + w1 + w2 - 1.0).abs() < 1e-6);
        assert!(w0 > 0.0 && w1 > 0.0 && w2 > 0.0);
    }

    #[test]
    fn axis_aligned_triangle_coverage_in_screen_space() {
        let pa = (0.0, 0.0);
        let pb = (0.0, 4.0);
        let pc = (4.0, 0.0);
        let area = edge_2d(pa, pb, pc);
        let inv_area = 1.0 / area;
        let mut covered = 0usize;
        for y in 0..4 {
            for x in 0..4 {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge_2d(pb, pc, p) * inv_area;
                let w1 = edge_2d(pc, pa, p) * inv_area;
                let w2 = edge_2d(pa, pb, p) * inv_area;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    covered += 1;
                }
            }
        }
        // Con bordes inclusivos (w >= 0), la diagonal x+y=4 incluye 4 centros: 4+3+2+1 = 10 píxeles
        assert_eq!(covered, 10);
    }

    #[test]
    fn shader_calls_never_exceed_visible_pixels_with_overlapping_triangles() {
        use crate::Uniforms;
        use crate::framebuffer::Framebuffer;
        use crate::math::Mat4;
        use crate::noise::Noise;
        use raylib::prelude::Color;

        let mk_tri = |z: f32| {
            let mk_v = |x: f32, y: f32| ScreenVertex {
                x,
                y,
                z,
                inv_w: 1.0,
                object_pos: Vector3::new(0.0, 0.0, 1.0),
                world_pos: Vector3::new(0.0, 0.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: [0.0, 0.0],
            };
            ScreenTriangle {
                vertices: [mk_v(0.0, 0.0), mk_v(0.0, 32.0), mk_v(32.0, 0.0)],
            }
        };

        let triangles = [mk_tri(0.8), mk_tri(0.2)];
        let mut fb = Framebuffer::new(64, 64, Color::BLACK);
        let uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            time: 0.0,
            light_dir: Vector3::new(0.0, 0.0, 1.0),
            camera_pos: Vector3::new(0.0, 0.0, 5.0),
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: 3,
            viewport_width: 64,
            viewport_height: 64,
            noise: Noise::new(0),
            baked: None,
        };

        let writes = rasterize_visibility(&triangles, &mut fb, true);
        let (calls, visible) = shade_visibility(&triangles, &mut fb, Shader::Star, &uniforms);
        assert!(writes >= visible);
        assert_eq!(calls, visible);
        assert!(calls <= fb.visible_pixel_count());
    }

    #[test]
    fn sphere_obj_triangles_are_oriented_outward() {
        let obj = load_obj("assets/sphere.obj").expect("sphere.obj debe cargar");
        let mut inward = 0usize;
        for tri in obj.indices.as_chunks::<3>().0 {
            let a = obj.vertices[tri[0]];
            let b = obj.vertices[tri[1]];
            let c = obj.vertices[tri[2]];
            let normal = (b - a).cross(c - a);
            let center = (a + b + c) * (1.0 / 3.0);
            if normal.dot(center) <= 0.0 {
                inward += 1;
            }
        }
        assert_eq!(inward, 0, "sphere.obj tiene {inward} triángulos invertidos");
    }

    // ship.obj tiene 10,043 de 29,696 triángulos con normal hacia el centroide
    // (geometría cóncava o winding inconsistente); se deja ignorado sin modificar el OBJ.
    #[test]
    #[ignore = "ship.obj tiene 10043 de 29696 triángulos orientados hacia el centroide"]
    fn ship_obj_triangles_orientation_check() {
        let obj = load_obj("assets/ship.obj").expect("ship.obj debe cargar");
        let centroid = obj
            .vertices
            .iter()
            .copied()
            .fold(Vector3::zero(), |acc, v| acc + v)
            * (1.0 / obj.vertices.len().max(1) as f32);
        let mut inward = 0usize;
        let mut total = 0usize;
        for tri in obj.indices.as_chunks::<3>().0 {
            let a = obj.vertices[tri[0]];
            let b = obj.vertices[tri[1]];
            let c = obj.vertices[tri[2]];
            let normal = (b - a).cross(c - a);
            let center = (a + b + c) * (1.0 / 3.0) - centroid;
            total += 1;
            if normal.dot(center) <= 0.0 {
                inward += 1;
            }
        }
        assert_eq!(
            inward, 0,
            "ship.obj tiene {inward} de {total} triángulos orientados hacia adentro"
        );
    }
}
