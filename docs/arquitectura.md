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
  - **Sol de los Phoenix Suns** (`Shader::PhoenixSuns`): placas emisivas extruidas con bisel de la explosión solar y estela de rayos medidas desde `refs/suns_logo.png` (`#F9A01B`) y balón de basquetbol 3D al frente (`#E56020` / `#934414`) con costuras giratorias y oscilación `±30°` (`1336` tris en detalle máximo).
  - **Cresta del Tottenham** (`Shader::Spurs`): silueta del gallo medida desde `refs/spurs_crest.jpg` extruida con bisel y sombreado chunky en 3 bandas sobre un balón 3D azul marino (`#001C58`) con costuras blancas, parche de cordones y oscilación `±35°` (`1300` tris en detalle máximo).
  - **AirPods** (`Shader::Chunky`): estuche redondeado (`~2.8:2.1:1`) con tapa a 70°, bisagra cilíndrica, LED frontal y auriculares espejados.
  - **Raqueta de tenis** (`Shader::Chunky` con barniz `material_id = 1`): Wilson Ultra medida desde `refs/raqueta.webp` (`docs/medidas-raqueta.md`), aro barrido con sección rectangular redondeada (`#61A4C5`→`#4C789E` y marino `#2D313C`), garganta en Y con puente, mango octogonal gris carbón con envoltura diagonal y cuerdas `11×13` de asteroides octaédricos alargados con el logo `W` rojo (`#C8261E`).
  - **Casco de Samus** (`Shader::Chunky` con barniz `material_id = 1` y visor emisivo `material_id = 2`): cúpula terracota `#AC6A52` ajustada al perfil radial medido de `refs/samus_helmet.jpg`, 3 surcos oscuros `#4C322C` adelgazados con Zhang–Suen, visor angular medido de 20 vértices (`#ECFEE6`/`#507B38`), barbilla con 3 rendijas, orejeras simétricas y mangueras acanaladas (`2412` tris en L5).
  - **La Bombonera** (`Shader::Chunky`): herradura en D de 3 niveles escalonados empinados (`#455681` / `#C49940`, proporciones medidas `30.6%`/`32.4%`/`37.0%`, tazón liso en `L0`), letras `"CABJ"` y `"BOCA"` con la fuente `5×7` del HUD, palcos grises con ventanas `12×6`, torres con balcones, torres de luces, zócalo y campo `105:68` a 14 franjas con líneas y arcos (`3252` tris en L5).
