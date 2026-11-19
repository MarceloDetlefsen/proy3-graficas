# Guía de Defensa Técnica

## Tabla de Parámetros Modificables

| Parámetro | Archivo | Qué cambia visualmente | Tecla en vivo |
|---|---|---|---|
| Radio del cuerpo seleccionado (`radius`) | `src/scene.rs` (`Body::radius`) | Escala el cuerpo y recalcula la posición de sus instancias en superficie | `Tab` (seleccionar) + `9` / `0` |
| Parámetro en vivo del shader (`shader_param`) | `src/shaders.rs` (`live_param`) | Modula contraste, brillo de corona, pulso o intensidad del shader activo | `Tab` (seleccionar) + `4` / `5` |
| Amplitud de deformación (`deformation_amplitude`) | `src/scene.rs` (`SurfaceParams`) | Aumenta o aplana el relieve geométrico de `Zebes` (`Rocky`) | `,` / `.` |
| Frecuencia de deformación (`noise_frequency`) | `src/main.rs` / `src/surface_map.rs` | Cambia la escala de montañas/cráteres y re-hornea los mapas estáticos | `[` / `]` |
| Semilla de distribución (`surface_seed`) | `src/main.rs` / `src/surface_scatter.rs` | Cambia la ubicación y densidad de árboles, rocas, edificios y monumentos | `I` / `O` |
| Escala de tiempo / velocidad orbital (`sim_clock.scale`) | `src/sim.rs` / `src/scene_config.rs` | Acelera o frena las órbitas, rotaciones y animaciones de shaders | `-` / `=` (`Enter` pausa) |
| Constante gravitacional (`GRAVITY_G = 7.2`) | `src/physics.rs` | Intensidad de la atracción gravitacional (y activación en vuelo) | `G` (ON/OFF en vivo; valor en código) |
| Resolución interna y número de hilos | `src/main.rs` (`res_scales`, `thread_options`) | Cambia entre `400×300`, `266×200`, `800×600` y paralelismo de tiles | `R` / `T` |

## Explicación por Criterio (3 a 5 líneas)

### 1. Pipeline propio (MVP, recorte, culling y tiles)
Toda la transformación `M = T·R·S`, vista `V` y proyección perspectiva `P` con vectores columna vive en `src/math.rs`. Antes de proyectar, `clip_near_plane` en `src/clipping.rs` recorta triángulos contra `z_clip + w >= 0` interpolando atributos. El rasterizador (`src/rasterizer.rs`) divide la pantalla en tiles de `64×64` procesados en paralelo con `rayon` y usa un buffer de visibilidad (`tri_id_buffer` + `depth_buffer`) para sombrear únicamente los píxeles visibles.

### 2. Icosfera procedural, deformación, normales y LOD
En `src/icosphere.rs`, un icosaedro base se subdivide recursivamente con caché de puntos medios (niveles `0..=5`) y cada nivel se deforma radialmente de forma perezosa usando la misma función `height_at` que hornea los mapas estáticos. Las normales suaves se recalculan tras deformar y `Rocky` las perturba en espacio tangente (`spherical_tangent_basis`). En `src/lod.rs`, `select_detail_level_hysteresis` elige el nivel según el radio proyectado en píxeles evitando oscilaciones.

### 3. Navegación 6-DOF, gravedad, colisión y modo local
`PhysicsStepper` (`src/physics.rs`) ejecuta un acumulador de paso fijo (`1/120 s`, máx. 8 subpasos) para que la trayectoria sea independiente del FPS. La gravedad suma la atracción de todos los cuerpos con suavizado cerca del centro y la colisión usa `effective_radius(body)` (radio base + deformación máxima). `NavState` (`src/nav.rs`) cambia entre marco global y local con histéresis (`2.5` y `3.0` radios) sin saltos de posición ni velocidad.

### 4. Planetas y shaders procedurales (capa estática + viva)
El sistema solar (`src/scene.rs`, `src/scene_config.rs`) contiene al Sol (`Star`) y 8 planetas con shaders propios y técnica distinta: `Rocky` (fbm + biomas + nubes + normal mapping), `GasGiant` (bandas + vórtice), `Ardens` (Voronoi tectónico + lava), `Abismo` (ondas 3D + cáusticas Voronoi + maelstrom + bioluminiscencia), `Basquia` (trazos *oilstick* por rejilla 3D a 6 FPS), `Vestigio` (dunas anisotrópicas + VHS), `Mural` (estratigrafía fbm 3D + escamas) y `Fundido` (bandas/derretimiento). Todos respetan la arquitectura de capa estática horneada (`512×256`) + capa animada por `time` + atenuación solar y halo atmosférico.

### 5. Distribución procedural en superficie y HUD propio
`scatter_for_body` (`src/surface_scatter.rs`) genera puntos sobre la esfera mediante espiral de Fibonacci con perturbación reproducible (`SplitMix64` en `src/rng.rs`) y los filtra con ruido procedural, orientando cada instancia según la normal deformada y descartando por frustum y distancia (`MAX_PROP_DISTANCE = 32.0`). El HUD (`src/hud.rs`) dibuja texto con una fuente bitmap `5×7` propia (ASCII + español) sobre un `OverlayBuffer` RGBA compuesto en `Framebuffer::draw_with_overlay`, sin usar `draw_text` de raylib.
