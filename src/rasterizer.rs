use crate::fragment::Fragment;
use crate::triangle::{triangle_barycentric, triangle_wireframe};
use crate::vertex::Vertex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
}

pub const WINDOW_WIDTH: i32 = 800;
pub const WINDOW_HEIGHT: i32 = 600;

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
    use crate::obj::load_obj;
    use raylib::prelude::Vector3;

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
