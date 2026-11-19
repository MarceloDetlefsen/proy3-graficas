use raylib::prelude::Vector3;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub uv: [f32; 2],
    pub world_position: Vector3,
    pub transformed_position: Vector3,
    pub transformed_normal: Vector3,
    pub clip_w: f32,
}

impl Vertex {
    #[allow(dead_code)]
    pub fn new(position: Vector3, normal: Vector3) -> Self {
        Self::with_uv(position, normal, [0.0, 0.0])
    }

    pub fn with_uv(position: Vector3, normal: Vector3, uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            tangent: Vector3::new(1.0, 0.0, 0.0),
            bitangent: Vector3::new(0.0, 1.0, 0.0),
            uv,
            world_position: position,
            transformed_position: position,
            transformed_normal: normal,
            clip_w: 1.0,
        }
    }
}
