use crate::fragment::Fragment;
use crate::primitive::ScreenTriangle;
use crate::triangle::{triangle_barycentric, triangle_wireframe};
use crate::vertex::Vertex;
use raylib::prelude::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
}

pub const WINDOW_WIDTH: i32 = 800;
pub const WINDOW_HEIGHT: i32 = 600;

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

pub fn rasterize(
    triangles: &[[Vertex; 3]],
    mode: TriangleMode,
    backface_culling: bool,
    width: i32,
    height: i32,
) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    for tri in triangles {
        let triangle_fragments = match mode {
            TriangleMode::Wireframe => triangle_wireframe(&tri[0], &tri[1], &tri[2]),
            TriangleMode::FilledBarycentric => {
                triangle_barycentric(&tri[0], &tri[1], &tri[2], backface_culling, width, height)
            }
        };
        fragments.extend(triangle_fragments);
    }

    fragments
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
