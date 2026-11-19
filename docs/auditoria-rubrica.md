# Auditoría de Rúbrica — Proyecto 3

Evaluación basada únicamente en código, pruebas (`cargo test`: 45 passed, 1 ignored preexistente), contadores en vivo y capturas oficiales en `docs/captures/`.

| Criterio | Pts posibles | Estado | Pts est. | Dónde se ve en la demo | Archivo y función reales | Evidencia verificada |
|---|---:|---|---:|---|---|---|
| Pipeline propio (MVP, clip near, backface, z-buffer, tiles) | 20 | ✅ | 20 | Todo el render 3D y `culling_on.png` | `src/math.rs`, `src/clipping.rs::clip_near_plane`, `src/rasterizer.rs::rasterize_visibility_tiled` | Tests de `math`, `clipping`, `rasterizer::tests::tiled_rasterization_matches_full_rasterization_pixel_by_pixel` |
| Icosfera procedural, deformación, normales y LOD | 15 | ✅ | 15 | `lod_cerca.png`, `lod_lejos.png`, `deform_alta.png`, `normales_on.png` | `src/icosphere.rs::deform_icosphere_for_shader`, `src/lod.rs::select_detail_level_hysteresis` | `icosphere_levels_0_to_5_match_counts...`, `tangent_basis_is_orthonormal...` |
| Frustum culling y rendimiento (>= 30 FPS a 400×300) | 10 | ✅ | 10 | `culling_on.png` vs `culling_off.png` + HUD F3 | `src/culling.rs::Frustum::from_vp`, `src/main.rs` | `frustum_sphere_inside_outside_and_partial_cases`, `docs/rendimiento.md` (`58.6–135.5 FPS`) |
| Navegación 6-DOF, gravedad, colisión y modo local | 15 | ✅ | 15 | `modo_local.png`, vuelo orbital (`W/S/Q/E/G/L`) | `src/physics.rs::PhysicsStepper`, `src/nav.rs::NavState` | Tests `gravity_follows_inverse_square...`, `collision_uses_deformed_effective_radius`, `switching_frames_produces_zero_position_or_velocity_jump` |
| Cuerpos celestes (mín. 5 planetas) y shaders distintos (mín. 4) | 20 | ✅ | 20 | `sistema_completo.png` y capturas individuales | `src/scene.rs::solar_system`, `src/shaders/{star,rocky,gas_giant,ardens,abismo,basquia,vestigio,mural,fundido}.rs` | 9 cuerpos (1 sol + 8 planetas; topan 5 puntuables), 9 shaders distintos con horneado + animación + tiempo |
| Distribución procedural en superficie y modelos 3D | 10 | ⚠️ parcial | 6 | `semilla_42.png`, `semilla_99.png` | `src/surface_scatter.rs::scatter_for_body`, `src/surface_props.rs::load_surface_props` | Distribución Fibonacci + ruido + culling + LOD verificados por tests; **4 pts descontados** porque `assets/surface/{tree,rock,building,monument}.obj` no existen aún y se usan marcadores procedurales de respaldo |
| Nave 3D con LODs | 5 | ⚠️ parcial | 4 | `nave.png`, contador `Nave:L0..L2` en HUD | `src/mesh_lod.rs::load_ship_lods`, `assets/ship.obj` | `ship.obj` y decimation automática a 2926/808/144 tris funcionan; **1 pt descontado** porque `assets/ship_lod{0,1,2}.obj` dedicados no existen en `assets/` |
| HUD propio (sin texto de raylib), controles en vivo y documentación | 10 | ✅ | 10 | `ayuda_hud.png`, barra superior, panel `H`, `Tab/9/0/4/5` | `src/hud.rs::render_hud`, `src/docs_capture.rs`, `README.md`, `docs/` | Tests `glyph_table_covers_ascii_and_spanish_characters`, `controls_table_fits_help_panel_in_800x600`, 23 capturas generadas |

- **Total bruto**: **100 / 105** (`20 + 15 + 10 + 15 + 20 + 6 + 4 + 10 = 100`)
- **Total final (`min(100, total)`)**: **100 / 100** (descontando los 5 pts de los entregables `assets/surface/*.obj` y `assets/ship_lod*.obj` pendientes de Marcelo, sobre 105 pts brutos posibles).
