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
        sistema: SystemId::Realista,
        tipo: "Estrella",
        descripcion_es: "Oscurecimiento al limbo, granulacion fbm animada, manchas solares y anillo de corona pulsante.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Star),
        has_mesh: true,
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
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
        light_dir: None,
    },
    // --- Sistema de favoritos (10 cuerpos Placeholder) ---
    Showcase {
        id: "phoenix_suns",
        nombre: "Sol de los Phoenix Suns",
        sistema: SystemId::Favoritos,
        tipo: "Estrella",
        descripcion_es: "Balon de basquetbol 3D con costuras giratorias delante de una explosion solar emisiva extruida con estela de rayos.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::PhoenixSuns),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "joycon_izquierdo",
        nombre: "Joy-Con izquierdo",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Carcasa gris mate con riel azul neon, palanca, cruceta de 4 botones, boton menos, captura y SL/SR; shader chunky.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "joycon_derecho",
        nombre: "Joy-Con derecho",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Carcasa gris mate con riel rojo neon, palanca, botones ABXY en rombo, boton mas, Home y SL/SR; shader chunky.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
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
        light_dir: None,
    },
    Showcase {
        id: "raqueta_tenis",
        nombre: "Raqueta de tenis",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Marco Wilson Ultra medido en turquesa, azul y marino con barniz, mango octogonal y cuerdas de asteroides 11x13 con la W roja.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "cubo_skweb",
        nombre: "Cubo Skewb",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Cubo negro con rombo central y cuatro triangulos de esquina por cara; shader chunky en 3 bandas.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "meteoro_ff7",
        nombre: "Meteoro de FF7",
        sistema: SystemId::Favoritos,
        tipo: "Cometa",
        descripcion_es: "Cabeza vitrea con trazos blancos en espiral horneados, cola de pinceladas con degradado por vertice y orbe.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Meteor),
        has_mesh: true,
        light_dir: Some(Vector3 { x: -0.716, y: -0.577, z: 0.391 }),
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
        light_dir: None,
    },
    Showcase {
        id: "trifuerza",
        nombre: "Trifuerza",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Tres prismas triangulares equilateros con centro vacio; shader dorado metalico con destello animado.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Triforce),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "megaminx",
        nombre: "Megaminx",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Dodecaedro regular negro con 11 pegatinas por cara (pentagono, 5 aristas y 5 cometas) en 12 colores; shader chunky.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "spurs_crest",
        nombre: "Cresta del Tottenham",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Gallo extruido con bisel desde silueta medida sobre balon azul marino con costuras y cordones blancos; giro +-35.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Spurs),
        has_mesh: true,
        light_dir: None,
    },
    Showcase {
        id: "airpods",
        nombre: "AirPods",
        sistema: SystemId::Favoritos,
        tipo: "Trofeo",
        descripcion_es: "Estuche blanco brillante con tapa entreabierta, bisagra y LED frontal, con par de auriculares espejados; shader chunky.",
        inspiracion: None,
        estado: ShowcaseStatus::Ready,
        shader: Some(Shader::Chunky),
        has_mesh: true,
        light_dir: None,
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
        axial_tilt: if matches!(entry.id, "meteoro_ff7" | "phoenix_suns" | "spurs_crest" | "raqueta_tenis") {
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
pub const FAVORITES_FLIGHT_ENABLED: bool = false;

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
                light_dir: None,
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
                light_dir: None,
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
                light_dir: None,
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
                light_dir: None,
            },
        ];
        assert!(has_required_favorites_bodies(&sample));
        const { assert!(!FAVORITES_FLIGHT_ENABLED); }
        assert!(!is_favorites_system_ready(&sample));
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
