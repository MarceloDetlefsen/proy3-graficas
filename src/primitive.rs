use crate::vertex::Vertex;

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
