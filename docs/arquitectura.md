# Arquitectura del Motor y Pipeline de Renderizado

## Convenciones Matemáticas

- **Sistema de coordenadas**: mano derecha (`+X` derecha, `+Y` arriba, `+Z` hacia el observador).
- **Cámara**: en espacio de vista mira hacia `-Z` (`forward = -Z`, `right = +X`, `up = +Y`).
- **Vectores columna**: todas las transformaciones se aplican por la izquierda (`v' = P * V * M * v`), con composición `M = T * R * S` (`src/math.rs`).
- **NDC (Normalized Device Coordinates)**: `x, y, z ∈ [-1, 1]` tras la división perspectiva (`w = -z_view`), verificado en `math::tests::perspective_maps_near_far_and_optical_axis_and_preserves_distance_w`.

## Orden de Etapas por Cuadro

1. **Entrada y reloj de simulación** (`src/main.rs`, `src/sim.rs`): lectura de teclado/ratón y avance de `SimClock` (pausa con `Enter`, escala con `-` / `=`).
2. **Paso fijo de física y navegación** (`src/physics.rs`, `src/nav.rs`): acumulador a **`FIXED_DT = 1.0 / 120.0` s** (`MAX_SUBSTEPS = 8`), integración de gravedad con `GRAVITY_G = 7.2`, colisión con radio efectivo deformado (`effective_radius`) y transición suave entre modo `Global` y `Local` (`LOCAL_ENTER_RADII = 2.5`, `LOCAL_EXIT_RADII = 3.0`).
3. **Escena, Frustum Culling y LOD** (`src/scene.rs`, `src/culling.rs`, `src/lod.rs`): cálculo de órbitas/rotaciones desde `scene_config.rs`, extracción de los 6 planos del frustum desde `VP`, descarte de esferas envolventes fuera del frustum y selección de LOD de icosfera (`0..=5`) y nave (`0..=2`) con histéresis (`DEFAULT_HYSTERESIS = 0.12`).
4. **Etapa de vértices y recorte** (`src/primitive.rs`, `src/clipping.rs`): transformación `MVP`, recorte contra el plano cercano (`clip_near_plane`) generando 0, 1 o 2 triángulos con atributos interpolados en espacio de clip, y proyección a pantalla.
5. **Rasterización por tiles con buffer de visibilidad** (`src/rasterizer.rs`): pantalla dividida en tiles de **`TILE_SIZE = 64`×`64`** píxeles procesados en paralelo (`rayon`); cada tile mantiene su `depth_buffer` y `tri_id_buffer` antes de invocar cualquier shader (eliminando el overdraw).
6. **Sombreado de materiales y cielo** (`src/shaders.rs`, `src/surface_map.rs`, `src/sky.rs`): solo los píxeles visibles ejecutan `fragment_shader` usando mapas estáticos horneados (`512×256`) + capa animada + atenuación solar + brillo atmosférico; los píxeles vacíos (`INVALID_TRI_ID`) evalúan el cielo estelar y el halo solar.
7. **Presentación y HUD RGBA** (`src/framebuffer.rs`, `src/hud.rs`): escalado *nearest-neighbor* del framebuffer interno (`400×300` por defecto, escalas `1/2`, `1/3`, `1/1`) a la ventana (`800×600`) y composición del `OverlayBuffer` RGBA propio (fuente `5×7` sin `draw_text` de raylib).

## Constructor de Mallas Procedurales y Color por Vértice

- **`src/mesh_builder.rs`**: define `Mesh` (posiciones, normales, colores por vértice, UVs e índices) con primitivas estancas (`box_mesh`, `rounded_box`, `cylinder`, `cone`, `sphere`, `torus`, `extruded_prism`, `loft_rings`), transformaciones (`translate`, `rotate_x/y/z`, `scale`), fusión (`merge`), verificación topológica (`is_watertight`, `outward_normal_ratio`) y caché de reconstrucción por nivel de detalle (`LodMeshCache`).
- **Color por vértice con corrección de perspectiva** (`src/vertex.rs`, `src/clipping.rs`, `src/primitive.rs`, `src/rasterizer.rs`): `Vertex::color` se interpola linealmente en espacio de clip al recortar y con pesos en perspectiva `(pw0, pw1, pw2)` en `interpolate_screen_fragment` hasta `Fragment::color`.
- **Cuerpos procedurales (`src/bodies.rs`, `src/showcase.rs`)**:
  - **Cubo de Rubik** (`Shader::Chunky`): 27 cubitos con holgura y calcomanías por cara en 3 niveles de detalle (`108` / `324` / `3564` tris).
  - **Trifuerza** (`Shader::Triforce`): tres prismas triangulares equiláteros con centro vacío y shader dorado con destello animado (`24` / `42` / `78` tris).
  - **Meteoro de FF7** (`Shader::Meteor`): cometa con cabeza de icosfera vítrea y trazos blancos en espiral horneados, cola de 24–38 pinceladas con degradado por vértice y un orbe separado (`672` / `1316` / `2880` tris).
  - **Cubo Skewb** (`Shader::Chunky`): cubo negro de lado 1 con rombo central y 4 triángulos rectángulos isósceles por cara elevados (`12` / `48` / `372` tris).
  - **Megaminx** (`Shader::Chunky`): dodecaedro regular negro con 11 pegatinas por cara en 12 colores (`36` / `312` / `1668` tris).
  - **Joy-Con izquierdo / derecho** (`Shader::Chunky`): carcasa redondeada gris mate espejada, riel lateral `#00C3E3` / `#FF4554`, palanca, cruceta/ABXY, `-`/`+`, captura/Home y SL/SR (`240` / `392` / `696` y `264` / `424` / `744` tris).
  - **Sol de los Phoenix Suns** (`Shader::PhoenixSuns`): esfera emisiva con proyección azimutal del logo alrededor de Z en ambos polos y halo dorado (`80` / `320` / `1280` tris).
  - **Escudo del Tottenham Hotspur** (`Shader::Chunky`): placa biselada azul marino/blanco, barras de cinta, balón con parches y gallo de primitivas con giro oscilante.
  - **AirPods** (`Shader::Chunky`): estuche redondeado (`~2.8:2.1:1`) con tapa a 70°, bisagra cilíndrica, LED frontal y auriculares espejados.
  - **Raqueta de tenis** (`Shader::Chunky`): marco ovalado, garganta en Y, mango con bandas y cuerdas como asteroides dispersos en cuadrícula `14×16`.
