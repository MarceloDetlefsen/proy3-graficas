use raylib::math::{Vector2, Vector3};

pub struct Fragment {
    #[allow(dead_code)]
    pub position: Vector2,
    pub object_pos: Vector3,
    pub world_pos: Vector3,
    pub normal: Vector3,
    #[allow(dead_code)]
    pub color: Vector3,
    pub depth: f32,
}
