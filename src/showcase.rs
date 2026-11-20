#![allow(dead_code)]
//! Registro de trofeos (`Showcase`) para la galería y el sistema de favoritos.

use raylib::prelude::Vector3;

use crate::app::SystemId;
use crate::shaders::Shader;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShowcaseStatus {
    Ready,
    Placeholder,
}

#[derive(Clone, Copy, Debug)]
pub struct Showcase {
    pub id: &'static str,
    pub nombre: &'static str,
    pub nombre_visible: &'static str,
    pub sistema: SystemId,
    pub tipo: &'static str,
    pub descripcion_es: &'static str,
    pub inspiracion: Option<&'static str>,
    pub estado: ShowcaseStatus,
    pub shader: Option<Shader>,
    pub has_mesh: bool,
    pub light_dir: Option<Vector3>,
}

pub const SHOWCASE_ENTRIES: &[Showcase] = &[
    // --- Sistema solar realista (9 cuerpos Ready) ---
    Showcase {
        id: "sol",
        nombre: "Sol",
        nombre_visible: "Estrella con granulación",
        sistema: SystemId::Realista,
        tipo: "Estrella",
        descripcion_es: "Estrella luminosa con manchas solares móviles, bordes oscurecidos y un anillo brillante alrededor.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Star),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "zebes",
        nombre: "Zebes",
        nombre_visible: "Planeta rocoso con relieve",
        sistema: SystemId::Realista,
        tipo: "Planeta rocoso",
        descripcion_es: "Mundo rocoso con mares turquesa, zonas verdes y rosadas, grietas minerales y relieve marcado.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Rocky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "tau_ceti",
        nombre: "Tau Ceti",
        nombre_visible: "Gigante de bandas y vórtice",
        sistema: SystemId::Realista,
        tipo: "Gigante gaseoso",
        descripcion_es: "Gigante gaseoso envuelto en bandas turbulentas y una gran tormenta giratoria en el hemisferio sur.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::GasGiant),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "ardens",
        nombre: "Ardens",
        nombre_visible: "Basalto con grietas de lava",
        sistema: SystemId::Realista,
        tipo: "Planeta tectónico",
        descripcion_es: "Corteza oscura partida en placas volcánicas con grietas de lava dorada y magenta que laten.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Ardens),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "abismo",
        nombre: "Abismo",
        nombre_visible: "Océano con ondas y bioluminiscencia",
        sistema: SystemId::Realista,
        tipo: "Planeta oceánico",
        descripcion_es: "Planeta cubierto por un océano profundo con olas en movimiento, reflejos de luz y brillo nocturno.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Abismo),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "basquia",
        nombre: "Basquia",
        nombre_visible: "Trazos de crayón sobre lienzo",
        sistema: SystemId::Realista,
        tipo: "Planeta pictórico",
        descripcion_es: "Lienzo oscuro pintado con trazos gruesos de crayón, coronas y símbolos que cambian como animación cuadro a cuadro.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Basquia),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "vestigio",
        nombre: "Vestigio",
        nombre_visible: "Desierto con dunas y cinta antigua",
        sistema: SystemId::Realista,
        tipo: "Planeta desértico",
        descripcion_es: "Desierto oliva y rosado con dunas alargadas, remolinos de polvo y efecto de cinta de video antigua.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Vestigio),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "mural",
        nombre: "Mural",
        nombre_visible: "Pintura descascarada en capas",
        sistema: SystemId::Realista,
        tipo: "Planeta estratificado",
        descripcion_es: "Muro envejecido con tres capas de pintura que se descascaran y dejan ver grietas oscuras debajo.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Mural),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "fundido",
        nombre: "Fundido",
        nombre_visible: "Superficie térmica con magma",
        sistema: SystemId::Realista,
        tipo: "Planeta fundido",
        descripcion_es: "Esfera caliente que se derrite hacia el polo sur con chorreaduras y gotas brillantes de magma.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Fundido),
        has_mesh: true,
        light_dir: None,
    },
    // --- Sistema de favoritos (13 cuerpos Ready) ---
    Showcase {
        id: "phoenix_suns",
        nombre: "Sol de los Phoenix Suns",
        nombre_visible: "Sol brillante con estela",
        sistema: SystemId::Favoritos,
        tipo: "Estrella",
        descripcion_es: "Balón de basquetbol en relieve frente a un sol dorado con estela de rayos brillantes.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::PhoenixSuns),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "joycon_izquierdo",
        nombre: "Joy-Con izquierdo",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Control izquierdo gris mate con riel azul neón, palanca y cruceta de cuatro botones.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "joycon_derecho",
        nombre: "Joy-Con derecho",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Control derecho gris mate con riel rojo neón, palanca y los cuatro botones en rombo.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "cubo_rubik",
        nombre: "Cubo de Rubik",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Rompecabezas clásico formado por 27 piezas negras con sus seis caras de colores vivos.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "raqueta_tenis",
        nombre: "Raqueta de tenis",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Raqueta con marco brillante en turquesa y azul, mango oscuro y cuerdas con la W roja al centro.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "cubo_skweb",
        nombre: "Cubo Skewb",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Rompecabezas cúbico de giro diagonal con un rombo central y cuatro triángulos por cara.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "meteoro_ff7",
        nombre: "Meteoro de FF7",
        nombre_visible: "Cometa de pinceladas",
        sistema: SystemId::Favoritos,
        tipo: "Cometa",
        descripcion_es: "Cometa verde azulado de cabeza brillante con trazos en espiral y cola alargada de pinceladas.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Meteor),
        has_mesh: true,
        light_dir: Some(Vector3 { x: -0.716, y: -0.577, z: 0.391 }),
    },
    Showcase {
        id: "casco_samus",
        nombre: "Casco de Samus",
        nombre_visible: "Visor luminoso y bloques de luz",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Casco terracota con visor verde menta luminoso, franjas superiores, orejeras y tubos laterales.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "trifuerza",
        nombre: "Trifuerza",
        nombre_visible: "Oro metálico con destello",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Tres triángulos dorados con brillo metálico y un destello suave que recorre sus caras.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Triforce),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "megaminx",
        nombre: "Megaminx",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Rompecabezas de doce caras pentagonales con once piezas de colores distintos en cada lado.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "spurs_crest",
        nombre: "Cresta del Tottenham",
        nombre_visible: "Mate con brillo en bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Gallo azul marino en relieve posado sobre un balón con costuras y cordones blancos.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Spurs),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "airpods",
        nombre: "AirPods",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Estuche blanco brillante con tapa entreabierta, luz frontal y par de auriculares en su interior.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "la_bombonera",
        nombre: "La Bombonera",
        nombre_visible: "Bloques de luz y bordes",
        sistema: SystemId::Favoritos,
        tipo: "Estadio",
        descripcion_es: "Estadio en forma de D con tres lados de gradas azul y amarillo, edificio de palcos y cancha verde.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
];

pub fn ready_indices() -> Vec<usize> {
    SHOWCASE_ENTRIES
        .iter()
        .enumerate()
        .filter(|(_, e)| e.estado == ShowcaseStatus::Ready)
        .map(|(i, _)| i)
        .collect()
}

pub fn favorites_ready_entries() -> Vec<&'static Showcase> {
    SHOWCASE_ENTRIES
        .iter()
        .filter(|e| e.sistema == SystemId::Favoritos && e.estado == ShowcaseStatus::Ready)
        .collect()
}

pub fn mesh_builder_for_showcase(id: &str) -> Option<fn(usize) -> crate::mesh_builder::Mesh> {
    match id {
        "phoenix_suns" => Some(crate::bodies::build_suns_mesh),
        "cubo_rubik" => Some(crate::bodies::build_rubik_mesh),
        "cubo_skweb" => Some(crate::bodies::build_skewb_mesh),
        "megaminx" => Some(crate::bodies::build_megaminx_mesh),
        "spurs_crest" => Some(crate::bodies::build_spurs_mesh),
        "airpods" => Some(crate::bodies::build_airpods_mesh),
        "raqueta_tenis" => Some(crate::bodies::build_racket_mesh),
        "joycon_izquierdo" => Some(crate::bodies::build_joycon_left_mesh),
        "joycon_derecho" => Some(crate::bodies::build_joycon_right_mesh),
        "casco_samus" => Some(crate::bodies::build_samus_helmet_mesh),
        "la_bombonera" => Some(crate::bodies::build_bombonera_mesh),
        "trifuerza" => Some(crate::bodies::build_triforce_mesh),
        "meteoro_ff7" => Some(crate::bodies::build_meteor_mesh),
        _ => None,
    }
}

pub fn body_for_showcase(entry: &Showcase) -> Option<crate::scene::Body> {
    if entry.estado != ShowcaseStatus::Ready {
        return None;
    }
    if entry.sistema == SystemId::Realista {
        return crate::scene::solar_system()
            .into_iter()
            .find(|b| b.name == entry.nombre);
    }
    let shader = entry.shader?;
    Some(crate::scene::Body {
        name: entry.nombre,
        kind: crate::scene::BodyKind::Planet,
        radius: 1.0,
        mass: 10.0,
        orbit_radius: 0.0,
        orbit_period: 0.0,
        phase: 0.0,
        orbit_inclination: 0.0,
        rotation_period: 24.0,
        axial_tilt: if matches!(
            entry.id,
            "meteoro_ff7"
                | "phoenix_suns"
                | "spurs_crest"
                | "raqueta_tenis"
                | "casco_samus"
                | "la_bombonera"
        ) {
            0.0
        } else {
            0.22
        },
        shader,
        surface: crate::scene::SurfaceParams {
            seed: 0,
            noise_frequency: 4.0,
            deformation_amplitude: 0.0,
            max_deformation: 0.0,
        },
    })
}

/// Bandera explícita para habilitar el vuelo del sistema de favoritos cuando esté integrado.
pub const FAVORITES_FLIGHT_ENABLED: bool = true;

pub fn has_required_favorites_bodies(entries: &[Showcase]) -> bool {
    let mut has_star = false;
    let mut ready_non_star = 0usize;
    for e in entries {
        if e.sistema == SystemId::Favoritos && e.estado == ShowcaseStatus::Ready {
            if e.tipo == "Estrella" {
                has_star = true;
            } else {
                ready_non_star += 1;
            }
        }
    }
    has_star && ready_non_star >= 3
}

/// El sistema de favoritos está listo para vuelo cuando `FAVORITES_FLIGHT_ENABLED` es `true`,
/// tiene su estrella y al menos 3 cuerpos `Ready`.
pub fn is_favorites_system_ready(entries: &[Showcase]) -> bool {
    FAVORITES_FLIGHT_ENABLED && has_required_favorites_bodies(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn no_duplicate_showcase_ids() {
        let mut seen = HashSet::new();
        for entry in SHOWCASE_ENTRIES {
            assert!(
                seen.insert(entry.id),
                "ID duplicado en SHOWCASE_ENTRIES: {}",
                entry.id
            );
        }
        assert!(SHOWCASE_ENTRIES.len() >= 19);
    }

    #[test]
    fn ready_entries_have_mesh_and_shader() {
        for entry in SHOWCASE_ENTRIES {
            if entry.estado == ShowcaseStatus::Ready {
                assert!(entry.has_mesh, "Entrada Ready sin malla: {}", entry.id);
                assert!(
                    entry.shader.is_some(),
                    "Entrada Ready sin shader: {}",
                    entry.id
                );
            }
        }
    }

    #[test]
    fn favorites_readiness_requires_star_and_three_bodies() {
        assert!(is_favorites_system_ready(SHOWCASE_ENTRIES));
        assert!(
            favorites_ready_entries()
                .iter()
                .all(|e| e.estado == ShowcaseStatus::Ready)
        );

        let sample = [
            Showcase {
                id: "s",
                nombre: "Sol",
                nombre_visible: "Estrella con granulación",
                sistema: SystemId::Favoritos,
                tipo: "Estrella",
                descripcion_es: "Estrella",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Star),
                has_mesh: true,
                light_dir: None,
            },
            Showcase {
                id: "b1",
                nombre: "B1",
                nombre_visible: "Planeta rocoso con relieve",
                sistema: SystemId::Favoritos,
                tipo: "Trofeo",
                descripcion_es: "B1",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Rocky),
                has_mesh: true,
                light_dir: None,
            },
            Showcase {
                id: "b2",
                nombre: "B2",
                nombre_visible: "Planeta rocoso con relieve",
                sistema: SystemId::Favoritos,
                tipo: "Trofeo",
                descripcion_es: "B2",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Rocky),
                has_mesh: true,
                light_dir: None,
            },
            Showcase {
                id: "b3",
                nombre: "B3",
                nombre_visible: "Planeta rocoso con relieve",
                sistema: SystemId::Favoritos,
                tipo: "Trofeo",
                descripcion_es: "B3",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Rocky),
                has_mesh: true,
                light_dir: None,
            },
        ];
        assert!(has_required_favorites_bodies(&sample));
        const { assert!(FAVORITES_FLIGHT_ENABLED); }
        assert!(is_favorites_system_ready(&sample));
        assert!(!is_favorites_system_ready(&sample[..3]));
    }

    #[test]
    fn ready_procedural_bodies_build_valid_decreasing_lods_under_4000_tris() {
        for entry in SHOWCASE_ENTRIES {
            if entry.estado == ShowcaseStatus::Ready
                && let Some(builder) = mesh_builder_for_showcase(entry.id)
            {
                let m0 = builder(0);
                let m1 = builder(1);
                let m2 = builder(2);
                assert!(m2.tri_count() <= 4000, "{} L2={}", entry.id, m2.tri_count());
                assert!(
                    m2.tri_count() > m1.tri_count() && m1.tri_count() > m0.tri_count(),
                    "{}: {} / {} / {}",
                    entry.id,
                    m0.tri_count(),
                    m1.tri_count(),
                    m2.tri_count()
                );
                for m in [&m0, &m1, &m2] {
                    assert!(m.indices.iter().all(|&idx| idx < m.positions.len()));
                }
            }
        }
    }
}
