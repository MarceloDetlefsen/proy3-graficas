#![allow(dead_code)]
//! Registro de trofeos (`Showcase`) para la galería y el sistema de favoritos.

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
    pub sistema: SystemId,
    pub tipo: &'static str,
    pub descripcion_es: &'static str,
    pub inspiracion: Option<&'static str>,
    pub estado: ShowcaseStatus,
    pub shader: Option<Shader>,
    pub has_mesh: bool,
}

pub const SHOWCASE_ENTRIES: &[Showcase] = &[
    // --- Sistema solar realista (9 cuerpos Ready) ---
    Showcase {
        id: "sol",
        nombre: "Sol",
        sistema: SystemId::Realista,
        tipo: "Estrella",
        descripcion_es: "Oscurecimiento al limbo, granulacion fbm animada, manchas solares y anillo de corona pulsante.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Star),
        has_mesh: true,
    },
    Showcase {
        id: "zebes",
        nombre: "Zebes",
        sistema: SystemId::Realista,
        tipo: "Planeta rocoso",
        descripcion_es: "Vetas anisotropicas con domain warp, bioma verde-rosa, grietas y normales horneadas.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Rocky),
        has_mesh: true,
    },
    Showcase {
        id: "tau_ceti",
        nombre: "Tau Ceti",
        sistema: SystemId::Realista,
        tipo: "Gigante gaseoso",
        descripcion_es: "Bandas latitudinales con triple domain warp animado y gran vortice anticiclonico.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::GasGiant),
        has_mesh: true,
    },
    Showcase {
        id: "ardens",
        nombre: "Ardens",
        sistema: SystemId::Realista,
        tipo: "Planeta tectonico",
        descripcion_es: "Placas tectonicas Voronoi 3D sobre basalto con grietas de lava pulsante animadas.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Ardens),
        has_mesh: true,
    },
    Showcase {
        id: "abismo",
        nombre: "Abismo",
        sistema: SystemId::Realista,
        tipo: "Planeta oceanico",
        descripcion_es: "Cuatro ondas direccionales animadas, causticas Voronoi 3D y bioluminiscencia.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Abismo),
        has_mesh: true,
    },
    Showcase {
        id: "basquia",
        nombre: "Basquia",
        sistema: SystemId::Realista,
        tipo: "Planeta pictorico",
        descripcion_es: "Trazos de oilstick, coronas, conteos y flechas con animacion stop-motion sobre lienzo.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Basquia),
        has_mesh: true,
    },
    Showcase {
        id: "vestigio",
        nombre: "Vestigio",
        sistema: SystemId::Realista,
        tipo: "Planeta desertico",
        descripcion_es: "Dunas anisotropicas estiradas, remolinos de polvo animados y grano de pelicula.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Vestigio),
        has_mesh: true,
    },
    Showcase {
        id: "mural",
        nombre: "Mural",
        sistema: SystemId::Realista,
        tipo: "Planeta estratificado",
        descripcion_es: "Tres capas de pintura descascarada por umbral fbm, grietas ridge y escamas animadas.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Mural),
        has_mesh: true,
    },
    Showcase {
        id: "fundido",
        nombre: "Fundido",
        sistema: SystemId::Realista,
        tipo: "Planeta fundido",
        descripcion_es: "Chorreaduras estiradas hacia el polo sur, charco de brea y gotas descendentes animadas.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Fundido),
        has_mesh: true,
    },
    // --- Sistema de favoritos (10 cuerpos Placeholder) ---
    Showcase {
        id: "phoenix_suns",
        nombre: "Sol de los Phoenix Suns",
        sistema: SystemId::Favoritos,
        tipo: "Estrella",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "joycon_izquierdo",
        nombre: "Joy-Con izquierdo",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "joycon_derecho",
        nombre: "Joy-Con derecho",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "cubo_rubik",
        nombre: "Cubo de Rubik",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "27 cubitos negros con holgura y calcomanias por cara; shader chunky en 3 bandas con borde Fresnel.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
    },
    Showcase {
        id: "raqueta_tenis",
        nombre: "Raqueta de tenis",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "cubo_skweb",
        nombre: "Cubo Skweb",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "asteroide_ff7",
        nombre: "Asteroide de FF7",
        sistema: SystemId::Favoritos,
        tipo: "Asteroide",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "casco_samus",
        nombre: "Casco de Samus",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "trifuerza",
        nombre: "Trifuerza",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
    },
    Showcase {
        id: "la_bombonera",
        nombre: "La Bombonera",
        sistema: SystemId::Favoritos,
        tipo: "Estadio",
        descripcion_es: "Trofeo pendiente de modelado y shader.",
        inspiracion: None,
        estado: ShowcaseStatus::Placeholder,
        shader: None,
        has_mesh: false,
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
        "cubo_rubik" => Some(crate::bodies::build_rubik_mesh),
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
        axial_tilt: 0.22,
        shader,
        surface: crate::scene::SurfaceParams {
            seed: 0,
            noise_frequency: 4.0,
            deformation_amplitude: 0.0,
            max_deformation: 0.0,
        },
    })
}

/// El sistema de favoritos está listo para vuelo cuando tiene su estrella
/// ("Sol de los Phoenix Suns" / tipo "Estrella") y al menos 3 cuerpos `Ready`.
pub fn is_favorites_system_ready(entries: &[Showcase]) -> bool {
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
        assert_eq!(SHOWCASE_ENTRIES.len(), 19);
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
        assert!(!is_favorites_system_ready(SHOWCASE_ENTRIES));
        assert!(
            favorites_ready_entries()
                .iter()
                .all(|e| e.estado == ShowcaseStatus::Ready)
        );

        let sample = [
            Showcase {
                id: "s",
                nombre: "Sol",
                sistema: SystemId::Favoritos,
                tipo: "Estrella",
                descripcion_es: "Estrella",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Star),
                has_mesh: true,
            },
            Showcase {
                id: "b1",
                nombre: "B1",
                sistema: SystemId::Favoritos,
                tipo: "Trofeo",
                descripcion_es: "B1",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Rocky),
                has_mesh: true,
            },
            Showcase {
                id: "b2",
                nombre: "B2",
                sistema: SystemId::Favoritos,
                tipo: "Trofeo",
                descripcion_es: "B2",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Rocky),
                has_mesh: true,
            },
            Showcase {
                id: "b3",
                nombre: "B3",
                sistema: SystemId::Favoritos,
                tipo: "Trofeo",
                descripcion_es: "B3",
                inspiracion: None,
                estado: ShowcaseStatus::Ready,
                shader: Some(Shader::Rocky),
                has_mesh: true,
            },
        ];
        assert!(is_favorites_system_ready(&sample));
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
