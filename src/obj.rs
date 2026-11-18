#![allow(dead_code)]

use crate::vertex::Vertex;
use raylib::prelude::Vector3;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceCorner {
    pub pos: usize,
    pub tex: Option<usize>,
    pub normal: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub texcoords: Vec<[f32; 2]>,
    pub vertex_normals: Vec<Vector3>,
    pub source_normals: Vec<Vector3>,
    pub indices: Vec<usize>,
    pub corner_indices: Vec<FaceCorner>,
}

impl Obj {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            texcoords: Vec::new(),
            vertex_normals: Vec::new(),
            source_normals: Vec::new(),
            indices: Vec::new(),
            corner_indices: Vec::new(),
        }
    }

    pub fn load(path: &str) -> io::Result<Self> {
        load_obj(path)
    }

    pub fn corner_normal(&self, corner: FaceCorner) -> Vector3 {
        if let Some(ni) = corner.normal
            && let Some(&n) = self.source_normals.get(ni)
        {
            return n.normalize();
        }
        self.vertex_normals
            .get(corner.pos)
            .copied()
            .unwrap_or_else(|| Vector3::new(0.0, 1.0, 0.0))
    }

    pub fn corner_uv(&self, corner: FaceCorner) -> [f32; 2] {
        corner
            .tex
            .and_then(|ti| self.texcoords.get(ti).copied())
            .unwrap_or([0.0, 0.0])
    }

    /// Construye una malla indexada duplicando vértices únicamente cuando difieren sus atributos.
    pub fn to_indexed_mesh(&self) -> (Vec<Vertex>, Vec<usize>) {
        let mut unique: HashMap<FaceCorner, usize> = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::with_capacity(self.corner_indices.len());

        for &corner in &self.corner_indices {
            if corner.pos >= self.vertices.len() {
                continue;
            }
            let idx = *unique.entry(corner).or_insert_with(|| {
                let next_idx = vertices.len();
                vertices.push(Vertex::with_uv(
                    self.vertices[corner.pos],
                    self.corner_normal(corner),
                    self.corner_uv(corner),
                ));
                next_idx
            });
            indices.push(idx);
        }

        compute_tangent_basis(&mut vertices, &indices);
        (vertices, indices)
    }

    /// Calcula la caja envolvente (bounding box) del modelo devolviendo (min, max).
    pub fn bounds(&self) -> (Vector3, Vector3) {
        if self.vertices.is_empty() {
            return (Vector3::zero(), Vector3::zero());
        }

        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for v in &self.vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);

            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        (min, max)
    }
}

/// Calcula tangentes y bitangentes ortonormalizadas por vértice a partir de
/// posiciones y coordenadas de textura `uv`.
pub fn compute_tangent_basis(vertices: &mut [Vertex], indices: &[usize]) {
    let mut tan_accum = vec![Vector3::zero(); vertices.len()];
    let mut bitan_accum = vec![Vector3::zero(); vertices.len()];

    for tri in indices.as_chunks::<3>().0 {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
        let p0 = vertices[i0].position;
        let p1 = vertices[i1].position;
        let p2 = vertices[i2].position;
        let uv0 = vertices[i0].uv;
        let uv1 = vertices[i1].uv;
        let uv2 = vertices[i2].uv;

        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let du1 = uv1[0] - uv0[0];
        let dv1 = uv1[1] - uv0[1];
        let du2 = uv2[0] - uv0[0];
        let dv2 = uv2[1] - uv0[1];

        let det = du1 * dv2 - du2 * dv1;
        if det.abs() <= 1e-8 {
            continue;
        }
        let inv = 1.0 / det;
        let t = (e1 * dv2 - e2 * dv1) * inv;
        let b = (e2 * du1 - e1 * du2) * inv;

        for &idx in &[i0, i1, i2] {
            tan_accum[idx] += t;
            bitan_accum[idx] += b;
        }
    }

    for (i, v) in vertices.iter_mut().enumerate() {
        let n = if v.normal.length_sqr() > 1e-10 {
            v.normal.normalize()
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };
        let t_raw = tan_accum[i];
        let t_ortho = t_raw - n * n.dot(t_raw);
        let tangent = if t_ortho.length_sqr() > 1e-10 {
            t_ortho.normalize()
        } else {
            let axis = if n.y.abs() < 0.9 {
                Vector3::new(0.0, 1.0, 0.0)
            } else {
                Vector3::new(1.0, 0.0, 0.0)
            };
            axis.cross(n).normalize()
        };
        let mut bitangent = n.cross(tangent).normalize();
        if bitan_accum[i].length_sqr() > 1e-10 && bitan_accum[i].dot(bitangent) < 0.0 {
            bitangent = -bitangent;
        }
        v.tangent = tangent;
        v.bitangent = bitangent;
    }
}

fn resolve_obj_index(token: &str, len: usize) -> Option<usize> {
    let idx = token.parse::<isize>().ok()?;
    if idx > 0 {
        let zero_based = (idx - 1) as usize;
        (zero_based < len).then_some(zero_based)
    } else if idx < 0 {
        let offset = (-idx) as usize;
        (offset <= len).then(|| len - offset)
    } else {
        None
    }
}

fn parse_face_corner(
    token: &str,
    v_len: usize,
    vt_len: usize,
    vn_len: usize,
) -> Option<FaceCorner> {
    let mut segs = token.split('/');
    let v_str = segs.next()?;
    let pos = resolve_obj_index(v_str, v_len)?;
    let tex = segs
        .next()
        .filter(|s| !s.is_empty())
        .and_then(|s| resolve_obj_index(s, vt_len));
    let normal = segs
        .next()
        .filter(|s| !s.is_empty())
        .and_then(|s| resolve_obj_index(s, vn_len));
    Some(FaceCorner { pos, tex, normal })
}

/// Carga un archivo OBJ reconociendo vértices, UVs (`vt`), normales (`vn`) y caras.
pub fn load_obj(path: &str) -> io::Result<Obj> {
    let file = File::open(path)?;
    load_obj_from_reader(BufReader::new(file))
}

pub fn load_obj_from_reader<R: BufRead>(reader: R) -> io::Result<Obj> {
    let mut vertices = Vec::new();
    let mut texcoords = Vec::new();
    let mut source_normals = Vec::new();
    let mut indices = Vec::new();
    let mut corner_indices = Vec::new();
    let mut faces = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Ignorar líneas vacías o comentarios
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let prefix = match parts.next() {
            Some(p) => p,
            None => continue,
        };

        match prefix {
            "v" => {
                if let (Some(x_str), Some(y_str), Some(z_str)) =
                    (parts.next(), parts.next(), parts.next())
                    && let (Ok(x), Ok(y), Ok(z)) = (
                        x_str.parse::<f32>(),
                        y_str.parse::<f32>(),
                        z_str.parse::<f32>(),
                    )
                {
                    vertices.push(Vector3::new(x, y, z));
                }
            }
            "vt" => {
                if let (Some(u_str), Some(v_str)) = (parts.next(), parts.next())
                    && let (Ok(u), Ok(v)) = (u_str.parse::<f32>(), v_str.parse::<f32>())
                {
                    texcoords.push([u, v]);
                }
            }
            "vn" => {
                if let (Some(x_str), Some(y_str), Some(z_str)) =
                    (parts.next(), parts.next(), parts.next())
                    && let (Ok(x), Ok(y), Ok(z)) = (
                        x_str.parse::<f32>(),
                        y_str.parse::<f32>(),
                        z_str.parse::<f32>(),
                    )
                {
                    source_normals.push(Vector3::new(x, y, z));
                }
            }
            "f" => {
                let mut face_corners = Vec::new();
                for part in parts {
                    if let Some(corner) = parse_face_corner(
                        part,
                        vertices.len(),
                        texcoords.len(),
                        source_normals.len(),
                    ) {
                        face_corners.push(corner);
                    }
                }
                if face_corners.len() >= 3 {
                    for i in 1..(face_corners.len() - 1) {
                        let tri = [face_corners[0], face_corners[i], face_corners[i + 1]];
                        for c in tri {
                            indices.push(c.pos);
                            corner_indices.push(c);
                        }
                        faces.push(vec![tri[0].pos, tri[1].pos, tri[2].pos]);
                    }
                }
            }
            _ => {}
        }
    }

    let needs_smooth_normals =
        source_normals.is_empty() || corner_indices.iter().any(|c| c.normal.is_none());
    let vertex_normals = if !needs_smooth_normals {
        let mut from_file = vec![Vector3::new(0.0, 1.0, 0.0); vertices.len()];
        for c in &corner_indices {
            if let Some(ni) = c.normal
                && let Some(&n) = source_normals.get(ni)
                && c.pos < from_file.len()
            {
                from_file[c.pos] = n.normalize();
            }
        }
        from_file
    } else {
        let centroid = vertices
            .iter()
            .copied()
            .fold(Vector3::zero(), |sum, position| sum + position)
            * (1.0 / vertices.len().max(1) as f32);
        if centroid.length() <= 1e-5 {
            vertices
                .iter()
                .map(|position| position.normalize())
                .collect()
        } else {
            let mut accumulated = vec![Vector3::zero(); vertices.len()];
            for face in faces {
                let [a, b, c] = [face[0], face[1], face[2]];
                if a >= vertices.len() || b >= vertices.len() || c >= vertices.len() {
                    continue;
                }
                let face_normal = (vertices[b] - vertices[a]).cross(vertices[c] - vertices[a]);
                for &index in &face {
                    if index < accumulated.len() {
                        accumulated[index] += face_normal;
                    }
                }
            }
            accumulated
                .into_iter()
                .map(|normal| normal.normalize())
                .collect()
        }
    };

    Ok(Obj {
        vertices,
        texcoords,
        vertex_normals,
        source_normals,
        indices,
        corner_indices,
    })
}

#[cfg(test)]
mod tests {
    use super::{load_obj, load_obj_from_reader};
    use raylib::prelude::Vector3;

    #[test]
    fn sphere_counts_and_smooth_normals_are_valid() {
        let obj = load_obj("assets/sphere.obj").expect("sphere OBJ should load");

        assert_eq!(obj.vertices.len(), 98);
        assert_eq!(obj.source_normals.len(), 192);
        assert_eq!(obj.indices.len(), 192 * 3);
        assert!(
            obj.corner_indices
                .iter()
                .all(|&c| (obj.corner_normal(c).length() - 1.0).abs() <= 1e-4)
        );
    }

    #[test]
    fn parses_face_formats_negative_indices_ngons_and_per_corner_normals() {
        let data = "\
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
v 0.5 2 0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vn 0 0 1
vn 0 1 0
# formato v/vt/vn en quad (2 triángulos)
f 1/1/1 2/2/1 3/3/1 4/4/1
# formato v//vn con índices negativos y distinta normal en vértice 1
f -5//-1 -4//-1 -3//-1
# formato v/vt y v puro en pentágono (3 triángulos)
f 1/1 2/2 3 4/4 5
";
        let obj = load_obj_from_reader(std::io::Cursor::new(data.as_bytes()))
            .expect("OBJ sintético debe cargar");

        // 2 tris (quad) + 1 tri + 3 tris (pentágono) = 6 triángulos = 18 esquinas
        assert_eq!(obj.corner_indices.len(), 18);
        assert_eq!(obj.indices.len(), 18);
        assert_eq!(obj.texcoords.len(), 4);
        assert_eq!(obj.source_normals.len(), 2);

        // Quad triangulado en abanico: [0,1,2] y [0,2,3]
        assert_eq!(obj.indices[0..6], [0, 1, 2, 0, 2, 3]);
        assert_eq!(obj.corner_indices[0].tex, Some(0));
        assert_eq!(obj.corner_indices[0].normal, Some(0));

        // Segundo triángulo usa índices negativos (-5 -> 0, -4 -> 1, -3 -> 2, vn -1 -> 1)
        assert_eq!(obj.indices[6..9], [0, 1, 2]);
        assert_eq!(obj.corner_indices[6].tex, None);
        assert_eq!(obj.corner_indices[6].normal, Some(1));
        assert!(
            (obj.corner_normal(obj.corner_indices[0]) - Vector3::new(0.0, 0.0, 1.0)).length()
                < 1e-5
        );
        assert!(
            (obj.corner_normal(obj.corner_indices[6]) - Vector3::new(0.0, 1.0, 0.0)).length()
                < 1e-5
        );

        // Pentágono: 3 triángulos en abanico [0,1,2], [0,2,3], [0,3,4]
        assert_eq!(obj.indices[9..18], [0, 1, 2, 0, 2, 3, 0, 3, 4]);
    }

    #[test]
    fn indexed_mesh_deduplicates_matching_corners_and_loads_assets() {
        let sphere = load_obj("assets/sphere.obj").expect("sphere.obj debe cargar");
        let (s_verts, s_idx) = sphere.to_indexed_mesh();
        assert_eq!(s_idx.len(), 192 * 3);
        assert!(s_verts.len() <= s_idx.len());

        let ship = load_obj("assets/ship.obj").expect("ship.obj debe cargar");
        let (sh_verts, sh_idx) = ship.to_indexed_mesh();
        assert_eq!(sh_idx.len(), 29696 * 3);
        assert!(sh_verts.len() <= sh_idx.len());
        println!(
            "Indexed mesh vertices -> sphere: {} (indices {}), ship: {} (indices {})",
            s_verts.len(),
            s_idx.len(),
            sh_verts.len(),
            sh_idx.len()
        );
    }
}
