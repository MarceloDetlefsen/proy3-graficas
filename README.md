# Proyecto 3 — Space Travel

[![CI](https://github.com/MarceloDetlefsen/proy3-graficas/actions/workflows/ci.yml/badge.svg)](https://github.com/MarceloDetlefsen/proy3-graficas/actions/workflows/ci.yml)

Renderizador 3D por software escrito en Rust que simula un sistema solar navegable con 9 cuerpos celestes (1 estrella y 8 planetas con shaders procedurales propios). Implementa pipeline `MVP` propio en mano derecha, recorte en espacio de clip contra el plano cercano, *frustum culling* por esferas envolventes, *backface culling*, rasterizador por *tiles* de `64×64` con buffer de visibilidad y paralelismo mediante `std::thread::scope`, icosferas procedurales con deformación radial y *LOD* con histéresis, mapeo de normales en espacio tangente, distribución procedural sobre la superficie con espiral de Fibonacci y `SplitMix64`, física a paso fijo (`1/120 s`) con gravedad y modo local/global, y un HUD RGBA propio con fuente bitmap `5×7`.

![Sistema Solar completo](docs/captures/sistema_completo.png)

## Demostración

<<COMPLETAR: enlace al video>>

## Requisitos

- **Rust** (edición 2024, `cargo` y `rustc` estables).
- Dependencias de sistema para **Raylib** en Linux (Ubuntu/Debian):
  `cmake`, `libasound2-dev`, `libx11-dev`, `libxrandr-dev`, `libxi-dev`, `libgl1-mesa-dev`, `libglu1-mesa-dev`, `libxcursor-dev`, `libxinerama-dev`.
- Ejecutar siempre con `--release` (`opt-level = 3`, `lto = true`, `codegen-units = 1`) para mantener el rendimiento por encima de 30 FPS.

## Compilación y ejecución

```bash
# Ejecutar el simulador interactivo (resolución interna por defecto 400x300, ventana 800x600, hilos automáticos)
cargo run --release

# Ejecutar la suite de pruebas unitarias
cargo test

# Generar las 23 capturas oficiales en docs/captures/ (sin ventana)
cargo run --release -- --docs
```

## Controles

Tabla generada desde `CONTROLS` (`src/hud.rs`):

| Acción | Tecla |
|---|---|
| Empuje adelante / atrás | `W / S` |
| Impulso (boost) | `Shift` |
| Frenado progresivo | `Espacio` |
| Cabeceo y guiñada (cámara en Libre) | `Mouse/Flechas` |
| Alabeo (roll) | `Q / E` |
| Cámara Seguir/Libre / Recentrar | `F / Backspace` |
| Modo control (Auto/Local/Global) / Gravedad | `L / G` |
| Pausar simulación / Escala de tiempo | `Enter / - =` |
| Seleccionar cuerpo siguiente | `Tab` |
| Ajustar radio del cuerpo seleccionado | `9 / 0` |
| Ajustar parámetro en vivo del shader | `4 / 5` |
| Amplitud / Frecuencia de deformación | `, . / [ ]` |
| Cambiar semilla de distribución | `I / O` |
| Forzar LOD / Frustum culling / Wireframe | `Z / X / M` |
| Normal map / Debug normales / Cull instancias | `B / Y / /` |
| Métricas / Resolución interna / Hilos | `F3 / R / T` |
| Backface culling / Re-hornear / Captura PNG | `C / K / P` |
| Galería: cambiar cuerpo / reiniciar vista | `A / D / R` |
| Volver al menú (en menú: salir) | `Esc` |
| Mostrar u ocultar este panel de ayuda | `H` |

## Arquitectura o pipeline

```mermaid
flowchart LR
    A["Entrada y Reloj (SimClock)"] --> B["Física paso fijo 1/120s y Navegación"]
    B --> C["Escena: Frustum Culling y LOD"]
    C --> D["Vértices: MVP y Clip Near"]
    D --> E["Rasterización por Tiles 64x64 (Visibilidad)"]
    E --> F["Sombreado: Mapas horneados y Normales"]
    E --> G["Cielo estelar y Halo solar"]
    F --> H["Framebuffer interno + Escalado + HUD RGBA"]
    G --> H
    H --> I["Raylib (solo presentación)"]
```

- **Buffer de visibilidad por tiles (`TILE_SIZE = 64`)**: primero se rasterizan todos los triángulos guardando `depth_buffer` y `tri_id_buffer`; el sombreado se ejecuta una sola vez por píxel visible, eliminando el *overdraw* de 2×.
- **Mapas estáticos horneados (`512×256`) + capa animada**: el relieve, biomas y gradientes estáticos se precalculan en mapas equirectangulares y en vivo solo se evalúan animaciones por `time`, atenuación solar y brillo atmosférico.
- **Raylib reducido a ventana, entrada y presentación**: ni la geometría, ni la iluminación, ni el texto del HUD usan funciones de dibujo 3D/texto de Raylib.

Más detalle: [docs/arquitectura.md](docs/arquitectura.md).

## Cuerpos del sistema solar (9)

| | | |
|:---:|:---:|:---:|
| ![Sol](docs/captures/sol.png)<br>**Sol** (`Star`)<br>Estrella emisiva con granulación de manchas, corona animada, sin auto-sombra y halo solar en píxeles vacíos. | ![Zebes](docs/captures/zebes.png)<br>**Zebes** (`Rocky`)<br>Planeta rocoso con mar turquesa, biomas, grietas minerales, nubes vivas, relieve deformado y *normal mapping* tangente. | ![Tau Ceti](docs/captures/tau_ceti.png)<br>**Tau Ceti** (`GasGiant`)<br>Gigante gaseoso con bandas latitudinales turbulentas y gran vórtice ciclónico animado en el hemisferio sur. |
| ![Ardens](docs/captures/ardens.png)<br>**Ardens** (`Ardens`)<br>Placas basálticas Voronoi con relieve direccional y grietas de lava magenta-dorada que laten con géiseres emisivos. | ![Abismo](docs/captures/abismo.png)<br>**Abismo** (`Abismo`)<br>Océano 3D con 4 ondas direccionales, cáusticas Voronoi dobles, *maelstrom* polar de 4 brazos y bioluminiscencia nocturna. | ![Basquia](docs/captures/basquia.png)<br>**Basquia** (`Basquia`)<br>Lienzo negro con trazos gruesos de *oilstick*/crayón por rejilla 3D, coronas de 3 puntas y conteos animados a ~6 FPS. |
| ![Vestigio](docs/captures/vestigio.png)<br>**Vestigio** (`Vestigio`)<br>Desierto oliva-rosa con dunas anisotrópicas 8×, 4 *dust devils* con estela, desfase cromático VHS, *scanlines* y grano. | ![Mural](docs/captures/mural.png)<br>**Mural** (`Mural`)<br>Muro de pintura descascarada en 3 capas `fbm` 3D (gris azulado, crema, rojo), grietas *ridge*, tinta y escamas animadas. | ![Fundido](docs/captures/fundido.png)<br>**Fundido** (`Fundido`)<br>Superficie en transición térmica con mitad superior salmón y bandas onduladas de magma naranja en el hemisferio sur. |

## Componentes principales

### Motor MVP y recorte
Construye las transformaciones de modelo (`T·R·S`), cámara (`look_at_rh`) y proyección perspectiva (`perspective_rh`) en mano derecha mirando hacia `-Z`, y recorta triángulos contra el plano cercano (`clip_near_plane`) antes de dividir por `w`.
- **Variable en código**: `fov_y = 45°`, `near = 0.1`, `far = 400.0` (`src/camera.rs`, `src/main.rs`) y `DEGENERATE_EPS = 1e-6` (`src/rasterizer.rs`).

![Culling activado](docs/captures/culling_on.png)

### Rasterizador y rendimiento
Divide la pantalla en *tiles* de `64×64` procesados en paralelo con `std::thread::scope` y utiliza un buffer de visibilidad (`depth_buffer` + `tri_id_buffer`) para sombrear una sola vez por píxel visible, descartando esferas fuera del frustum antes de ensamblar triángulos.
- **Variable en código**: `TILE_SIZE = 64` (`src/rasterizer.rs`) y `res_scales = [2, 3, 1]` (`src/main.rs`, tecla `R`).

| Culling ON | Culling OFF |
|:---:|:---:|
| ![Culling ON](docs/captures/culling_on.png) | ![Culling OFF](docs/captures/culling_off.png) |

### Navegación
Integra movimiento 6-DOF y gravedad a paso fijo (`FIXED_DT`) independiente del FPS, usando colisión contra el radio efectivo deformado y cambio automático con histéresis entre marco global y marco local del planeta más cercano.
- **Variable en código**: `FIXED_DT = 1.0 / 120.0`, `GRAVITY_G = 7.2` (`src/physics.rs`) y `LOCAL_ENTER_RADII = 2.5` / `LOCAL_EXIT_RADII = 3.0` (`src/nav.rs`).

| Modo Local | Nave 3D |
|:---:|:---:|
| ![Modo Local](docs/captures/modo_local.png) | ![Nave](docs/captures/nave.png) |

### Planetas y shaders
Orquesta los 9 cuerpos celestes con configuración centralizada (escalado de órbitas para un recorrido de ~1 min, atenuación solar con ambiente mínimo, halo solar en píxeles vacíos y brillo atmosférico Fresnel por planeta) y parámetro en vivo por shader.
- **Variable en código**: `ORBIT_DISTANCE_SCALE = 0.65`, `ORBIT_SPEED_SCALE = 1.15`, `MIN_AMBIENT = 0.14`, `SOLAR_ATTENUATION_K = 0.00055` (`src/scene_config.rs`) y `live_param` (`src/shaders.rs`, teclas `4`/`5`).

![Sistema completo](docs/captures/sistema_completo.png)

### Relieve y normales
Genera icosferas procedurales (`0..=5`) deformadas por la misma función `height_at` del horneado, selecciona el nivel LOD según el radio proyectado en pantalla con histéresis y perturba las normales en espacio tangente en `Zebes`.
- **Variable en código**: `ICOSPHERE_THRESHOLDS_PX = [8.0, 18.0, 40.0, 85.0, 160.0]`, `DEFAULT_HYSTERESIS = 0.12` (`src/lod.rs`) y `deformation_amplitude` / `noise_frequency` (`src/scene.rs`, teclas `,`/`.` y `[`/`]`).

| Normales ON | Deformación alta |
|:---:|:---:|
| ![Normales ON](docs/captures/normales_on.png) | ![Deformación alta](docs/captures/deform_alta.png) |

### Superficie
Distribuye instancias sobre la esfera con espiral de Fibonacci, perturbación `SplitMix64` y filtrado por ruido procedural, orientando cada objeto según la normal deformada y aplicando *frustum culling* y LOD por distancia.
- **Variable en código**: `surface_seed = 42` (`src/main.rs`, teclas `I`/`O`), `MAX_PROP_DISTANCE = 32.0` (`src/main.rs`) e `INSTANCE_LOD_DIST = 12.0` (`src/lod.rs`).

| Semilla 42 | Semilla 99 |
|:---:|:---:|
| ![Semilla 42](docs/captures/semilla_42.png) | ![Semilla 99](docs/captures/semilla_99.png) |

### HUD
Dibuja la barra de título, telemetría, contadores de culling/LOD, selección de cuerpos (`Tab`) y el panel de ayuda (`H`) mediante una fuente bitmap `5×7` propia sobre un `OverlayBuffer` RGBA compuesto al presentar sin texto de Raylib.
- **Variable en código**: `GLYPH_WIDTH = 5`, `GLYPH_HEIGHT = 7` y tabla `CONTROLS` (`src/hud.rs`).

![Panel de ayuda HUD](docs/captures/ayuda_hud.png)

## Funcionalidades o criterios evaluables

| Criterio | Pts | Cumple | Dónde verlo | Archivo y función | Parámetro modificable |
|---|---:|:---:|---|---|---|
| Pipeline propio (MVP, clip near, backface, z-buffer, tiles) | 20 | ✅ | `culling_on.png` | `src/math.rs`, `src/clipping.rs::clip_near_plane`, `src/rasterizer.rs::rasterize_visibility_tiled` | `TILE_SIZE`, `R` |
| Icosfera procedural, deformación, normales y LOD | 15 | ✅ | `lod_cerca.png`, `normales_on.png`, `deform_alta.png` | `src/icosphere.rs::deform_icosphere_for_shader`, `src/lod.rs::select_detail_level_hysteresis` | `, / .`, `[ / ]`, `Z`, `B` |
| Frustum culling y rendimiento (>= 30 FPS) | 10 | ✅ | `culling_on.png` vs `culling_off.png` | `src/culling.rs::Frustum::from_vp`, `docs/rendimiento.md` | `X`, `R`, `T`, `F3` |
| Navegación 6-DOF, gravedad, colisión y modo local | 15 | ✅ | `modo_local.png` | `src/physics.rs::PhysicsStepper`, `src/nav.rs::NavState` | `G`, `L`, `GRAVITY_G` |
| Cuerpos celestes (máx. 5 puntuables) y shaders (máx. 4) | 20 | ✅ | `sistema_completo.png` + 9 capturas | `src/scene.rs::solar_system`, `src/shaders/*.rs` | `Tab`, `9 / 0`, `4 / 5` |
| Distribución procedural en superficie y modelos 3D | 10 | ⚠️ (6) | `semilla_42.png`, `semilla_99.png` | `src/surface_scatter.rs::scatter_for_body`, `src/surface_props.rs::load_surface_props` | `I / O`, `/` |
| Nave 3D con LODs | 5 | ✅ | `nave.png` | `src/mesh_lod.rs::load_ship_lods`, `assets/ship_lod{0,1,2}.obj` | `Z` / distancia |
| HUD propio (sin texto raylib), controles en vivo y docs | 10 | ✅ | `ayuda_hud.png` | `src/hud.rs::render_hud`, `src/docs_capture.rs` | `H`, `Tab`, `9/0`, `4/5` |

- **Total bruto**: **101 / 105**
- **Total (`min(100, total)`)**: **100 / 100** (detalle completo en [docs/auditoria-rubrica.md](docs/auditoria-rubrica.md)).
- **Características evaluables**: pipeline por software sin rasterizado/texto de Raylib, icosfera deformada con LOD y normales, frustum culling con contadores, navegación 6-DOF con gravedad y modo local, 9 shaders procedurales distintos con capa estática + viva, distribución Fibonacci + ruido sobre superficie, y HUD propio con ajuste de parámetros en vivo.

## Verificación de reglas

1. **Sin funciones 3D ni texto de Raylib para renderizar/HUD**: ✅ Cumplido; Raylib solo crea la ventana, lee entrada y presenta las dos texturas (`framebuffer` + `OverlayBuffer` en `src/framebuffer.rs::draw_with_overlay`).
2. **Mínimo 30 FPS a resolución interna por defecto (`400×300`)**: ✅ Cumplido (`58.6–135.5 FPS` en `docs/rendimiento.md` y `36.5–217.9 FPS` en los shaders nuevos).
3. **Al menos 5 planetas y 4 shaders procedurales distintos**: ✅ Cumplido (1 estrella + 8 planetas con 9 shaders distintos sin repetir técnicas).
4. **Una sola función `height_at` para deformación, horneado y normales**: ✅ Cumplido (`src/surface_map.rs::shader_height` alimenta `deform_icosphere_for_shader`, `BakedMaps` y `surface_point_and_normal`).
5. **Distribución procedural reproducible sin posiciones hardcodeadas**: ✅ Cumplido (`src/rng.rs::SplitMix64` + `src/surface_scatter.rs::fibonacci_sphere_points`).

## Estructura del proyecto

```text
proy3-graficas/
├── .github/workflows/ci.yml
├── assets/
│   ├── ship.obj
│   └── sphere.obj
├── docs/
│   ├── captures/                # 23 capturas oficiales (--docs)
│   ├── arquitectura.md
│   ├── auditoria-paletas.md
│   ├── auditoria-rubrica.md
│   ├── blender.md
│   ├── defensa.md
│   ├── guion-video.md
│   ├── rendimiento.md
│   └── reuse-audit.md
├── src/
│   ├── camera.rs
│   ├── clipping.rs
│   ├── culling.rs
│   ├── docs_capture.rs
│   ├── fragment.rs
│   ├── framebuffer.rs
│   ├── hud.rs
│   ├── icosphere.rs
│   ├── lod.rs
│   ├── main.rs
│   ├── math.rs
│   ├── mesh_lod.rs
│   ├── nav.rs
│   ├── noise.rs
│   ├── obj.rs
│   ├── perf.rs
│   ├── physics.rs
│   ├── primitive.rs
│   ├── rasterizer.rs
│   ├── rng.rs
│   ├── scene.rs
│   ├── scene_config.rs
│   ├── shaders/                 # star, rocky, gas_giant, ardens, abismo, basquia, vestigio, mural, fundido, ship
│   ├── shaders.rs
│   ├── ship.rs
│   ├── sim.rs
│   ├── sky.rs
│   ├── surface_map.rs
│   ├── surface_props.rs
│   ├── surface_scatter.rs
│   └── vertex.rs
├── Cargo.toml
└── README.md
```

## Limitaciones conocidas

- Los 4 modelos `.obj` de superficie (`assets/surface/{tree,rock,building,monument}.obj`) no han sido agregados aún; el motor utiliza los marcadores procedurales de respaldo de `src/surface_props.rs`.
- Los LOD de la nave (`assets/ship_lod0.obj`, `ship_lod1.obj`, `ship_lod2.obj`, con `3000`/`800`/`610` triángulos) ya se cargan desde archivo; el respaldo por decimación en `src/mesh_lod.rs::load_ship_lods` se mantiene únicamente si faltaran.
- El menú principal y la galería de trofeos ya están implementados; el sistema de favoritos cuenta con 3 cuerpos `Ready` en la galería (`Cubo de Rubik`, `Trifuerza` y `Asteroide de FF7`, construidos con `src/mesh_builder.rs` y `src/bodies.rs`) y 7 entradas en estado `Placeholder` ("en construcción"). El vuelo del sistema de favoritos permanecerá deshabilitado hasta contar con su estrella (`Sol de los Phoenix Suns`).

## Dependencias principales

- `raylib = "6.0.0"` (con `SUPPORT_IMAGE_GENERATION`): ventana, lectura de teclado/ratón, carga/actualización de textura y exportación de PNG.
- Biblioteca estándar de Rust (`std::thread::scope`): paralelismo por bandas/tiles sin dependencias externas.

## Autor

**Marcelo Detlefsen** - Carné 24554  
*Universidad del Valle de Guatemala*  
*Gráficas por Computadora - Proyecto 3*
