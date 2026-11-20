# Textos de interfaz en lenguaje claro

Tabla de revisión con todos los textos visibles para el usuario (menú, HUD de vuelo, panel de ayuda, galería de trofeos, descripciones y mensajes de consola).

| Texto anterior | Texto nuevo | Dónde se ve |
|---|---|---|
| `Renderizador por software en Rust` | `Simulador espacial 3D hecho desde cero en Rust` | Menú principal (subtítulo) |
| `en construccion` | `en construcción` | Menú principal y lista de galería |
| `W/S o Flechas: navegar \| Enter: elegir \| Esc: salir` | `W/S o Flechas: moverse · Enter: elegir · Esc: salir` | Menú principal (pie) |
| `Enter / Backspace: volver` | `Enter o Retroceso: volver` | Menú principal (panel de controles) |
| `CONTROLES (H para cerrar)` | `AYUDA Y CONTROLES (pulsa H para cerrar)` | Panel de ayuda en vuelo (`H`) |
| `Galeria \| A/D: cuerpo \| Mouse/Flechas: girar \| R: vista \| 4/5: param \| Esc: menu \| {fps} FPS` | `Galería de trofeos — A/D: cambiar objeto · Ratón o flechas: girar · R: reiniciar vista · 4/5: ajustar · Esc: menú · {fps} FPS` | Barra superior de la galería |
| `Trofeos (Showcase)` | `Trofeos` | Lista lateral de la galería |
| `Tipo: {tipo} \| Sistema: {sistema}` | `Colección: {sistema}` | Ficha de la galería |
| `Radio: {r} u \| Malla: {n} tris (L{l})` | `Tamaño: {r}` y `Triángulos: {n} · Nivel de detalle: máximo (5 de 5)` (o `{l} de 5`) | Ficha de la galería |
| `Shader: {nombre_interno}` | `Estilo visual: {nombre_visible}` | Ficha de la galería |
| `{param}(4/5): {val}` | `{param}  [teclas 4 y 5]: {val}` | Ficha de la galería |
| `Inspiracion: -` | *(Oculto cuando no hay inspiración)* / `Inspiración: {texto}` | Ficha de la galería |
| `Descripcion:` | `Descripción:` | Ficha de la galería |
| `FPS:{fps} ({ms}ms) \| {w}x{h} {t}h \| Tiempo:{s}x` | `FPS: {fps} ({ms} ms) · Imagen: {w}x{h} · Núcleos: {t} · Tiempo: {s}x` | HUD de vuelo (línea 1) |
| `Modo: {m} \| Alt: {a} u \| Vel:{v}` | `Vuelo: {m} · Altura: {a} · Velocidad: {v}` | HUD de vuelo (línea 2) |
| `Grav(G):ON/OFF \| Semilla(I/O):{s} \| Inst:{d}/{t}(ON/OFF)` | `Gravedad [G]: SÍ/NO · Semilla [I/O]: {s} · Objetos: {d}/{t} (ocultar lejanos / ver todos)` | HUD de vuelo (línea 3) |
| `Cull(X):ON/OFF {b}->{a} tris \| Nave:L{l}({t}) \| LOD(Z):...` | `Objetos descartados por no verse [X]: SÍ/NO · {b} -> {a} triángulos · Nave: nivel {l+1} de 3 ({t} triángulos)` + línea `Nivel de detalle [Z]: ...` | HUD de vuelo (líneas 4–5) |
| `Sel(Tab): {n} \| R(9/0):{r} \| {p}(4/5):{v}` | `Elegido [Tab]: {n} · Tamaño [9/0]: {r} · {p} [4/5]: {v}` | HUD de vuelo (línea 6) |
| `Etapas ms -> V:{v} R:{r} S:{s} Pres:{p}` | `Tiempos (ms) — Vértices: {v} · Dibujo: {r} · Color: {s} · Pantalla: {p}` | HUD de vuelo (`F3`) |
| `COLISION CON SUPERFICIE: {nombre}` | `¡CONTACTO CON LA SUPERFICIE DE {nombre}!` | HUD de vuelo (colisión) |
| `Local({n}) [Auto/ForzarLocal/ForzarGlobal]` | `Cerca de {n} (Automático / Fijo cerca / Fijo libre)` / `Espacio libre (...)` | HUD de vuelo |
| `Intensidad corona`, `Contraste relieve`, `Brillo causticas`, `Brillo oilstick`, `Contraste VHS` | `Intensidad de la corona`, `Contraste del relieve`, `Brillo del agua`, `Brillo de los trazos`, `Contraste visual` | Parámetros en vivo (`4`/`5`) |
| `Shader: Star`, `Shader: Chunky`, `Shader: Meteoro`, etc. | `Estrella con granulación`, `Bloques de luz y bordes`, `Cometa de pinceladas`, etc. (`nombre_visible` en `Showcase`) | Ficha de galería |
| `Empuje adelante / atrás`, `Impulso (boost)`, `Cabeceo y guiñada...`, `Forzar LOD / Frustum culling / Wireframe`, `Backface culling / Re-hornear / Captura PNG`, `Métricas / Resolución interna / Hilos` | Textos explicativos en español claro sin jerga (`Acelerar hacia adelante o atrás`, `Turbo para ir más rápido`, `FPS (cuadros por segundo: mide la fluidez)`, `Ver la malla (alambre)`, `Relieve fino (normal map)`, etc.) | Tabla `CONTROLS` (`H`, menú y README) |
| Descripciones técnicas de los 22 cuerpos (`fbm`, `domain warp`, `Voronoi`, `shader chunky`, `oilstick`, `#103F91`, etc.) | Descripciones de 1–2 frases en español claro sobre qué es y qué tiene de especial cada objeto | Registro `SHOWCASE_ENTRIES` / Galería |
| `Aviso: no se encontró assets/surface/*.obj...` (4 líneas) | `Nota: se usan figuras generadas para los detalles de superficie (no hay modelos externos en assets/surface/).` (1 línea) | Consola al iniciar vuelo |
| `Mapas re-horneados ({w}x{h}) en {ms} ms` | `Texturas de superficie recalculadas ({w}x{h}) en {ms} ms.` | Consola al pulsar `K` |
| `LODs de nave listos (desde_archivo=..., respaldo_decimacion=...): lod0=... tris...` | `Modelos de la nave listos (desde archivo=..., generados=...): alto ... triángulos, medio ... triángulos, bajo ... triángulos.` | Consola al iniciar |
