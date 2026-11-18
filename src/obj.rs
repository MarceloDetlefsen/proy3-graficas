#![allow(dead_code)]

use raylib::prelude::Vector3;
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
                let mut face = Vec::new();
                for part in parts {
                    if let Some(corner) = parse_face_corner(
                        part,
                        vertices.len(),
                        texcoords.len(),
                        source_normals.len(),
                    ) {
                        indices.push(corner.pos);
                        corner_indices.push(corner);
                        face.push(corner.pos);
                    }
                }
                if face.len() >= 3 {
                    faces.push(face);
                }
            }
            _ => {}
        }
    }

    let centroid = vertices
        .iter()
        .copied()
        .fold(Vector3::zero(), |sum, position| sum + position)
        * (1.0 / vertices.len().max(1) as f32);
    let centered_at_origin = centroid.length() <= 1e-5;
    let vertex_normals = if centered_at_origin {
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
    use super::load_obj;

    #[test]
    fn sphere_counts_and_smooth_normals_are_valid() {
        let obj = load_obj("assets/sphere.obj").expect("sphere OBJ should load");

        assert_eq!(obj.vertices.len(), 98);
        assert_eq!(obj.source_normals.len(), 192);
        assert_eq!(obj.indices.len(), 192 * 3);
        assert_eq!(obj.vertex_normals.len(), obj.vertices.len());
        assert!(
            obj.vertex_normals
                .iter()
                .all(|normal| (normal.length() - 1.0).abs() <= 1e-4)
        );
    }
}
